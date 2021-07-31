use std::net::SocketAddr;

use crate::cmd::{Command, Response};
use crate::tokio::stream::ConnectionStream;
use crate::{Config, Host, NutError, Variable};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

mod stream;

/// An async NUT client connection.
pub enum Connection {
    /// A TCP connection.
    Tcp(TcpConnection),
}

impl Connection {
    /// Initializes a connection to a NUT server (upsd).
    pub async fn new(config: &Config) -> crate::Result<Self> {
        match &config.host {
            Host::Tcp(host) => Ok(Self::Tcp(
                TcpConnection::new(config.clone(), &host.addr).await?,
            )),
        }
    }

    /// Queries a list of UPS devices.
    pub async fn list_ups(&mut self) -> crate::Result<Vec<(String, String)>> {
        match self {
            Self::Tcp(conn) => conn.list_ups().await,
        }
    }

    /// Queries a list of client IP addresses connected to the given device.
    pub async fn list_clients(&mut self, ups_name: &str) -> crate::Result<Vec<String>> {
        match self {
            Self::Tcp(conn) => conn.list_clients(ups_name).await,
        }
    }

    /// Queries the list of variables for a UPS device.
    pub async fn list_vars(&mut self, ups_name: &str) -> crate::Result<Vec<Variable>> {
        match self {
            Self::Tcp(conn) => Ok(conn
                .list_vars(ups_name)
                .await?
                .into_iter()
                .map(|(key, val)| Variable::parse(key.as_str(), val))
                .collect()),
        }
    }

    /// Queries one variable for a UPS device.
    pub async fn get_var(&mut self, ups_name: &str, variable: &str) -> crate::Result<Variable> {
        match self {
            Self::Tcp(conn) => {
                let var = conn.get_var(ups_name, variable).await?;
                Ok(Variable::parse(var.0.as_str(), var.1))
            }
        }
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
            stream: ConnectionStream::Plain(tcp_stream),
        };

        // Initialize SSL connection
        connection = connection.enable_ssl().await?;

        // Attempt login using `config.auth`
        connection.login().await?;

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
                    if let crate::ClientError::Nut(NutError::FeatureNotConfigured) = e {
                        crate::ClientError::Nut(NutError::SslNotSupported)
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
                    .ok_or(crate::ClientError::Nut(NutError::SslInvalidHostname))?;

                dns_name = webpki::DNSNameRef::try_from_ascii_str(&hostname)
                    .map_err(|_| crate::ClientError::Nut(NutError::SslInvalidHostname))?
                    .to_owned();

                ssl_config
                    .root_store
                    .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
            };

            let config = tokio_rustls::TlsConnector::from(std::sync::Arc::new(ssl_config));

            // Wrap and override the TCP stream
            self.stream = self.stream.upgrade_ssl(config, dns_name.as_ref()).await?;

            // Send a test command
            self.get_network_version().await?;
        }
        Ok(self)
    }

    #[cfg(not(feature = "async-ssl"))]
    async fn enable_ssl(self) -> crate::Result<Self> {
        Ok(self)
    }

    async fn login(&mut self) -> crate::Result<()> {
        if let Some(auth) = self.config.auth.clone() {
            // Pass username and check for 'OK'
            self.write_cmd(Command::SetUsername(&auth.username)).await?;
            self.read_response().await?.expect_ok()?;

            // Pass password and check for 'OK'
            if let Some(password) = &auth.password {
                self.write_cmd(Command::SetPassword(password)).await?;
                self.read_response().await?.expect_ok()?;
            }
        }
        Ok(())
    }

    async fn list_ups(&mut self) -> crate::Result<Vec<(String, String)>> {
        let query = &["UPS"];
        self.write_cmd(Command::List(query)).await?;

        let list = self.read_list(query).await?;
        list.into_iter().map(|row| row.expect_ups()).collect()
    }

    async fn list_clients(&mut self, ups_name: &str) -> crate::Result<Vec<String>> {
        let query = &["CLIENT", ups_name];
        self.write_cmd(Command::List(query)).await?;

        let list = self.read_list(query).await?;
        list.into_iter().map(|row| row.expect_client()).collect()
    }

    async fn list_vars(&mut self, ups_name: &str) -> crate::Result<Vec<(String, String)>> {
        let query = &["VAR", ups_name];
        self.write_cmd(Command::List(query)).await?;

        let list = self.read_list(query).await?;
        list.into_iter().map(|row| row.expect_var()).collect()
    }

    async fn get_var<'a>(
        &mut self,
        ups_name: &'a str,
        variable: &'a str,
    ) -> crate::Result<(String, String)> {
        let query = &["VAR", ups_name, variable];
        self.write_cmd(Command::Get(query)).await?;

        self.read_response().await?.expect_var()
    }

    #[allow(dead_code)]
    async fn get_network_version(&mut self) -> crate::Result<String> {
        self.write_cmd(Command::NetworkVersion).await?;
        self.read_plain_response().await
    }

    async fn write_cmd(&mut self, line: Command<'_>) -> crate::Result<()> {
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
            .map_err(|e| NutError::Generic(format!("Parsing server response failed: {}", e)))?;

        Ok(args)
    }

    async fn read_response(&mut self) -> crate::Result<Response> {
        let mut reader = BufReader::new(&mut self.stream);
        let args = Self::parse_line(&mut reader, self.config.debug).await?;
        Response::from_args(args)
    }

    async fn read_plain_response(&mut self) -> crate::Result<String> {
        let mut reader = BufReader::new(&mut self.stream);
        let args = Self::parse_line(&mut reader, self.config.debug).await?;
        Ok(args.join(" "))
    }

    async fn read_list(&mut self, query: &[&str]) -> crate::Result<Vec<Response>> {
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