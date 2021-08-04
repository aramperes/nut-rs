use crate::proto::ClientSentences;
use core::fmt;
use std::io;

/// A NUT-native error.
#[derive(Debug)]
pub enum NutError {
    /// Occurs when the username/password combination is rejected.
    AccessDenied,
    /// Occurs when the specified UPS device does not exist.
    UnknownUps,
    /// The specified UPS doesn't support the variable in the request.
    VarNotSupported,
    /// The specified UPS doesn't support the instant command in the request.
    CmdNotSupported,
    /// The client sent an argument to a command which is not recognized or is otherwise invalid in this context.
    InvalidArgument,
    /// Server failed to deliver the instant command request to the driver. No further information is available to the client.
    InstCmdFailed,
    /// Server failed to deliver the set request to the driver.
    SetFailed,
    /// The requested variable in a SET command is not writable.
    ReadOnly,
    /// The requested value in a SET command is too long.
    TooLong,
    /// The server does not support the requested feature.
    FeatureNotSupported,
    /// TLS/SSL mode is already enabled on this connection, so the server can't start it again.
    AlreadySslMode,
    /// The server can't perform the requested command, since the driver for that UPS is not connected.
    DriverNotConnected,
    /// Server is connected to the driver for the UPS, but that driver isn't providing regular updates
    /// or has specifically marked the data as stale.
    DataStale,
    /// The client already sent LOGIN for a UPS and can't do it again.
    /// There is presently a limit of one LOGIN record per connection.
    AlreadyLoggedIn,
    /// The client sent an invalid PASSWORD - perhaps an empty one.
    InvalidPassword,
    /// The client already set a PASSWORD and can't set another.
    AlreadySetPassword,
    /// The client sent an invalid USERNAME.
    InvalidUsername,
    /// The client has already set a USERNAME, and can't set another.
    AlreadySetUsername,
    /// The requested command requires a username for authentication, but the client hasn't set one.
    UsernameRequired,
    /// The requested command requires a password for authentication, but the client hasn't set one.
    PasswordRequired,
    /// The server doesn't recognize the requested command.
    UnknownCommand,
    /// The value specified in the request is not valid.
    InvalidValue,
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
    /// The client or server sent a message that could not be processed.
    NotProcessable,
    /// Generic (usually internal) client error.
    Generic(String),
}

impl fmt::Display for NutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AccessDenied => write!(f, "Access denied"),
            Self::UnknownUps => write!(f, "Unknown UPS device"),
            Self::VarNotSupported => write!(f, "Variable not supported"),
            Self::CmdNotSupported => write!(f, "Command not supported"),
            Self::InvalidArgument => write!(f, "Invalid argument"),
            Self::InstCmdFailed => write!(f, "Instant command failed"),
            Self::SetFailed => write!(f, "Failed to set variable"),
            Self::ReadOnly => write!(f, "Cannot set read-only variable"),
            Self::TooLong => write!(f, "Value is too long"),
            Self::FeatureNotSupported => write!(f, "Feature is not supported by server"),
            Self::AlreadySslMode => write!(f, "Connection is already in TLS/SSL"),
            Self::DriverNotConnected => write!(f, "Driver is not connected"),
            Self::DataStale => write!(f, "Data is stale"),
            Self::AlreadyLoggedIn => write!(f, "Connection is already authenticated"),
            Self::InvalidPassword => write!(f, "Invalid password"),
            Self::AlreadySetPassword => write!(f, "Password can only be set once"),
            Self::InvalidUsername => write!(f, "Invalid username"),
            Self::AlreadySetUsername => write!(f, "Username can only be set once"),
            Self::UsernameRequired => write!(f, "Username required"),
            Self::PasswordRequired => write!(f, "Password required"),
            Self::UnknownCommand => write!(f, "Unknown command"),
            Self::InvalidValue => write!(f, "Invalid value"),
            Self::UnexpectedResponse => write!(f, "Unexpected server response content"),
            Self::UnknownResponseType(ty) => write!(f, "Unknown response type: {}", ty),
            Self::SslNotSupported => write!(f, "SSL not supported by server or transport"),
            Self::SslInvalidHostname => write!(
                f,
                "Given hostname cannot be used for a strict SSL connection"
            ),
            Self::FeatureNotConfigured => write!(f, "Feature not configured by server"),
            Self::NotProcessable => write!(f, "Message could not be processed"),
            Self::Generic(msg) => write!(f, "NUT error: {}", msg),
        }
    }
}

impl From<ClientSentences> for NutError {
    fn from(sentence: ClientSentences) -> Self {
        if let ClientSentences::RespondErr { message, .. } = sentence {
            match message.as_str() {
                "ACCESS-DENIED" => Self::AccessDenied,
                "UNKNOWN-UPS" => Self::UnknownUps,
                "VAR-NOT-SUPPORTED" => Self::VarNotSupported,
                "CMD-NOT-SUPPORTED" => Self::CmdNotSupported,
                "INVALID-ARGUMENT" => Self::InvalidArgument,
                "INSTCMD-FAILED" => Self::InstCmdFailed,
                "SET-FAILED" => Self::SetFailed,
                "READONLY" => Self::ReadOnly,
                "TOO-LONG" => Self::TooLong,
                "FEATURE-NOT-SUPPORTED" => Self::FeatureNotSupported,
                "FEATURE-NOT-CONFIGURED" => Self::FeatureNotConfigured,
                "ALREADY-SSL-MODE" => Self::AlreadySslMode,
                "DRIVER-NOT-CONNECTED" => Self::DriverNotConnected,
                "DATA-STALE" => Self::DataStale,
                "ALREADY-LOGGED-IN" => Self::AlreadyLoggedIn,
                "INVALID-PASSWORD" => Self::InvalidPassword,
                "ALREADY-SET-PASSWORD" => Self::AlreadySetPassword,
                "INVALID-USERNAME" => Self::InvalidUsername,
                "ALREADY-SET-USERNAME" => Self::AlreadySetUsername,
                "USERNAME-REQUIRED" => Self::UsernameRequired,
                "PASSWORD-REQUIRED" => Self::PasswordRequired,
                "UNKNOWN-COMMAND" => Self::UnknownCommand,
                "INVALID-VALUE" => Self::InvalidValue,
                _ => Self::Generic(message.to_string()),
            }
        } else {
            // This is not supposed to happen...
            panic!("Cannot convert {:?} into NutError", sentence);
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

/// Encapsulation for errors emitted by the `rups` library.
#[derive(Debug)]
pub enum Error {
    /// Encapsulates IO errors.
    Io(io::Error),
    /// Encapsulates NUT errors.
    Nut(NutError),
}

impl Error {
    /// Constructs a generic rups error.
    pub fn generic<T: ToString>(message: T) -> Self {
        NutError::generic(message.to_string()).into()
    }

    /// Constructs an EOF (end-of-file) error.
    pub fn eof() -> Self {
        Self::Io(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Reached end of stream while sentence was expected",
        ))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => err.fmt(f),
            Self::Nut(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<NutError> for Error {
    fn from(err: NutError) -> Self {
        Error::Nut(err)
    }
}

/// Result type for [`Error`]
pub type Result<T> = std::result::Result<T, Error>;
