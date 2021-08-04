use crate::blocking::stream::ConnectionStream;
use crate::Config;

/// A synchronous NUT client.
pub struct Client {
    /// The client configuration.
    config: Config,
    /// The client connection.
    stream: ConnectionStream,
}

impl Client {}
