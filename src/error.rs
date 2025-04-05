use std::error::Error as StdError;
use std::{fmt, io};

#[derive(Debug)]
pub enum TaskMasterError {
    TaskNotFound(u32),
    ProjectNotFound(u32),
    InvalidOperation(String),
    StorageError(String),
    IoError(io::Error),
    SerializationError(String),
    ChannelError(String),
}

impl fmt::Display for TaskMasterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskMasterError::TaskNotFound(id) => write!(f, "Task with ID {} not found", id),
            TaskMasterError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            TaskMasterError::ProjectNotFound(id) => write!(f, "Project with ID {} not found", id),
            TaskMasterError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            TaskMasterError::IoError(err) => write!(f, "I/O error: {}", err),
            TaskMasterError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            TaskMasterError::ChannelError(msg) => write!(f, "Channel error: {}", msg),
        }
    }
}

impl StdError for TaskMasterError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            TaskMasterError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for TaskMasterError {
    fn from(err: io::Error) -> TaskMasterError {
        TaskMasterError::IoError(err)
    }
}

impl From<serde_json::Error> for TaskMasterError {
    fn from(err: serde_json::Error) -> Self {
        TaskMasterError::SerializationError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, TaskMasterError>;
