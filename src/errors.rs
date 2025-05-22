use std::path::PathBuf;
use thiserror::Error;
use colored::*;

/// CI-specific error types
#[derive(Error, Debug)]
pub enum CIError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Git operation failed: {0}")]
    Git(String),
    
    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),
    
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    
    #[error("External command failed: {0}")]
    Command(String),
    
    #[error("Environment error: {0}")]
    Environment(String),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    
    #[error("Project operation failed: {0}")]
    Project(String),
    
    #[error("API key error: {0}")]
    ApiKey(String),
    
    #[error("Agent operation failed: {0}")]
    Agent(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("System error: {0}")]
    SystemError(String),
    
    // NEW: Topology errors
    #[error("Repository topology error: {0}")]
    TopologyError(String),
    
    #[error("Repository state validation failed: {0}")]
    RepositoryStateError(String),
    
    #[error("Metadata operation failed: {0}")]
    MetadataError(String),
    
    #[error("Git operation failed: {0}")]
    GitOperationError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Custom Result type for CI operations
pub type Result<T> = std::result::Result<T, CIError>;

/// Helper function for consistent error handling and formatting
pub fn handle_error(err: CIError) -> ! {
    eprintln!("{} {}", "Error:".red().bold(), err);
    std::process::exit(1);
}

/// Handle errors with a specific context
pub fn handle_error_with_context(err: CIError, context: &str) -> ! {
    eprintln!("{} {} - {}", "Error:".red().bold(), context, err);
    std::process::exit(1);
}

/// Convert string errors to CIError
pub fn to_config_error<E: std::fmt::Display>(err: E) -> CIError {
    CIError::Config(err.to_string())
}

/// Convert string errors to Git-specific CIError
pub fn to_git_error<E: std::fmt::Display>(err: E) -> CIError {
    CIError::Git(err.to_string())
}

/// Convert path errors to PathNotFound error
pub fn path_not_found(path: PathBuf) -> CIError {
    CIError::PathNotFound(path)
}

/// Convert string errors to Command error
pub fn to_command_error<E: std::fmt::Display>(err: E) -> CIError {
    CIError::Command(err.to_string())
}

/// Convert string errors to API key error
pub fn to_api_key_error<E: std::fmt::Display>(err: E) -> CIError {
    CIError::ApiKey(err.to_string())
}

/// Convert string errors to Agent error
pub fn to_agent_error<E: std::fmt::Display>(err: E) -> CIError {
    CIError::Agent(err.to_string())
}

/// Helper for mapping errors with additional context
pub trait ErrorExt<T> {
    /// Map an error with additional context information
    fn with_context<C, F>(self, context: F) -> Result<T>
    where
        C: std::fmt::Display,
        F: FnOnce() -> C;
}

impl<T, E> ErrorExt<T> for std::result::Result<T, E>
where
    E: std::fmt::Display,
{
    fn with_context<C, F>(self, context: F) -> Result<T>
    where
        C: std::fmt::Display,
        F: FnOnce() -> C,
    {
        self.map_err(|err| {
            CIError::Unknown(format!("{} - {}", context(), err))
        })
    }
}

/// Convert from anyhow::Error to CIError
impl From<anyhow::Error> for CIError {
    fn from(err: anyhow::Error) -> Self {
        // Try to downcast to specific error types if possible
        if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
            return CIError::IO(io_err.kind().into());
        }
        
        CIError::Unknown(err.to_string())
    }
}

// Convert CIError to anyhow::Error - commented out to avoid conflicts with anyhow's blanket implementation
// impl From<CIError> for anyhow::Error {
//     fn from(err: CIError) -> Self {
//         anyhow::anyhow!("{}", err)
//     }
// }