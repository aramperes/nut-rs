use std::io::{Read, Write};
use std::net::TcpStream;

/// A wrapper for various synchronous stream types.
pub enum ConnectionStream {
    /// A plain TCP stream.
    Plain(TcpStream),

    /// A stream wrapped with SSL using `rustls`.
    #[cfg(feature = "ssl")]
    Ssl(Box<rustls::StreamOwned<rustls::ClientSession, ConnectionStream>>),
}

impl ConnectionStream {
    /// Wraps the current stream with SSL using `rustls`.
    #[cfg(feature = "ssl")]
    pub fn upgrade_ssl(self, session: rustls::ClientSession) -> crate::Result<ConnectionStream> {
        Ok(ConnectionStream::Ssl(Box::new(rustls::StreamOwned::new(
            session, self,
        ))))
    }
}

impl Read for ConnectionStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.read(buf),
            #[cfg(feature = "ssl")]
            Self::Ssl(stream) => stream.read(buf),
        }
    }
}

impl Write for ConnectionStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.write(buf),
            #[cfg(feature = "ssl")]
            Self::Ssl(stream) => stream.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Plain(stream) => stream.flush(),
            #[cfg(feature = "ssl")]
            Self::Ssl(stream) => stream.flush(),
        }
    }
}
