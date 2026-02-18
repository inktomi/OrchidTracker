use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use surrealdb::types::SurrealValue;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
}

/// Convert a SurrealDB RecordId to a "table:key" string for session storage and UserInfo.
#[cfg(feature = "ssr")]
pub fn record_id_to_string(id: &surrealdb::types::RecordId) -> String {
    use surrealdb::types::RecordIdKey;
    let key = match &id.key {
        RecordIdKey::String(s) => s.clone(),
        RecordIdKey::Number(n) => n.to_string(),
        other => format!("{:?}", other),
    };
    format!("{}:{}", id.table, key)
}

/// SSR-only struct matching SurrealDB's record shape (id is a RecordId, not a String).
#[cfg(feature = "ssr")]
#[derive(Deserialize, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
pub struct UserDbRow {
    pub id: surrealdb::types::RecordId,
    pub username: String,
    pub email: String,
}

#[cfg(feature = "ssr")]
impl UserDbRow {
    pub fn into_user_info(self) -> UserInfo {
        UserInfo {
            id: record_id_to_string(&self.id),
            username: self.username,
            email: self.email,
        }
    }
}

#[server]
pub async fn register(
    username: String,
    email: String,
    password: String,
) -> Result<UserInfo, ServerFnError> {
    use crate::auth::{hash_password, create_session};
    use crate::db::db;

    // Username: 1-50 chars, alphanumeric + underscore/hyphen
    if username.is_empty() || username.len() > 50 {
        return Err(ServerFnError::new("Username must be 1-50 characters"));
    }
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(ServerFnError::new("Username may only contain letters, numbers, underscores, and hyphens"));
    }
    // Email: 1-254 chars, must contain @
    if email.is_empty() || email.len() > 254 || !email.contains('@') {
        return Err(ServerFnError::new("A valid email address is required (max 254 characters)"));
    }
    // Password: 8-128 chars
    if password.len() < 8 || password.len() > 128 {
        return Err(ServerFnError::new("Password must be 8-128 characters"));
    }

    let password_hash = hash_password(&password)
        .map_err(|e| ServerFnError::new(format!("Password hash error: {}", e)))?;

    let result: Option<UserDbRow> = db()
        .query("CREATE user SET username = $username, email = $email, password_hash = $hash RETURN id, username, email")
        .bind(("username", username))
        .bind(("email", email))
        .bind(("hash", password_hash))
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?
        .take(0)
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    let user = result.ok_or_else(|| ServerFnError::new("Failed to create user"))?.into_user_info();

    create_session(&user.id).await?;

    Ok(user)
}

#[server]
pub async fn login(username: String, password: String) -> Result<UserInfo, ServerFnError> {
    use crate::auth::verify_password;
    use crate::db::db;

    if username.is_empty() || username.len() > 50 {
        return Err(ServerFnError::new("Invalid credentials"));
    }
    if password.is_empty() || password.len() > 128 {
        return Err(ServerFnError::new("Invalid credentials"));
    }

    #[derive(Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct UserLoginRow {
        id: surrealdb::types::RecordId,
        username: String,
        email: String,
        password_hash: String,
    }

    let result: Option<UserLoginRow> = db()
        .query("SELECT id, username, email, password_hash FROM user WHERE username = $username LIMIT 1")
        .bind(("username", username))
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?
        .take(0)
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    let user_row = result.ok_or_else(|| ServerFnError::new("Invalid credentials"))?;

    if !verify_password(&password, &user_row.password_hash)
        .map_err(|e| ServerFnError::new(format!("Auth error: {}", e)))?
    {
        return Err(ServerFnError::new("Invalid credentials"));
    }

    let user_id = record_id_to_string(&user_row.id);
    crate::auth::create_session(&user_id).await?;

    Ok(UserInfo {
        id: user_id,
        username: user_row.username,
        email: user_row.email,
    })
}

#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    crate::auth::destroy_session().await
}

#[server]
pub async fn get_current_user() -> Result<Option<UserInfo>, ServerFnError> {
    crate::auth::get_session_user().await
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use surrealdb::types::{RecordId, RecordIdKey};

    #[test]
    fn test_record_id_string_key_to_string() {
        let id = RecordId::new("user", "abc123");
        let result = record_id_to_string(&id);
        assert_eq!(result, "user:abc123");
    }

    #[test]
    fn test_record_id_numeric_key_to_string() {
        let id = RecordId {
            table: "user".into(),
            key: RecordIdKey::Number(42),
        };
        let result = record_id_to_string(&id);
        assert_eq!(result, "user:42");
    }

    #[test]
    fn test_record_id_roundtrip_via_parse() {
        let original = RecordId::new("user", "test123");
        let as_string = record_id_to_string(&original);
        let parsed = RecordId::parse_simple(&as_string).expect("should parse");
        assert_eq!(parsed.table, original.table);
        assert_eq!(parsed.key, original.key);
    }

    #[test]
    fn test_user_db_row_into_user_info() {
        let row = UserDbRow {
            id: RecordId::new("user", "xyz789"),
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        };
        let info = row.into_user_info();
        assert_eq!(info.id, "user:xyz789");
        assert_eq!(info.username, "alice");
        assert_eq!(info.email, "alice@example.com");
    }
}
