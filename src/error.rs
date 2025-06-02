#[derive(Debug)]
pub enum AzuriteStorageError {
    ContainerNotFound(String),
    BlobNotFound(String),
    Unauthorized,
    InternalError(String),
    RuntimeCreationFailed,
    InvalidParameter(String),
}

impl std::fmt::Display for AzuriteStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AzuriteStorageError::ContainerNotFound(name) => {
                write!(f, "Container '{}' not found", name)
            }
            AzuriteStorageError::BlobNotFound(name) => write!(f, "Blob '{}' not found", name),
            AzuriteStorageError::Unauthorized => write!(f, "Unauthorized access"),
            AzuriteStorageError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            AzuriteStorageError::RuntimeCreationFailed => {
                write!(f, "Failed to create Tokio runtime")
            }
            AzuriteStorageError::InvalidParameter(param) => {
                write!(f, "Invalid parameter: {}", param)
            }
        }
    }
}

impl std::error::Error for AzuriteStorageError {}
