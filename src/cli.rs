use clap::{Parser, Subcommand};

use crate::auth::hash_password;
use crate::db::db;
use surrealdb::types::SurrealValue;

#[derive(Parser)]
#[command(name = "orchid-tracker", about = "OrchidTracker web server")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Reset a user's password
    ResetPassword {
        /// The username to reset
        #[arg(short, long)]
        username: String,
        /// The new password
        #[arg(short, long)]
        password: String,
    },
}

pub async fn run_reset_password(username: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    let hash = hash_password(password)?;

    let mut response = db()
        .query("UPDATE user SET password_hash = $hash WHERE username = $username")
        .bind(("hash", hash))
        .bind(("username", username.to_owned()))
        .await?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(format!("Database error: {}", err_msg).into());
    }

    // Check that a row was actually updated
    #[derive(serde::Deserialize, surrealdb::types::SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct UpdatedRow {
        #[allow(dead_code)]
        username: String,
    }

    let rows: Vec<UpdatedRow> = response.take(0)?;
    if rows.is_empty() {
        return Err(format!("No user found with username '{}'", username).into());
    }

    tracing::info!("Password reset successfully for user '{}'", username);
    Ok(())
}
