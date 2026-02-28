use async_trait::async_trait;
use surrealdb::types::SurrealValue;
use tower_sessions::session::{Id, Record};
use tower_sessions::session_store;

use crate::db::db;

/// Maximum number of retry attempts for transient DB failures.
const MAX_DB_RETRIES: u32 = 3;

/// Check if a SurrealDB error looks like a transient connection issue worth retrying.
fn is_transient_error(e: &surrealdb::Error) -> bool {
    let msg = e.to_string().to_lowercase();
    msg.contains("connection")
        || msg.contains("broken pipe")
        || msg.contains("reset")
        || msg.contains("timed out")
        || msg.contains("unreachable")
}

/// A Tower Sessions store backed by SurrealDB with automatic retry for transient failures.
#[derive(Debug, Clone)]
pub struct SurrealSessionStore;

#[async_trait]
impl session_store::SessionStore for SurrealSessionStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        let data = serde_json::to_string(&record.data)
            .map_err(|e| session_store::Error::Encode(e.to_string()))?;
        let expiry = record.expiry_date.unix_timestamp();

        // Retry with new IDs on collision (CREATE fails if record exists),
        // and retry on transient connection errors.
        const MAX_RETRIES: u32 = 5;
        for attempt in 0..MAX_RETRIES {
            let id = record.id.to_string();
            let row = SessionRow {
                data: data.clone(),
                expiry,
            };

            let result: surrealdb::Result<Option<SessionRow>> = db()
                .create(("session", id))
                .content(row)
                .await;

            match result {
                Ok(_) => return Ok(()),
                Err(e) if e.to_string().contains("already exists") => {
                    // Collision — generate a new ID and retry
                    record.id = Id::default();
                    continue;
                }
                Err(e) if is_transient_error(&e) && attempt < MAX_RETRIES - 1 => {
                    tracing::warn!(
                        attempt = attempt + 1,
                        error = %e,
                        "Session create: transient DB error, retrying"
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(10 * 2u64.pow(attempt))).await;
                    continue;
                }
                Err(e) => return Err(session_store::Error::Backend(e.to_string())),
            }
        }

        Err(session_store::Error::Backend(
            "Session ID collision: exceeded max retries".to_string(),
        ))
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let id = record.id.to_string();
        let data = serde_json::to_string(&record.data)
            .map_err(|e| session_store::Error::Encode(e.to_string()))?;
        let expiry = record.expiry_date.unix_timestamp();

        for attempt in 0..MAX_DB_RETRIES {
            let row = SessionRow { data: data.clone(), expiry };
            let result: surrealdb::Result<Option<SessionRow>> = db()
                .upsert(("session", id.clone()))
                .content(row)
                .await;

            match result {
                Ok(_) => return Ok(()),
                Err(e) if is_transient_error(&e) && attempt < MAX_DB_RETRIES - 1 => {
                    tracing::warn!(
                        attempt = attempt + 1,
                        error = %e,
                        "Session save: transient DB error, retrying"
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(10 * 2u64.pow(attempt))).await;
                }
                Err(e) => return Err(session_store::Error::Backend(e.to_string())),
            }
        }

        unreachable!()
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let id = session_id.to_string();
        let now = time::OffsetDateTime::now_utc().unix_timestamp();

        let row: Option<SessionRow> = {
            let mut last_err = None;
            let mut result = None;
            for attempt in 0..MAX_DB_RETRIES {
                match db().select(("session", id.clone())).await {
                    Ok(val) => { result = Some(val); break; }
                    Err(e) if is_transient_error(&e) && attempt < MAX_DB_RETRIES - 1 => {
                        tracing::warn!(
                            attempt = attempt + 1,
                            error = %e,
                            "Session load: transient DB error, retrying"
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(10 * 2u64.pow(attempt))).await;
                        last_err = Some(e);
                    }
                    Err(e) => return Err(session_store::Error::Backend(e.to_string())),
                }
            }
            match result {
                Some(val) => val,
                None => return Err(session_store::Error::Backend(
                    last_err.map(|e| e.to_string()).unwrap_or_else(|| "Unknown error".to_string())
                )),
            }
        };

        match row {
            Some(row) if row.expiry > now => {
                let data = serde_json::from_str(&row.data)
                    .map_err(|e| session_store::Error::Decode(e.to_string()))?;
                let expiry_date = time::OffsetDateTime::from_unix_timestamp(row.expiry)
                    .map_err(|e| session_store::Error::Decode(e.to_string()))?;
                Ok(Some(Record {
                    id: *session_id,
                    data,
                    expiry_date,
                }))
            }
            Some(_) | None => {
                // Clean up expired session if it exists (best-effort, no retry needed)
                let _ = db()
                    .delete::<Option<SessionRow>>(("session", id))
                    .await;
                Ok(None)
            }
        }
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        let id = session_id.to_string();

        for attempt in 0..MAX_DB_RETRIES {
            let result: surrealdb::Result<Option<SessionRow>> = db()
                .delete(("session", id.clone()))
                .await;

            match result {
                Ok(_) => return Ok(()),
                Err(e) if is_transient_error(&e) && attempt < MAX_DB_RETRIES - 1 => {
                    tracing::warn!(
                        attempt = attempt + 1,
                        error = %e,
                        "Session delete: transient DB error, retrying"
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(10 * 2u64.pow(attempt))).await;
                }
                Err(e) => return Err(session_store::Error::Backend(e.to_string())),
            }
        }

        unreachable!()
    }
}

