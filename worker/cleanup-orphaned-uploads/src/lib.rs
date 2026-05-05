pub type CleanupResult<T> = Result<T, CleanupError>;

#[derive(Debug, thiserror::Error)]
pub enum CleanupError {
    #[error("DateTime error: {0}")]
    DateTime(#[from] aws_smithy_types_convert::date_time::Error),
    #[error("SDK error: {0}")]
    Sdk(String),
}
