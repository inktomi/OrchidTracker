use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Auth(String),
    Database(String),
    Network(String),
    Serialization(String),
    Validation(String),
    ImageStorage(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Auth(msg) => write!(f, "Authentication error: {}", msg),
            AppError::Database(msg) => write!(f, "Database error: {}", msg),
            AppError::Network(msg) => write!(f, "Network error: {}", msg),
            AppError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            AppError::Validation(msg) => write!(f, "Validation error: {}", msg),
            AppError::ImageStorage(msg) => write!(f, "Image storage error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

/// Log an internal error and return a generic ServerFnError safe for the UI.
/// The real error details go to the server logs only.
#[cfg(feature = "ssr")]
pub fn internal_error(context: &str, err: impl std::fmt::Display) -> leptos::prelude::ServerFnError {
    tracing::error!("{context}: {err}");
    leptos::prelude::ServerFnError::new("An internal error occurred. Please try again later.")
}
