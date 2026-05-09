/// Error types for sandbox-file.

#[derive(Debug, thiserror::Error)]
pub enum SandboxFileError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("S3 error: {0}")]
    S3(String),

    #[error("tool not found: {0}")]
    ToolNotFound(String),

    #[error("tool execution error: {0}")]
    ToolExec(String),

    #[error("tool timeout after {0}s")]
    Timeout(u64),

    #[error("YARA error: {0}")]
    Yara(String),

    #[error("invalid payload: {0}")]
    InvalidPayload(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<SandboxFileError> for tonic::Status {
    fn from(e: SandboxFileError) -> Self {
        match e {
            SandboxFileError::Db(e) => tonic::Status::internal(format!("db: {}", e)),
            SandboxFileError::S3(e) => tonic::Status::internal(format!("s3: {}", e)),
            SandboxFileError::ToolNotFound(t) => tonic::Status::internal(format!("tool not found: {}", t)),
            SandboxFileError::ToolExec(e) => tonic::Status::internal(format!("tool: {}", e)),
            SandboxFileError::Timeout(s) => tonic::Status::deadline_exceeded(format!("timeout: {}s", s)),
            SandboxFileError::Yara(e) => tonic::Status::internal(format!("yara: {}", e)),
            SandboxFileError::InvalidPayload(e) => tonic::Status::invalid_argument(e),
            SandboxFileError::Io(e) => tonic::Status::internal(format!("io: {}", e)),
        }
    }
}
