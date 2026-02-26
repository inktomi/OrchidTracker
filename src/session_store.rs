use async_trait::async_trait;
use surrealdb::types::SurrealValue;
use tower_sessions::session::{Id, Record};
use tower_sessions::session_store;

use crate::db::db;

/// A Tower Sessions store backed by SurrealDB.
#[derive(Debug, Clone)]
pub struct SurrealSessionStore;

#[async_trait]
impl session_store::SessionStore for SurrealSessionStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        let data = serde_json::to_string(&record.data)
            .map_err(|e| session_store::Error::Encode(e.to_string()))?;
        let expiry = record.expiry_date.unix_timestamp();

        // Retry with new IDs on collision (CREATE fails if record exists).
        const MAX_RETRIES: u32 = 5;
        for _ in 0..MAX_RETRIES {
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

        let row = SessionRow { data, expiry };

        let _: Option<SessionRow> = db()
            .upsert(("session", id))
            .content(row)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let id = session_id.to_string();
        let now = time::OffsetDateTime::now_utc().unix_timestamp();

        let row: Option<SessionRow> = db()
            .select(("session", id.clone()))
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

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
                // Clean up expired session if it exists
                let _ = db()
                    .delete::<Option<SessionRow>>(("session", id))
                    .await;
                Ok(None)
            }
        }
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        let id = session_id.to_string();

        let _: Option<SessionRow> = db()
            .delete(("session", id))
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
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
    async fn test_create_syntax_bug_reproduction() {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();

        let id = "some_random_id".to_string();
        let data = "{}".to_string();
        let expiry = 12345;

        // This is the EXACT query string that was failing and being masked!
        // In SurrealDB, CREATE type::record(...) is syntactically invalid for the CREATE statement target.
        let mut response = db
            .query("CREATE type::record('session', $id) SET data = $data, expiry = $expiry")
            .bind(("id", id))
            .bind(("data", data))
            .bind(("expiry", expiry))
            .await
            .unwrap();

        let errors = response.take_errors();
        // This assertion verifies that the query indeed causes an error. 
        // Before our fix, this error was incorrectly swallowed and assumed to be a collision.
        assert!(!errors.is_empty(), "Expected CREATE to fail due to syntax/target errors");
    }

    #[tokio::test]
    async fn test_session_operations_with_new_api() {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();

        let mut record = Record {
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
        let result: Option<SessionRow> = db.create(("session", id.clone())).content(&row).await.unwrap();
        assert!(result.is_some());

        // Select
        let loaded: Option<SessionRow> = db.select(("session", id.clone())).await.unwrap();
        assert_eq!(loaded.unwrap().expiry, row.expiry);
    }
}
