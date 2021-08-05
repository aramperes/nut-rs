use crate::proto::util::{join_sentence, split_sentence};
use crate::proto::Sentence;
use crate::{Error, NutError};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{
    AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, ReadBuf,
};
use tokio::net::TcpStream;

/// A wrapper for various Tokio stream types.
#[derive(Debug)]
pub enum ConnectionStream {
    /// A plain TCP stream.
    Tcp(TcpStream),

    /// A stream wrapped with `BufReader`.
    ///
    /// Use `.buffered()` to wrap any stream with `BufReader`.
    /// It can then be un-wrapped with `.unbuffered()`.
    Buffered(Box<BufReader<ConnectionStream>>),

    /// A client stream wrapped with SSL using `rustls`.
    #[cfg(feature = "async-ssl")]
    SslClient(Box<tokio_rustls::client::TlsStream<ConnectionStream>>),

    /// A server stream wrapped with SSL using `rustls`.
    #[cfg(feature = "async-ssl")]
    SslServer(Box<tokio_rustls::server::TlsStream<ConnectionStream>>),

    /// A mock stream, used for testing.
    #[cfg(test)]
    Mock(crate::tokio::mockstream::AsyncMockStream),
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

    /// Writes a sentence on the current stream.
    pub async fn write_sentence<T: Sentence>(&mut self, sentence: &T) -> crate::Result<()> {
        let encoded = sentence.encode();
        let joined = join_sentence(encoded);
        self.write_literal(&joined).await?;
        self.flush().await.map_err(crate::Error::Io)
    }

    /// Writes a collection of sentences on the current stream.
    pub async fn write_sentences<T: Sentence>(&mut self, sentences: &[T]) -> crate::Result<()> {
        for sentence in sentences {
            let encoded = sentence.encode();
            let joined = join_sentence(encoded);
            self.write_literal(&joined).await?;
        }
        self.flush().await.map_err(crate::Error::Io)
    }

    /// Writes a literal string to the current stream.
    /// Note: the literal string MUST end with a new-line character (`\n`).
    ///
    /// Note: does not automatically flush.
    pub async fn write_literal(&mut self, literal: &str) -> crate::Result<()> {
        assert!(literal.ends_with('\n'));
        self.write_all(literal.as_bytes()).await?;
        Ok(())
    }

    /// Reads a literal string from the current stream.
    ///
    /// Note: the literal string will ends with a new-line character (`\n`)
    /// Note: requires stream to be buffered.
    pub async fn read_literal(&mut self) -> crate::Result<String> {
        let mut raw = String::new();
        self.read_line(&mut raw).await?;
        Ok(raw)
    }

    /// Reads a sentence from the current stream.
    ///
    /// Note: requires stream to be buffered.
    pub async fn read_sentence<T: Sentence>(&mut self) -> crate::Result<T> {
        dbg!(&self);
        let raw = self.read_literal().await?;
        if raw.is_empty() {
            return Err(Error::eof());
        }
        let split = split_sentence(raw).ok_or(crate::NutError::NotProcessable)?;
        T::decode(split)
            .ok_or(Error::Nut(NutError::InvalidArgument))?
            .into()
    }

    /// Reads all sentences in the stream until the given `matcher` function evaluates to `true`.
    ///
    /// The final sentence is excluded.
    ///
    /// Note: requires stream to be buffered.
    pub async fn read_sentences_until<T: Sentence, F: Fn(&T) -> bool>(
        &mut self,
        matcher: F,
    ) -> crate::Result<Vec<T>> {
        let mut result = Vec::new();
        let max_iter = 1000; // Exit after 1000 lines to prevent overflow.
        for _ in 0..max_iter {
            let sentence: T = self.read_sentence().await?;
            if matcher(&sentence) {
                return Ok(result);
            } else {
                result.push(sentence);
            }
        }
        Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::Interrupted,
            "Reached maximum read capacity.",
        )))
    }

    /// Wraps the current stream with a `BufReader`.
    pub fn buffered(self) -> ConnectionStream {
        Self::Buffered(Box::new(BufReader::new(self)))
    }

    /// Unwraps the underlying stream from the current `BufReader`.
    /// If the current stream is not buffered, it returns itself (no-op).
    ///
    /// Note that, if the stream is buffered, any un-consumed data will be discarded.
    pub fn unbuffered(self) -> ConnectionStream {
        if let Self::Buffered(buf) = self {
            buf.into_inner()
        } else {
            self
        }
    }
}

impl AsyncRead for ConnectionStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Tcp(stream) => {
                let pinned = Pin::new(stream);
                pinned.poll_read(cx, buf)
            }
            Self::Buffered(reader) => {
                let pinned = Pin::new(reader.get_mut());
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
            #[cfg(test)]
            Self::Mock(stream) => {
                let pinned = Pin::new(stream);
                pinned.poll_read(cx, buf)
            }
        }
    }
}

