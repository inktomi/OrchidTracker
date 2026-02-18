use async_trait::async_trait;
use surrealdb::types::SurrealValue;
use tower_sessions::session::{Id, Record};
use tower_sessions::session_store;

use crate::db::db;

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
            let mut response = db()
                .query("CREATE type::record('session', $id) SET data = $data, expiry = $expiry")
                .bind(("id", id))
                .bind(("data", data.clone()))
                .bind(("expiry", expiry))
                .await
                .map_err(|e| session_store::Error::Backend(e.to_string()))?;

            let errors = response.take_errors();
            if errors.is_empty() {
                return Ok(());
            }

            // Collision — generate a new ID and retry
            record.id = Id::default();
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

        db()
            .query("UPSERT type::record('session', $id) SET data = $data, expiry = $expiry")
            .bind(("id", id))
            .bind(("data", data))
            .bind(("expiry", expiry))
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let id = session_id.to_string();
        let now = time::OffsetDateTime::now_utc().unix_timestamp();

        let mut response = db()
            .query("SELECT * FROM type::record('session', $id) WHERE expiry > $now")
            .bind(("id", id.clone()))
            .bind(("now", now))
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        let errors = response.take_errors();
        if !errors.is_empty() {
            return Err(session_store::Error::Backend(format!("{:?}", errors)));
        }

        let row: Option<SessionRow> = response
            .take(0)
            .map_err(|e| session_store::Error::Decode(e.to_string()))?;

        match row {
            Some(row) => {
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
            None => {
                // Clean up expired session if it exists
                let _ = db()
                    .query("DELETE type::record('session', $id)")
                    .bind(("id", id))
                    .await;
                Ok(None)
            }
        }
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        let id = session_id.to_string();

        db()
            .query("DELETE type::record('session', $id)")
            .bind(("id", id))
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }
}

impl SurrealSessionStore {
    pub async fn cleanup_expired(&self) {
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        let result = db()
            .query("DELETE session WHERE expiry < $now")
            .bind(("now", now))
            .await;
        if let Err(e) = result {
            tracing::warn!("Failed to clean up expired sessions: {}", e);
        }
    }
}

#[derive(serde::Deserialize, surrealdb::types::SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct SessionRow {
    data: String,
    expiry: i64,
}
