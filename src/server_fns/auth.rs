use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use surrealdb::types::SurrealValue;

/// **What is it?**
/// A struct containing the essential details of the currently authenticated user.
///
/// **Why does it exist?**
/// It exists to provide a serializable, client-friendly representation of the user without exposing sensitive backend data like password hashes.
///
/// **How should it be used?**
/// Use this struct on the frontend to display the user's name, email, or to verify that an active session exists.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserInfo {
    /// The unique identifier for the user.
    pub id: String,
    /// The user's username.
    pub username: String,
    /// The user's email address.
    pub email: String,
}

/// **What is it?**
/// A utility function that converts a SurrealDB `RecordId` into a standard "table:key" string format.
///
/// **Why does it exist?**
/// It exists because SurrealDB returns native `RecordId` types in the backend, but the frontend and HTTP session cookies require simple string identifiers.
///
/// **How should it be used?**
/// Call this when serializing a user record from the database to extract their string-based ID for session storage or `UserInfo`.
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

/// **What is it?**
/// An SSR-only struct representing the shape of a user record exactly as it is returned from SurrealDB.
///
/// **Why does it exist?**
/// It exists to safely deserialize the database query result, including its native `RecordId`, before mapping it to the frontend `UserInfo` struct.
///
/// **How should it be used?**
/// Use this type internally within backend queries (like `SELECT * FROM user`) as the target struct for deserialization.
#[cfg(feature = "ssr")]
#[derive(Deserialize, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
pub struct UserDbRow {
    /// The underlying SurrealDB record ID for the user.
    pub id: surrealdb::types::RecordId,
    /// The user's username.
    pub username: String,
    /// The user's email address.
    pub email: String,
}

#[cfg(feature = "ssr")]
impl UserDbRow {
    /// Converts a UserDbRow into a UserInfo struct for the client.
    pub fn into_user_info(self) -> UserInfo {
        UserInfo {
            id: record_id_to_string(&self.id),
            username: self.username,
            email: self.email,
        }
    }
}

/// **What is it?**
/// A server function that registers a new user with the given username, email, and password.
///
/// **Why does it exist?**
/// It exists to handle account creation securely, hashing the provided password and creating the initial user record in the database.
///
/// **How should it be used?**
/// Call this from the frontend registration form when a new user signs up.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn register(
    /// The desired username for the new account.
    username: String,
    /// The user's email address.
    email: String,
    /// The password for the new account.
    password: String,
) -> Result<UserInfo, ServerFnError> {
    use crate::auth::{hash_password, create_session};
    use crate::db::db;
    use crate::error::internal_error;

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
        .map_err(|e| internal_error("Password hashing failed", e))?;

    let mut response = db()
        .query("CREATE user SET username = $username, email = $email, password_hash = $hash RETURN id, username, email")
        .bind(("username", username))
        .bind(("email", email))
        .bind(("hash", password_hash))
        .await
        .map_err(|e| internal_error("Registration query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Registration query error", err_msg));
    }

    let result: Option<UserDbRow> = response.take(0)
        .map_err(|e| internal_error("Registration result parse failed", e))?;

    let user = result.ok_or_else(|| ServerFnError::new("Failed to create user"))?.into_user_info();

    create_session(&user.id).await?;

    Ok(user)
}

/// **What is it?**
/// A server function that authenticates an existing user and establishes an active HTTP session.
///
/// **Why does it exist?**
/// It exists to securely verify credentials and generate the session cookies needed for subsequent backend requests.
///
/// **How should it be used?**
/// Call this from the frontend login form when a user submits their username and password.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn login(
    /// The user's username.
    username: String, 
    /// The user's password.
    password: String
) -> Result<UserInfo, ServerFnError> {
    use crate::auth::verify_password;
    use crate::db::db;
    use crate::error::internal_error;

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

    let mut response = db()
        .query("SELECT id, username, email, password_hash FROM user WHERE username = $username LIMIT 1")
        .bind(("username", username))
        .await
        .map_err(|e| internal_error("Login query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Login query error", err_msg));
    }

    let result: Option<UserLoginRow> = response.take(0)
        .map_err(|e| internal_error("Login result parse failed", e))?;

    let user_row = result.ok_or_else(|| ServerFnError::new("Invalid credentials"))?;

    if !verify_password(&password, &user_row.password_hash)
        .map_err(|e| internal_error("Password verification failed", e))?
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

/// **What is it?**
/// A server function that logs out the current user by destroying their active HTTP session.
///
/// **Why does it exist?**
/// It exists to revoke a user's access credentials on the backend and clear their session, forcing a re-login for any future secure actions.
///
/// **How should it be used?**
/// Call this function when the user clicks a "Log Out" button in the application.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn logout() -> Result<(), ServerFnError> {
    crate::auth::destroy_session().await
}

/// **What is it?**
/// A server function that retrieves the current authenticated user's profile information based on their active session.
///
/// **Why does it exist?**
/// It exists to check if the incoming request has a valid session cookie and, if so, loads the corresponding user record from the database.
///
/// **How should it be used?**
/// Call this repeatedly during application startup or route transitions on the frontend to determine if a user is logged in.
#[server]
#[tracing::instrument(level = "info", skip_all)]
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
