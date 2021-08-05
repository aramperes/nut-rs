use crate::blocking::stream::ConnectionStream;
use crate::proto::{ClientSentences, Sentence, ServerSentences};
use crate::{Config, Error, Host, NutError, TcpHost};
use std::net::TcpStream;

/// A synchronous NUT client.
pub struct Client {
    /// The client configuration.
    config: Config,
    /// The client connection.
    pub(crate) stream: ConnectionStream,
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

        Ok(client)
    }

    /// Authenticates to the given UPS device with the username and password set in the config.
    pub fn login(&mut self, ups_name: String) -> crate::Result<()> {
        if let Some(auth) = self.config.auth.clone() {
            // Pass username and check for 'OK'
            self.set_username(auth.username)?;

            // Pass password and check for 'OK'
            if let Some(password) = auth.password {
                self.set_password(password)?;
            }

            // Submit login
            self.exec_login(ups_name)
        } else {
            Ok(())
        }
    }

    #[cfg(feature = "ssl")]
    fn enable_ssl(mut self) -> crate::Result<Self> {
        if self.config.ssl {
            self.exec_start_tls()?;

            // Initialize SSL configurations
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

            // Un-buffer to get back underlying stream
            self.stream = self.stream.unbuffered();

            // Upgrade to SSL
            self.stream = self.stream.upgrade_ssl_client(sess)?;

            // Re-buffer
            self.stream = self.stream.buffered();
        }
        Ok(self)
    }

    #[cfg(not(feature = "ssl"))]
    fn enable_ssl(self) -> crate::Result<Self> {
        Ok(self)
    }
}
