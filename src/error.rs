use std::fmt;

/// What is it? The unified error enumeration for the application backend and data layers.
/// Why does it exist? It provides a single type to propagate failures (like network, database, or validation errors) up the call stack, simplifying error handling.
/// How should it be used? Return it as the `Err` variant in `Result<T, AppError>` for all internal operations, and match against its variants when logging or translating to HTTP responses.
#[derive(Debug)]
pub enum AppError {
    /// Errors related to user authentication and authorization.
    Auth(String),
    /// Errors originating from database operations.
    Database(String),
    /// Errors caused by network requests or connectivity issues.
    Network(String),
    /// Errors occurring during serialization or deserialization of data.
    Serialization(String),
    /// Errors due to invalid user input or data constraints.
    Validation(String),
    /// Errors encountered while saving or retrieving images.
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

/// What is it? A utility function for converting an internal error into a safe, user-facing error.
/// Why does it exist? It ensures sensitive backend details (like database query syntax or stack traces) are logged but never leaked to the client browser.
/// How should it be used? Call it within Leptos server functions (`#[server]`) when an `AppError` is encountered, returning its safe `ServerFnError` output to the frontend.
#[cfg(feature = "ssr")]
pub fn internal_error(
    context: &str,
    err: impl std::fmt::Display,
) -> leptos::prelude::ServerFnError {
    tracing::error!("{context}: {err}");
    leptos::prelude::ServerFnError::new("An internal error occurred. Please try again later.")
}
