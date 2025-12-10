use thiserror::Error;

#[derive(Debug, Error)]
pub enum LuCastraError {
    #[error("device not found: {0}")]
    DeviceNotFound(String),

    #[error("device io error: {0}")]
    DeviceIoError(String),

    #[error("filesystem error: {0}")]
    FilesystemError(String),

    #[error("input error: {0}")]
    InputError(String),

    #[error("invalid command: {0}")]
    InvalidCommand(String),

    #[error("service error: {0}")]
    ServiceError(String),

    #[error("config error: {0}")]
    ConfigError(String),

    #[error("compat/syscall error: {0}")]
    SyscallError(String),
}

pub type Result<T> = std::result::Result<T, LuCastraError>;
