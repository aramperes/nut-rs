use core::fmt;
use std::io;

/// A NUT-native error.
#[derive(Debug)]
pub enum NutError {
    /// Occurs when the username/password combination is rejected.
    AccessDenied,
    /// Occurs when the specified UPS device does not exist.
    UnknownUps,
    /// Occurs when the response type or content wasn't expected at the current stage.
    UnexpectedResponse,
    /// Occurs when the response type is not recognized by the client.
    UnknownResponseType(String),
    /// Occurs when attempting to use SSL in a transport that doesn't support it, or
    /// if the server is not configured for it.
    SslNotSupported,
    /// Occurs when trying to initialize a strict SSL connection with an invalid hostname.
    SslInvalidHostname,
    /// Occurs when the client used a feature that is disabled by the server.
    FeatureNotConfigured,
    /// Generic (usually internal) client error.
    Generic(String),
}

impl fmt::Display for NutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AccessDenied => write!(f, "Authentication failed"),
            Self::UnknownUps => write!(f, "Unknown UPS device name"),
            Self::UnexpectedResponse => write!(f, "Unexpected server response content"),
            Self::UnknownResponseType(ty) => write!(f, "Unknown response type: {}", ty),
            Self::SslNotSupported => write!(f, "SSL not supported by server or transport"),
            Self::SslInvalidHostname => write!(
                f,
                "Given hostname cannot be used for a strict SSL connection"
            ),
            Self::FeatureNotConfigured => write!(f, "Feature not configured by server"),
            Self::Generic(msg) => write!(f, "Client error: {}", msg),
        }
    }
}

impl NutError {
    /// Constructs a generic rups error.
    pub fn generic<T: ToString>(message: T) -> Self {
        Self::Generic(message.to_string())
    }
}

impl std::error::Error for NutError {}

/// Encapsulation for errors emitted by the client library.
#[derive(Debug)]
pub enum ClientError {
    /// Encapsulates IO errors.
    Io(io::Error),
    /// Encapsulates NUT and client-specific errors.
    Nut(NutError),
}

impl ClientError {
    /// Constructs a generic rups error.
    pub fn generic<T: ToString>(message: T) -> Self {
        NutError::generic(message.to_string()).into()
    }
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

/// Result type for [`ClientError`]
pub type Result<T> = std::result::Result<T, ClientError>;
