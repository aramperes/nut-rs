use crate::blocking::stream::ConnectionStream;
use crate::proto::{ClientSentences, Sentence, ServerSentences};
use crate::{Config, Host, TcpHost};
use std::net::TcpStream;

/// A synchronous NUT client.
pub struct Client {
    /// The client configuration.
    config: Config,
    /// The client connection.
    stream: ConnectionStream,
}

impl Client {
    /// Connects to a remote NUT server using a blocking connection.
    pub fn new(config: &Config) -> crate::Result<Self> {
        match &config.host {
            Host::Tcp(host) => Self::new_tcp(config, host),
        }
        // TODO: Support Unix domain sockets
    }

    /// Connects to a remote NUT server using a blocking TCP connection.
    fn new_tcp(config: &Config, host: &TcpHost) -> crate::Result<Self> {
        let tcp_stream = TcpStream::connect_timeout(&host.addr, config.timeout)?;
        let mut client = Client {
            config: config.clone(),
            stream: ConnectionStream::Tcp(tcp_stream).buffered(),
        };

        client = client.enable_ssl()?;

        // TODO: Enable SSL
        // TODO: Login
        Ok(client)
    }

    #[cfg(feature = "ssl")]
    fn enable_ssl(mut self) -> crate::Result<Self> {
        if self.config.ssl {
            // Send STARTTLS
            self.stream
                .write_sentence(&ServerSentences::ExecStartTLS {})?;

            // Expect the OK
            self.stream
                .read_sentence::<ClientSentences>()?
                .as_exactly(ClientSentences::StartTLSOk {})?;

            // Un-buffer to get back underlying stream
            self.stream = self.stream.unbuffered();

            // TODO: Un-buffer
            // TODO: Do the upgrade
        }
        Ok(self)
    }

    #[cfg(not(feature = "ssl"))]
    fn enable_ssl(self) -> crate::Result<Self> {
        Ok(self)
    }
}