impl AsyncBufRead for ConnectionStream {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<&[u8]>> {
        dbg!(&self);
        match self.get_mut() {
            Self::Buffered(reader) => {
                let pinned = Pin::new(reader);
                pinned.poll_fill_buf(cx)
            }
            s => core::task::Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                format!("Stream is not buffered: {:?}", s),
            ))),
        }
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        if let Self::Buffered(reader) = self.get_mut() {
            let pinned = Pin::new(reader);
            pinned.consume(amt)
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
            Self::Tcp(stream) => {
                let pinned = Pin::new(stream);
                pinned.poll_write(cx, buf)
            }
            Self::Buffered(reader) => {
                let pinned = Pin::new(reader.get_mut());
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
            #[cfg(test)]
            Self::Mock(stream) => {
                let pinned = Pin::new(stream);
                pinned.poll_write(cx, buf)
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Tcp(stream) => {
                let pinned = Pin::new(stream);
                pinned.poll_flush(cx)
            }
            Self::Buffered(reader) => {
                let pinned = Pin::new(reader.get_mut());
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
            #[cfg(test)]
            Self::Mock(stream) => {
                let pinned = Pin::new(stream);
                pinned.poll_flush(cx)
            }
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Tcp(stream) => {
                let pinned = Pin::new(stream);
                pinned.poll_shutdown(cx)
            }
            Self::Buffered(reader) => {
                let pinned = Pin::new(reader.get_mut());
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
            #[cfg(test)]
            Self::Mock(stream) => {
                let pinned = Pin::new(stream);
                pinned.poll_shutdown(cx)
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::ConnectionStream;
    use crate::proto::{ClientSentences, Sentence, ServerSentences};

    #[tokio::test]
    async fn read_write_sentence() {
        let client_stream = crate::tokio::mockstream::AsyncMockStream::new();
        let server_stream = client_stream.clone();

        let mut client_stream = ConnectionStream::Mock(client_stream).buffered();
        let mut server_stream = ConnectionStream::Mock(server_stream).buffered();

        // Client requests list of UPS devices
        client_stream
            .write_sentence(&ServerSentences::QueryListUps {})
            .await
            .expect("Failed to write LIST UPS");

        dbg!(&client_stream);
        dbg!(&server_stream);

        // Server reads query for list of UPS devices
        let sentence = server_stream
            .read_sentence::<ServerSentences>()
            .await
            .expect("Failed to read LIST UPS");
        assert_eq!(sentence, ServerSentences::QueryListUps {});

        // Server sends list of UPS devices.
        server_stream
            .write_sentences(&[
                ClientSentences::BeginListUps {},
                ClientSentences::RespondUps {
                    ups_name: "nutdev0".into(),
                    description: "A NUT device.".into(),
                },
                ClientSentences::RespondUps {
                    ups_name: "nutdev1".into(),
                    description: "Another NUT device.".into(),
                },
                ClientSentences::EndListUps {},
            ])
            .await
            .expect("Failed to write UPS LIST");

        // Client reads list of UPS devices.
        client_stream
            .read_sentence::<ClientSentences>()
            .await
            .expect("Failed to read BEGIN LIST UPS")
            .exactly(ClientSentences::BeginListUps {})
            .unwrap();

        let sentences: Vec<ClientSentences> = client_stream
            .read_sentences_until(|s| matches!(s, ClientSentences::EndListUps {}))
            .await
            .expect("Failed to read UPS items");

        assert_eq!(
            sentences,
            vec![
                ClientSentences::RespondUps {
                    ups_name: "nutdev0".into(),
                    description: "A NUT device.".into(),
                },
                ClientSentences::RespondUps {
                    ups_name: "nutdev1".into(),
                    description: "Another NUT device.".into(),
                },
            ]
        );

        // Client sends login
        client_stream
            .write_sentence(&ServerSentences::ExecLogin {
                ups_name: "nutdev0".into(),
            })
            .await
            .expect("Failed to write LOGIN nutdev0");

        // Server receives login
        server_stream
            .read_sentence::<ServerSentences>()
            .await
            .expect("Failed to read LOGIN nutdev0")
            .exactly(ServerSentences::ExecLogin {
                ups_name: "nutdev0".into(),
            })
            .unwrap();

        // Server rejects login
        server_stream
            .write_sentence(&ClientSentences::RespondErr {
                message: "USERNAME-REQUIRED".into(),
                extras: vec![],
            })
            .await
            .expect("Failed to write ERR USERNAME-REQUIRED");

        // Client expects error
        let error: crate::Error = client_stream
            .read_sentence::<ClientSentences>()
            .await
            .expect_err("Failed to read ERR");
        assert!(matches!(
            error,
            crate::Error::Nut(crate::NutError::UsernameRequired)
        ));
    }
}
