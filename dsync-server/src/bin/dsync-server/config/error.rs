#[derive(Debug, thiserror::Error)]
pub enum ConfigLoadError {
    #[error("I/O error: {0}")]
    IoError(std::io::Error),

    #[error("Failed to parse config: {0}")]
    ParseError(String),

    #[error("Other error: {0}")]
    Other(String),
}
