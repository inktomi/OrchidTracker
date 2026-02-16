use std::fmt;

#[derive(Debug)]
pub enum AppError {
    ConfigMissing,
    LfsUpload(String),
    GithubApi(String),
    Database(String),
    Network(String),
    Serialization(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ConfigMissing => write!(f, "GitHub configuration missing"),
            AppError::LfsUpload(msg) => write!(f, "LFS upload failed: {}", msg),
            AppError::GithubApi(msg) => write!(f, "GitHub API error: {}", msg),
            AppError::Database(msg) => write!(f, "Database error: {}", msg),
            AppError::Network(msg) => write!(f, "Network error: {}", msg),
            AppError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}
