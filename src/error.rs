use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuroraError {
    #[error("DBus error: {0}")]
    DBus(String),

    #[error("DConf error: {0}")]
    DConf(String),

    #[error("Property not found: {0}")]
    PropertyNotFound(String),

    #[error("Invalid value type: expected {expected}, got {actual}")]
    InvalidType { expected: String, actual: String },

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

pub type Result<T> = std::result::Result<T, AuroraError>;
