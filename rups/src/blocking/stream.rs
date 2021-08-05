use crate::proto::util::{join_sentence, split_sentence};
use crate::proto::Sentence;
use crate::{Error, NutError};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;

/// A wrapper for various synchronous stream types.
pub enum ConnectionStream {
    /// A plain TCP stream.
    Tcp(TcpStream),

    /// A stream wrapped with `BufReader`.
    ///
    /// Use `.buffered()` to wrap any stream with `BufReader`.
    /// It can then be un-wrapped with `.unbuffered()`.
    Buffered(Box<BufReader<ConnectionStream>>),

    /// A client stream wrapped with SSL using `rustls`.
    #[cfg(feature = "ssl")]
    SslClient(Box<rustls::StreamOwned<rustls::ClientSession, ConnectionStream>>),

    /// A server stream wrapped with SSL using `rustls`.
    #[cfg(feature = "ssl")]
    SslServer(Box<rustls::StreamOwned<rustls::ServerSession, ConnectionStream>>),

    /// A mock stream, used for testing.
    #[cfg(test)]
    Mock(mockstream::SharedMockStream),
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

    /// Writes a sentence on the current stream.
    pub fn write_sentence<T: Sentence>(&mut self, sentence: &T) -> crate::Result<()> {
        let encoded = sentence.encode();
        let joined = join_sentence(encoded);
        self.write_literal(&joined)?;
        self.flush().map_err(crate::Error::Io)
    }

    /// Writes a collection of sentences on the current stream.
    pub fn write_sentences<T: Sentence>(&mut self, sentences: &[T]) -> crate::Result<()> {
        for sentence in sentences {
            let encoded = sentence.encode();
            let joined = join_sentence(encoded);
            self.write_literal(&joined)?;
        }
        self.flush().map_err(crate::Error::Io)
    }

    /// Writes a literal string to the current stream.
    /// Note: the literal string MUST end with a new-line character (`\n`).
    ///
    /// Note: does not automatically flush.
    pub fn write_literal(&mut self, literal: &str) -> crate::Result<()> {
        assert!(literal.ends_with('\n'));
        self.write_all(literal.as_bytes())?;
        Ok(())
    }

    /// Reads a literal string from the current stream.
    ///
    /// Note: the literal string will ends with a new-line character (`\n`)
    /// Note: requires stream to be buffered.
    pub fn read_literal(&mut self) -> crate::Result<String> {
        let mut raw = String::new();
        self.read_line(&mut raw)?;
        Ok(raw)
    }

    /// Reads a sentence from the current stream.
    ///
    /// Note: requires stream to be buffered.
    pub fn read_sentence<T: Sentence>(&mut self) -> crate::Result<T> {
        let raw = self.read_literal()?;
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
    pub fn read_sentences_until<T: Sentence, F: Fn(&T) -> bool>(
        &mut self,
        matcher: F,
    ) -> crate::Result<Vec<T>> {
        let mut result = Vec::new();
        let max_iter = 1000; // Exit after 1000 lines to prevent overflow.
        for _ in 0..max_iter {
            let sentence: T = self.read_sentence()?;
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

impl Read for ConnectionStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Tcp(stream) => stream.read(buf),
            Self::Buffered(reader) => reader.read(buf),
            #[cfg(feature = "ssl")]
            Self::SslClient(stream) => stream.read(buf),
            #[cfg(feature = "ssl")]
            Self::SslServer(stream) => stream.read(buf),
            #[cfg(test)]
            Self::Mock(stream) => stream.read(buf),
        }
    }
}

impl BufRead for ConnectionStream {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        match self {
            Self::Buffered(reader) => reader.fill_buf(),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Stream is not buffered",
            )),
        }
    }

    fn consume(&mut self, amt: usize) {
        if let Self::Buffered(reader) = self {
            reader.consume(amt)
        }
    }
}

impl Write for ConnectionStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Tcp(stream) => stream.write(buf),
            Self::Buffered(reader) => reader.get_mut().write(buf),
            #[cfg(feature = "ssl")]
            Self::SslClient(stream) => stream.write(buf),
            #[cfg(feature = "ssl")]
            Self::SslServer(stream) => stream.write(buf),
            #[cfg(test)]
            Self::Mock(stream) => {
                let len = buf.len();
                stream.push_bytes_to_read(buf);
                Ok(len)
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Tcp(stream) => stream.flush(),
            Self::Buffered(reader) => reader.get_mut().flush(),
            #[cfg(feature = "ssl")]
            Self::SslClient(stream) => stream.flush(),
            #[cfg(feature = "ssl")]
            Self::SslServer(stream) => stream.flush(),
            #[cfg(test)]
            Self::Mock(stream) => stream.flush(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ConnectionStream;
    use crate::proto::{ClientSentences, Sentence, ServerSentences};

    #[test]
    fn read_write_sentence() {
        let client_stream = mockstream::SharedMockStream::new();
        let server_stream = client_stream.clone();

        let mut client_stream = ConnectionStream::Mock(client_stream).buffered();
        let mut server_stream = ConnectionStream::Mock(server_stream).buffered();

        // Client requests list of UPS devices
        client_stream
            .write_sentence(&ServerSentences::QueryListUps {})
            .expect("Failed to write LIST UPS");

        // Server reads query for list of UPS devices
        let sentence = server_stream
            .read_sentence::<ServerSentences>()
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
            .expect("Failed to write UPS LIST");

        // Client reads list of UPS devices.
        client_stream
            .read_sentence::<ClientSentences>()
            .expect("Failed to read BEGIN LIST UPS")
            .exactly(ClientSentences::BeginListUps {})
            .unwrap();

        let sentences: Vec<ClientSentences> = client_stream
            .read_sentences_until(|s| matches!(s, ClientSentences::EndListUps {}))
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
            .expect("Failed to write LOGIN nutdev0");

        // Server receives login
        server_stream
            .read_sentence::<ServerSentences>()
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
            .expect("Failed to write ERR USERNAME-REQUIRED");

        // Client expects error
        let error: crate::Error = client_stream
            .read_sentence::<ClientSentences>()
            .expect_err("Failed to read ERR");
        assert!(matches!(
            error,
            crate::Error::Nut(crate::NutError::UsernameRequired)
        ));
    }
}
