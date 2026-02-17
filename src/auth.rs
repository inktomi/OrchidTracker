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
    use leptos_axum::extract;
    use tower_sessions::Session;

    let session: Session = extract().await?;
    session.insert("user_id", user_id).await
        .map_err(|e| ServerFnError::new(format!("Session error: {}", e)))?;
    Ok(())
}

/// Destroy the current session
pub async fn destroy_session() -> Result<(), ServerFnError> {
    use leptos_axum::extract;
    use tower_sessions::Session;

    let session: Session = extract().await?;
    session.flush().await
        .map_err(|e| ServerFnError::new(format!("Session error: {}", e)))?;
    Ok(())
}

/// Get the current user from the session
pub async fn get_session_user() -> Result<Option<UserInfo>, ServerFnError> {
    use leptos_axum::extract;
    use tower_sessions::Session;
    use crate::db::db;

    let session: Session = extract().await?;
    let user_id: Option<String> = session.get("user_id").await
        .map_err(|e| ServerFnError::new(format!("Session error: {}", e)))?;

    let Some(uid) = user_id else {
        return Ok(None);
    };

    let user: Option<UserInfo> = db()
        .query("SELECT id, username, email FROM user WHERE id = $id LIMIT 1")
        .bind(("id", uid))
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?
        .take(0)
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    Ok(user)
}
