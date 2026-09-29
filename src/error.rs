use std::io;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, SuZapretError>;

#[derive(Debug, Error)]
pub enum SuZapretError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("invalid strategy: {0}")]
    InvalidStrategy(String),
    #[error("failed to parse strategy: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("strategy is already running")]
    AlreadyRunning,
    #[error("strategy is not running")]
    NotRunning,
    #[error("command failed: {command} ({status}){details}")]
    CommandFailed {
        command: String,
        status: String,
        details: String,
    },
}
