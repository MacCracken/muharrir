//! Error types for muharrir.

/// Errors that can occur in muharrir operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Expression evaluation failed.
    #[cfg(feature = "expr")]
    #[error("expression error: {0}")]
    Expr(String),

    /// History operation failed.
    #[cfg(feature = "history")]
    #[error("history error: {0}")]
    History(String),

    /// Command execution failed.
    #[cfg(feature = "command")]
    #[error("command error: {0}")]
    Command(String),

    /// Serialization/deserialization failed.
    #[error("serialization error: {0}")]
    Serde(String),
}

/// Result type alias for muharrir.
pub type Result<T> = std::result::Result<T, Error>;