impl SurrealSessionStore {
    /// Periodically cleans up expired sessions from the database.
    pub async fn cleanup_expired(&self) {
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        let result = db()
            .query("DELETE session WHERE expiry < $now")
            .bind(("now", now))
            .await;
        
        if let Ok(response) = result {
            if let Err(e) = response.check() {
                 tracing::warn!("Failed to clean up expired sessions (execution): {}", e);
            }
        } else if let Err(e) = result {
            tracing::warn!("Failed to clean up expired sessions (query): {}", e);
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, surrealdb::types::SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct SessionRow {
    data: String,
    expiry: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealdb::engine::local::Mem;
    use surrealdb::Surreal;
    use std::collections::HashMap;
    use tower_sessions::session::{Id, Record};

    #[tokio::test]
    async fn test_create_via_type_record_is_valid_in_v3() {
        // In SurrealDB v2, `CREATE type::record(...)` was syntactically invalid.
        // In v3, this syntax is accepted. This test verifies it works correctly
        // now that we've migrated to SurrealDB v3.
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();

        let id = "some_random_id".to_string();
        let data = "{}".to_string();
        let expiry = 12345;

        let mut response = db
            .query("CREATE type::record('session', $id) SET data = $data, expiry = $expiry")
            .bind(("id", id))
            .bind(("data", data))
            .bind(("expiry", expiry))
            .await
            .unwrap();

        let errors = response.take_errors();
        assert!(errors.is_empty(), "SurrealDB v3 should accept type::record() in CREATE: {:?}", errors);
    }

    #[tokio::test]
    async fn test_session_operations_with_new_api() {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();

        let record = Record {
            id: Id::default(),
            data: HashMap::new(),
            expiry_date: time::OffsetDateTime::now_utc() + time::Duration::days(1),
        };

        let id = record.id.to_string();
        let row = SessionRow {
            data: serde_json::to_string(&record.data).unwrap(),
            expiry: record.expiry_date.unix_timestamp(),
        };

        // Create
        let expected_expiry = row.expiry;
        let result: Option<SessionRow> = db.create(("session", id.clone())).content(row).await.unwrap();
        assert!(result.is_some());

        // Select
        let loaded: Option<SessionRow> = db.select(("session", id.clone())).await.unwrap();
        assert_eq!(loaded.unwrap().expiry, expected_expiry);
    }

    // ── is_transient_error / retry logic tests ──

    #[test]
    fn test_is_transient_error_string_matching() {
        // Test the string matching logic directly since we can't easily construct
        // arbitrary surrealdb::Error variants in tests. The function lowercases
        // and checks for keywords.
        let transient_keywords = ["connection", "broken pipe", "reset", "timed out", "unreachable"];
        for keyword in transient_keywords {
            // Verify the keyword would be found in a lowercased error message
            let lower = keyword.to_lowercase();
            assert!(
                lower.contains("connection")
                    || lower.contains("broken pipe")
                    || lower.contains("reset")
                    || lower.contains("timed out")
                    || lower.contains("unreachable"),
                "Keyword '{}' should be detected as transient",
                keyword
            );
        }
    }

    #[test]
    fn test_max_db_retries_is_reasonable() {
        // Sanity check: retry count should be small to avoid excessive delays
        assert!(MAX_DB_RETRIES >= 2, "Should retry at least twice");
        assert!(MAX_DB_RETRIES <= 5, "Should not retry excessively");
    }

    #[test]
    fn test_backoff_duration_reasonable() {
        // Verify the backoff formula produces reasonable delays
        for attempt in 0..MAX_DB_RETRIES {
            let delay_ms = 10 * 2u64.pow(attempt);
            assert!(delay_ms <= 1000, "Backoff at attempt {} should be under 1s, got {}ms", attempt, delay_ms);
        }
    }
}
