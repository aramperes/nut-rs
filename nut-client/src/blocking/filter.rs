use std::io::{Read, Write};
use std::net::TcpStream;

#[allow(clippy::large_enum_variant)]
pub enum ConnectionPipeline {
    Tcp(TcpStream),

    #[cfg(feature = "ssl")]
    Ssl(rustls::StreamOwned<rustls::ClientSession, TcpStream>),
}

impl ConnectionPipeline {
    pub fn tcp(&self) -> Option<TcpStream> {
        match self {
            Self::Tcp(stream) => Some(stream.try_clone().ok()).flatten(),
            _ => None,
        }
    }
}

impl Read for ConnectionPipeline {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Tcp(stream) => stream.read(buf),
            #[cfg(feature = "ssl")]
            Self::Ssl(stream) => stream.read(buf),
        }
    }
}

impl Write for ConnectionPipeline {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Tcp(stream) => stream.write(buf),
            #[cfg(feature = "ssl")]
            Self::Ssl(stream) => stream.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Tcp(stream) => stream.flush(),
            #[cfg(feature = "ssl")]
            Self::Ssl(stream) => stream.flush(),
        }
    }
}
