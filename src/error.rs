use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::ConfigError),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),

    #[error("HTTP request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Semaphore acquisition error: {0}")]
    SemaphoreError(#[from] tokio::sync::AcquireError),

    #[error("Browser automation error: {0}")]
    BrowserError(String),

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Metadata lookup failed: {0}")]
    MetadataError(String),

    #[error("Parsing error: {0}")]
    ParseError(String),
}

impl From<std::num::ParseIntError> for AppError {
    fn from(err: std::num::ParseIntError) -> Self {
        AppError::ParseError(err.to_string())
    }
}

impl From<tokio::task::JoinError> for AppError {
    fn from(err: tokio::task::JoinError) -> Self {
        AppError::MetadataError(format!("Task join error: {}", err))
    }
}