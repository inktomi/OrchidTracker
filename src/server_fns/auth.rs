use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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

    if username.is_empty() || email.is_empty() || password.len() < 8 {
        return Err(ServerFnError::new("Invalid input: username/email required, password min 8 chars"));
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

    #[derive(Deserialize)]
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
