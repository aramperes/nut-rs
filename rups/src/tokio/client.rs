use crate::tokio::stream::ConnectionStream;
use crate::{Config, Error, Host, NutError, TcpHost};
use tokio::net::TcpStream;

/// An asynchronous NUT client, using Tokio.
pub struct Client {
    /// The client configuration.
    config: Config,
    /// The client connection.
    pub(crate) stream: ConnectionStream,
}

impl Client {
    /// Connects to a remote NUT server using a blocking connection.
    pub async fn new(config: &Config) -> crate::Result<Self> {
        match &config.host {
            Host::Tcp(host) => Self::new_tcp(config, host).await,
        }
        // TODO: Support Unix domain sockets
    }

    /// Connects to a remote NUT server using a blocking TCP connection.
    async fn new_tcp(config: &Config, host: &TcpHost) -> crate::Result<Self> {
        let tcp_stream = TcpStream::connect(&host.addr).await?;
        let mut client = Client {
            config: config.clone(),
            stream: ConnectionStream::Tcp(tcp_stream).buffered(),
        };

        client = client.enable_ssl().await?;

        Ok(client)
    }

    /// Authenticates to the given UPS device with the username and password set in the config.
    pub async fn login(&mut self, ups_name: String) -> crate::Result<()> {
        if let Some(auth) = self.config.auth.clone() {
            // Pass username and check for 'OK'
            self.set_username(auth.username).await?;

            // Pass password and check for 'OK'
            if let Some(password) = auth.password {
                self.set_password(password).await?;
            }

            // Submit login
            self.exec_login(ups_name).await
        } else {
            Ok(())
        }
    }

    #[cfg(feature = "async-ssl")]
    async fn enable_ssl(mut self) -> crate::Result<Self> {
        if self.config.ssl {
            self.exec_start_tls().await?;

            // Initialize SSL configurations
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

            // Un-buffer to get back underlying stream
            self.stream = self.stream.unbuffered();

            // Upgrade to SSL
            self.stream = self
                .stream
                .upgrade_ssl_client(config, dns_name.as_ref())
                .await?;

            // Re-buffer
            self.stream = self.stream.buffered();
        }
        Ok(self)
    }

    #[cfg(not(feature = "async-ssl"))]
    async fn enable_ssl(self) -> crate::Result<Self> {
        Ok(self)
    }
}
