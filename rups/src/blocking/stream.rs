use std::io::{Read, Write};
use std::net::TcpStream;

/// A wrapper for various synchronous stream types.
pub enum ConnectionStream {
    /// A plain TCP stream.
    Plain(TcpStream),

    /// A client stream wrapped with SSL using `rustls`.
    #[cfg(feature = "ssl")]
    SslClient(Box<rustls::StreamOwned<rustls::ClientSession, ConnectionStream>>),

    /// A server stream wrapped with SSL using `rustls`.
    #[cfg(feature = "ssl")]
    SslServer(Box<rustls::StreamOwned<rustls::ServerSession, ConnectionStream>>),
}

impl ConnectionStream {
    /// Wraps the current stream with SSL using `rustls` (client-side).
    #[cfg(feature = "ssl")]
    pub fn upgrade_ssl_client(
        self,
        session: rustls::ClientSession,
    ) -> crate::Result<ConnectionStream> {
        Ok(ConnectionStream::SslClient(Box::new(
            rustls::StreamOwned::new(session, self),
        )))
    }

    /// Wraps the current stream with SSL using `rustls` (client-side).
    #[cfg(feature = "ssl")]
    pub fn upgrade_ssl_server(
        self,
        session: rustls::ServerSession,
    ) -> crate::Result<ConnectionStream> {
        Ok(ConnectionStream::SslServer(Box::new(
            rustls::StreamOwned::new(session, self),
        )))
    }
}

impl Read for ConnectionStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.read(buf),
            #[cfg(feature = "ssl")]
            Self::SslClient(stream) => stream.read(buf),
            #[cfg(feature = "ssl")]
            Self::SslServer(stream) => stream.read(buf),
        }
    }
}

impl Write for ConnectionStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.write(buf),
            #[cfg(feature = "ssl")]
            Self::SslClient(stream) => stream.write(buf),
            #[cfg(feature = "ssl")]
            Self::SslServer(stream) => stream.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Plain(stream) => stream.flush(),
            #[cfg(feature = "ssl")]
            Self::SslClient(stream) => stream.flush(),
            #[cfg(feature = "ssl")]
            Self::SslServer(stream) => stream.flush(),
        }
    }
}
