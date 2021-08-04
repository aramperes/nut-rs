use crate::Error;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;

/// A wrapper for various Tokio stream types.
pub enum ConnectionStream {
    /// A plain TCP stream.
    Plain(TcpStream),

    /// A client stream wrapped with SSL using `rustls`.
    #[cfg(feature = "async-ssl")]
    SslClient(Box<tokio_rustls::client::TlsStream<ConnectionStream>>),

    /// A server stream wrapped with SSL using `rustls`.
    #[cfg(feature = "async-ssl")]
    SslServer(Box<tokio_rustls::server::TlsStream<ConnectionStream>>),
}

impl ConnectionStream {
    /// Wraps the current stream with SSL using `rustls`.
    #[cfg(feature = "async-ssl")]
    pub async fn upgrade_ssl_client(
        self,
        config: tokio_rustls::TlsConnector,
        dns_name: webpki::DNSNameRef<'_>,
    ) -> crate::Result<ConnectionStream> {
        Ok(ConnectionStream::SslClient(Box::new(
            config.connect(dns_name, self).await.map_err(Error::Io)?,
        )))
    }

    /// Wraps the current stream with SSL using `rustls`.
    #[cfg(feature = "async-ssl")]
    pub async fn upgrade_ssl_server(
        self,
        acceptor: tokio_rustls::TlsAcceptor,
    ) -> crate::Result<ConnectionStream> {
        Ok(ConnectionStream::SslServer(Box::new(
            acceptor.accept(self).await.map_err(Error::Io)?,
        )))
    }
}

impl AsyncRead for ConnectionStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Plain(stream) => {
                let pinned = Pin::new(stream);
                pinned.poll_read(cx, buf)
            }
            #[cfg(feature = "async-ssl")]
            Self::SslClient(stream) => {
                let pinned = Pin::new(stream);
                pinned.poll_read(cx, buf)
            }
            #[cfg(feature = "async-ssl")]
            Self::SslServer(stream) => {
                let pinned = Pin::new(stream);
                pinned.poll_read(cx, buf)
            }
        }
    }
}

impl AsyncWrite for ConnectionStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.get_mut() {
            Self::Plain(stream) => {
                let pinned = Pin::new(stream);
                pinned.poll_write(cx, buf)
            }
            #[cfg(feature = "async-ssl")]
            Self::SslClient(stream) => {
                let pinned = Pin::new(stream);
                pinned.poll_write(cx, buf)
            }
            #[cfg(feature = "async-ssl")]
            Self::SslServer(stream) => {
                let pinned = Pin::new(stream);
                pinned.poll_write(cx, buf)
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Plain(stream) => {
                let pinned = Pin::new(stream);
                pinned.poll_flush(cx)
            }
            #[cfg(feature = "async-ssl")]
            Self::SslClient(stream) => {
                let pinned = Pin::new(stream);
                pinned.poll_flush(cx)
            }
            #[cfg(feature = "async-ssl")]
            Self::SslServer(stream) => {
                let pinned = Pin::new(stream);
                pinned.poll_flush(cx)
            }
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Plain(stream) => {
                let pinned = Pin::new(stream);
                pinned.poll_shutdown(cx)
            }
            #[cfg(feature = "async-ssl")]
            Self::SslClient(stream) => {
                let pinned = Pin::new(stream);
                pinned.poll_shutdown(cx)
            }
            #[cfg(feature = "async-ssl")]
            Self::SslServer(stream) => {
                let pinned = Pin::new(stream);
                pinned.poll_shutdown(cx)
            }
        }
    }
}
