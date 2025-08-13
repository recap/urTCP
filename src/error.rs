use thiserror::Error;

#[derive(Error, Debug)]
pub enum UtcpError {
    #[error("device error: {0}")]
    Device(String),
    #[error("malformed packet")]
    Malformed,
    #[error("checksum mismatch")]
    Checksum,
    #[error("connection not found")]
    ConnNotFound,
    #[error("would block")]
    WouldBlock,
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
}
pub type Result<T> = std::result::Result<T, UtcpError>;
