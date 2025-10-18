use std::num::{ParseFloatError, ParseIntError};

use thiserror::Error;
use tokio_tungstenite;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("Format error")]
    FormatError,
    #[error("Failed to parse integer: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("Failed to parse float: {0}")]
    ParseFloatError(#[from] ParseFloatError),
    #[error("Message data is not binary or text data")]
    NotBinaryOrText,
    #[error("Invalid feature ID {0}")]
    InvalidFeatureId(u16),
    #[error("Invalid feature name {0}")]
    InvalidFeatureName(String),
    #[error("Invalid message size: {0}")]
    InvalidDataSize(usize),
    #[error("Tungstenite error: {0}")]
    Tungstenite(#[from] tokio_tungstenite::tungstenite::Error)
}