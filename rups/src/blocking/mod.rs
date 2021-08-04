use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};

use crate::blocking::stream::ConnectionStream;
use crate::cmd::{Command, Response};
use crate::{Config, Error, Host, NutError};

mod stream;

/// A blocking NUT client connection.
pub enum Connection {
    /// A TCP connection.
    Tcp(TcpConnection),
}

impl Connection {
    /// Initializes a connection to a NUT server (upsd).
    pub fn new(config: &Config) -> crate::Result<Self> {
        let mut conn = match &config.host {
            Host::Tcp(host) => Self::Tcp(TcpConnection::new(config.clone(), &host.addr)?),
        };

        conn.get_network_version()?;
        conn.login(config)?;

        Ok(conn)
    }

    /// Gracefully closes the connection.
    pub fn close(mut self) -> crate::Result<()> {
        self.logout()?;
        Ok(())
    }

    /// Sends username and password, as applicable.
    fn login(&mut self, config: &Config) -> crate::Result<()> {
        if let Some(auth) = config.auth.clone() {
            // Pass username and check for 'OK'
            self.set_username(&auth.username)?;

            // Pass password and check for 'OK'
            if let Some(password) = &auth.password {
                self.set_password(password)?;
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
    fn new(config: Config, socket_addr: &SocketAddr) -> crate::Result<Self> {
        // Create the TCP connection
        let tcp_stream = TcpStream::connect_timeout(socket_addr, config.timeout)?;
        let mut connection = Self {
            config,
            stream: ConnectionStream::Plain(tcp_stream),
        };
        connection = connection.enable_ssl()?;
        Ok(connection)
    }

    #[cfg(feature = "ssl")]
    fn enable_ssl(mut self) -> crate::Result<Self> {
        if self.config.ssl {
            self.write_cmd(Command::StartTLS)?;
            self.read_response()
                .map_err(|e| {
                    if let Error::Nut(NutError::FeatureNotConfigured) = e {
                        Error::Nut(NutError::SslNotSupported)
                    } else {
                        e
                    }
                })?
                .expect_ok()?;

            let mut ssl_config = rustls::ClientConfig::new();
            let sess = if self.config.ssl_insecure {
                ssl_config
                    .dangerous()
                    .set_certificate_verifier(std::sync::Arc::new(
                        crate::ssl::InsecureCertificateValidator::new(&self.config),
                    ));

                let dns_name = webpki::DNSNameRef::try_from_ascii_str("www.google.com").unwrap();

                rustls::ClientSession::new(&std::sync::Arc::new(ssl_config), dns_name)
            } else {
                // Try to get hostname as given (e.g. localhost can be used for strict SSL, but not 127.0.0.1)
                let hostname = self
                    .config
                    .host
                    .hostname()
                    .ok_or(Error::Nut(NutError::SslInvalidHostname))?;

                let dns_name = webpki::DNSNameRef::try_from_ascii_str(&hostname)
                    .map_err(|_| Error::Nut(NutError::SslInvalidHostname))?;

                ssl_config
                    .root_store
                    .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

                rustls::ClientSession::new(&std::sync::Arc::new(ssl_config), dns_name)
            };

            // Wrap and override the TCP stream
            self.stream = self.stream.upgrade_ssl_client(sess)?;
        }
        Ok(self)
    }

    #[cfg(not(feature = "ssl"))]
    fn enable_ssl(self) -> crate::Result<Self> {
        Ok(self)
    }

    pub(crate) fn write_cmd(&mut self, line: Command) -> crate::Result<()> {
        let line = format!("{}\n", line);
        if self.config.debug {
            eprint!("DEBUG -> {}", line);
        }
        self.stream.write_all(line.as_bytes())?;
        self.stream.flush()?;
        Ok(())
    }

    fn parse_line(
        reader: &mut BufReader<&mut ConnectionStream>,
        debug: bool,
    ) -> crate::Result<Vec<String>> {
        let mut raw = String::new();
        reader.read_line(&mut raw)?;
        if debug {
            eprint!("DEBUG <- {}", raw);
        }
        raw = raw[..raw.len() - 1].to_string(); // Strip off \n

        // Parse args by splitting whitespace, minding quotes for args with multiple words
        let args = shell_words::split(&raw)
            .map_err(|e| NutError::generic(format!("Parsing server response failed: {}", e)))?;

        Ok(args)
    }

    pub(crate) fn read_response(&mut self) -> crate::Result<Response> {
        let mut reader = BufReader::new(&mut self.stream);
        let args = Self::parse_line(&mut reader, self.config.debug)?;
        Response::from_args(args)
    }

    pub(crate) fn read_plain_response(&mut self) -> crate::Result<String> {
        let mut reader = BufReader::new(&mut self.stream);
        let args = Self::parse_line(&mut reader, self.config.debug)?;
        Ok(args.join(" "))
    }

    pub(crate) fn read_list(&mut self, query: &[&str]) -> crate::Result<Vec<Response>> {
        let mut reader = BufReader::new(&mut self.stream);
        let args = Self::parse_line(&mut reader, self.config.debug)?;

        Response::from_args(args)?.expect_begin_list(query)?;
        let mut lines: Vec<Response> = Vec::new();

        loop {
            let args = Self::parse_line(&mut reader, self.config.debug)?;
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
