use std::net::SocketAddr;

use crate::cmd::{Command, Response};
use crate::tokio::stream::ConnectionStream;
use crate::{Config, Error, Host, NutError};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

mod client;
#[cfg(test)]
mod mockstream;
mod stream;

pub use client::Client;

// TODO: Remove me
/// An async NUT client connection.
pub enum Connection {
    /// A TCP connection.
    Tcp(TcpConnection),
}

impl Connection {
    /// Initializes a connection to a NUT server (upsd).
    pub async fn new(config: &Config) -> crate::Result<Self> {
        let mut conn = match &config.host {
            Host::Tcp(host) => Self::Tcp(TcpConnection::new(config.clone(), &host.addr).await?),
        };

        conn.get_network_version().await?;
        conn.login(config).await?;

        Ok(conn)
    }

    /// Gracefully closes the connection.
    pub async fn close(mut self) -> crate::Result<()> {
        self.logout().await?;
        Ok(())
    }

    /// Sends username and password, as applicable.
    async fn login(&mut self, config: &Config) -> crate::Result<()> {
        if let Some(auth) = config.auth.clone() {
            // Pass username and check for 'OK'
            self.set_username(&auth.username).await?;

            // Pass password and check for 'OK'
            if let Some(password) = &auth.password {
                self.set_password(password).await?;
            }
        }
        Ok(())
    }
}

/// A blocking TCP NUT client connection.
pub struct TcpConnection {
    config: Config,
    stream: ConnectionStream,
}

impl TcpConnection {
    async fn new(config: Config, socket_addr: &SocketAddr) -> crate::Result<Self> {
        // Create the TCP connection
        let tcp_stream = TcpStream::connect(socket_addr).await?;
        let mut connection = Self {
            config,
            stream: ConnectionStream::Tcp(tcp_stream),
        };
        connection = connection.enable_ssl().await?;
        Ok(connection)
    }

    #[cfg(feature = "async-ssl")]
    async fn enable_ssl(mut self) -> crate::Result<Self> {
        if self.config.ssl {
            // Send TLS request and check for 'OK'
            self.write_cmd(Command::StartTLS).await?;
            self.read_response()
                .await
                .map_err(|e| {
                    if let Error::Nut(NutError::FeatureNotConfigured) = e {
                        Error::Nut(NutError::SslNotSupported)
                    } else {
                        e
                    }
                })?
                .expect_ok()?;

            let mut ssl_config = rustls::ClientConfig::new();
            let dns_name: webpki::DNSName;

            if self.config.ssl_insecure {
                ssl_config
                    .dangerous()
                    .set_certificate_verifier(std::sync::Arc::new(
                        crate::ssl::InsecureCertificateValidator::new(&self.config),
                    ));

                dns_name = webpki::DNSNameRef::try_from_ascii_str("www.google.com")
                    .unwrap()
                    .to_owned();
            } else {
                // Try to get hostname as given (e.g. localhost can be used for strict SSL, but not 127.0.0.1)
                let hostname = self
                    .config
                    .host
                    .hostname()
                    .ok_or(Error::Nut(NutError::SslInvalidHostname))?;

                dns_name = webpki::DNSNameRef::try_from_ascii_str(&hostname)
                    .map_err(|_| Error::Nut(NutError::SslInvalidHostname))?
                    .to_owned();

                ssl_config
                    .root_store
                    .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
            };

            let config = tokio_rustls::TlsConnector::from(std::sync::Arc::new(ssl_config));

            // Wrap and override the TCP stream
            self.stream = self
                .stream
                .upgrade_ssl_client(config, dns_name.as_ref())
                .await?;
        }
        Ok(self)
    }

    #[cfg(not(feature = "async-ssl"))]
    async fn enable_ssl(self) -> crate::Result<Self> {
        Ok(self)
    }

    pub(crate) async fn write_cmd(&mut self, line: Command<'_>) -> crate::Result<()> {
        let line = format!("{}\n", line);
        if self.config.debug {
            eprint!("DEBUG -> {}", line);
        }
        self.stream.write_all(line.as_bytes()).await?;
        self.stream.flush().await?;
        Ok(())
    }

    async fn parse_line(
        reader: &mut BufReader<&mut ConnectionStream>,
        debug: bool,
    ) -> crate::Result<Vec<String>> {
        let mut raw = String::new();
        reader.read_line(&mut raw).await?;
        if debug {
            eprint!("DEBUG <- {}", raw);
        }
        raw = raw[..raw.len() - 1].to_string(); // Strip off \n

        // Parse args by splitting whitespace, minding quotes for args with multiple words
        let args = shell_words::split(&raw)
            .map_err(|e| NutError::generic(format!("Parsing server response failed: {}", e)))?;

        Ok(args)
    }

    pub(crate) async fn read_response(&mut self) -> crate::Result<Response> {
        let mut reader = BufReader::new(&mut self.stream);
        let args = Self::parse_line(&mut reader, self.config.debug).await?;
        Response::from_args(args)
    }

    pub(crate) async fn read_plain_response(&mut self) -> crate::Result<String> {
        let mut reader = BufReader::new(&mut self.stream);
        let args = Self::parse_line(&mut reader, self.config.debug).await?;
        Ok(args.join(" "))
    }

    pub(crate) async fn read_list(&mut self, query: &[&str]) -> crate::Result<Vec<Response>> {
        let mut reader = BufReader::new(&mut self.stream);
        let args = Self::parse_line(&mut reader, self.config.debug).await?;

        Response::from_args(args)?.expect_begin_list(query)?;
        let mut lines: Vec<Response> = Vec::new();

        loop {
            let args = Self::parse_line(&mut reader, self.config.debug).await?;
            let resp = Response::from_args(args)?;

            match resp {
                Response::EndList(_) => {
                    break;
                }
                _ => lines.push(resp),
            }
        }

        Ok(lines)
    }
}
