use crate::error::AppError;
use crate::server_fns::auth::UserInfo;
use leptos::prelude::*;

/// Hash a password using argon2
pub fn hash_password(password: &str) -> Result<String, AppError> {
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString, PasswordHasher},
        Argon2,
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Auth(format!("Hash error: {}", e)))?;
    Ok(hash.to_string())
}

/// Verify a password against a stored hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    use argon2::{
        password_hash::{PasswordHash, PasswordVerifier},
        Argon2,
    };

    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Auth(format!("Invalid hash: {}", e)))?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

/// Extract the user_id from the current session, or return an error
pub async fn require_auth() -> Result<String, ServerFnError> {
    let user = get_session_user().await?;
    match user {
        Some(u) => Ok(u.id),
        None => Err(ServerFnError::new("Not authenticated")),
    }
}

/// Create a session for the given user_id (store in tower-sessions)
pub async fn create_session(user_id: &str) -> Result<(), ServerFnError> {
    use crate::error::internal_error;
    use leptos_axum::extract;
    use tower_sessions::Session;

    let session: Session = extract().await?;
    session.insert("user_id", user_id).await
        .map_err(|e| internal_error("Session insert failed", e))?;
    Ok(())
}

/// Destroy the current session
pub async fn destroy_session() -> Result<(), ServerFnError> {
    use crate::error::internal_error;
    use leptos_axum::extract;
    use tower_sessions::Session;

    let session: Session = extract().await?;
    session.flush().await
        .map_err(|e| internal_error("Session flush failed", e))?;
    Ok(())
}

/// Get the current user from the session
pub async fn get_session_user() -> Result<Option<UserInfo>, ServerFnError> {
    use crate::error::internal_error;
    use leptos_axum::extract;
    use tower_sessions::Session;
    use crate::db::db;
    use crate::server_fns::auth::UserDbRow;

    let session: Session = extract().await?;
    let user_id: Option<String> = session.get("user_id").await
        .map_err(|e| internal_error("Session read failed", e))?;

    let Some(uid) = user_id else {
        return Ok(None);
    };

    // Parse the stored "table:key" string back to a RecordId for the query
    let record_id = surrealdb::types::RecordId::parse_simple(&uid)
        .map_err(|e| internal_error("User ID parse failed", e))?;

    let mut response = db()
        .query("SELECT id, username, email FROM user WHERE id = $id LIMIT 1")
        .bind(("id", record_id))
        .await
        .map_err(|e| internal_error("Session user query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Session user query error", err_msg));
    }

    let row: Option<UserDbRow> = response.take(0)
        .map_err(|e| internal_error("Session user parse failed", e))?;

    Ok(row.map(|r| r.into_user_info()))
}
