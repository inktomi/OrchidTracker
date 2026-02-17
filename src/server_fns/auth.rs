use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use surrealdb::types::SurrealValue;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(surrealdb::types::SurrealValue))]
#[cfg_attr(feature = "ssr", surreal(crate = "surrealdb::types"))]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
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

    let result: Option<UserInfo> = db()
        .query("CREATE user SET username = $username, email = $email, password_hash = $hash RETURN id, username, email")
        .bind(("username", username))
        .bind(("email", email))
        .bind(("hash", password_hash))
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?
        .take(0)
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    let user = result.ok_or_else(|| ServerFnError::new("Failed to create user"))?;

    create_session(&user.id).await?;

    Ok(user)
}

#[server]
pub async fn login(username: String, password: String) -> Result<UserInfo, ServerFnError> {
    use crate::auth::verify_password;
    use crate::db::db;
    use surrealdb::types::SurrealValue;

    if username.is_empty() || username.len() > 50 {
        return Err(ServerFnError::new("Invalid credentials"));
    }
    if password.is_empty() || password.len() > 128 {
        return Err(ServerFnError::new("Invalid credentials"));
    }

    #[derive(Deserialize, surrealdb::types::SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct UserRow {
        id: String,
        username: String,
        email: String,
        password_hash: String,
    }

    let result: Option<UserRow> = db()
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

    crate::auth::create_session(&user_row.id).await?;

    Ok(UserInfo {
        id: user_row.id,
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
