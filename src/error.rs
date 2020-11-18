use core::fmt;
use std::io;

/// A NUT-native error.
#[derive(Debug)]
pub enum NutError {
    /// Occurs when the username/password combination is rejected.
    AccessDenied,
    UnexpectedResponse,
    UnknownResponseType(String),
    /// Generic (usually internal) client error.
    Generic(String),
}

impl fmt::Display for NutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AccessDenied => write!(f, "Authentication failed"),
            Self::UnexpectedResponse => write!(f, "Unexpected server response"),
            Self::UnknownResponseType(ty) => write!(f, "Unknown response type: {}", ty),
            Self::Generic(msg) => write!(f, "Internal client error: {}", msg),
        }
    }
}

impl std::error::Error for NutError {}

#[derive(Debug)]
pub enum ClientError {
    Io(io::Error),
    Nut(NutError),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => err.fmt(f),
            Self::Nut(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for ClientError {}

impl From<io::Error> for ClientError {
    fn from(err: io::Error) -> Self {
        ClientError::Io(err)
    }
}

impl From<NutError> for ClientError {
    fn from(err: NutError) -> Self {
        ClientError::Nut(err)
    }
}

pub type Result<T> = std::result::Result<T, ClientError>;
