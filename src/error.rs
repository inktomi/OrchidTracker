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
