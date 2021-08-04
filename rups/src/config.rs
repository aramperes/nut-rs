use core::fmt;
use std::convert::{TryFrom, TryInto};
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;

use crate::Error;

/// A host specification.
#[derive(Clone, Debug)]
pub enum Host {
    /// A TCP hostname, and address (IP + port).
    Tcp(TcpHost),
    // TODO: Support Unix socket streams.
}

impl Host {
    /// Returns the hostname as given, if any.
    pub fn hostname(&self) -> Option<String> {
        match self {
            Host::Tcp(host) => Some(host.hostname.to_owned()),
            // _ => None,
        }
    }
}

impl Default for Host {
    fn default() -> Self {
        (String::from("localhost"), 3493)
            .try_into()
            .expect("Failed to parse local hostname; this is a bug.")
    }
}

impl From<SocketAddr> for Host {
    fn from(addr: SocketAddr) -> Self {
        let hostname = addr.ip().to_string();
        Self::Tcp(TcpHost { hostname, addr })
    }
}

/// A TCP address, preserving the original DNS hostname if any.
#[derive(Clone, Debug)]
pub struct TcpHost {
    pub(crate) hostname: String,
    pub(crate) addr: SocketAddr,
}

impl TryFrom<(String, u16)> for Host {
    type Error = Error;

    fn try_from(hostname_port: (String, u16)) -> Result<Self, Self::Error> {
        let (hostname, _) = hostname_port.clone();
        let addr = hostname_port
            .to_socket_addrs()
            .map_err(Error::Io)?
            .next()
            .ok_or_else(|| {
                Error::Io(std::io::Error::new(
                    std::io::ErrorKind::AddrNotAvailable,
                    "no address given",
                ))
            })?;
        Ok(Host::Tcp(TcpHost { hostname, addr }))
    }
}

/// An authentication mechanism.
#[derive(Clone)]
pub struct Auth {
    /// The username of the user to login as.
    pub(crate) username: String,
    /// Optional password assigned to the remote user.
    pub(crate) password: Option<String>,
}

impl Auth {
    /// Initializes authentication credentials with a username, and optionally a password.
    pub fn new(username: String, password: Option<String>) -> Self {
        Auth { username, password }
    }
}

impl fmt::Debug for Auth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Auth")
            .field("username", &self.username)
            .field("password", &self.password.as_ref().map(|_| "(redacted)"))
            .finish()
    }
}

/// Configuration for connecting to a remote NUT server.
#[derive(Clone, Debug)]
pub struct Config {
    pub(crate) host: Host,
    pub(crate) auth: Option<Auth>,
    pub(crate) timeout: Duration,
    pub(crate) ssl: bool,
    pub(crate) ssl_insecure: bool,
    pub(crate) debug: bool,
}

impl Config {
    /// Creates a connection configuration.
    pub fn new(
        host: Host,
        auth: Option<Auth>,
        timeout: Duration,
        ssl: bool,
        ssl_insecure: bool,
        debug: bool,
    ) -> Self {
        Config {
            host,
            auth,
            timeout,
            ssl,
            ssl_insecure,
            debug,
        }
    }
}

/// A builder for [`Config`].
#[derive(Clone, Debug, Default)]
pub struct ConfigBuilder {
    host: Option<Host>,
    auth: Option<Auth>,
    timeout: Option<Duration>,
    ssl: Option<bool>,
    ssl_insecure: Option<bool>,
    debug: Option<bool>,
}

impl ConfigBuilder {
    /// Initializes an empty builder for [`Config`].
    pub fn new() -> Self {
        ConfigBuilder::default()
    }

    /// Sets the connection host, such as the TCP address and port.
    pub fn with_host(mut self, host: Host) -> Self {
        self.host = Some(host);
        self
    }

    /// Sets the optional authentication parameters.
    pub fn with_auth(mut self, auth: Option<Auth>) -> Self {
        self.auth = auth;
        self
    }

    /// Sets the network connection timeout. This may be ignored by non-network
    /// connections, such as Unix domain sockets.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Enables SSL on the connection.
    ///
    /// This will enable strict SSL verification (including hostname),
    /// unless `.with_insecure_ssl` is also set to `true`.
    #[cfg(feature = "ssl")]
    pub fn with_ssl(mut self, ssl: bool) -> Self {
        self.ssl = Some(ssl);
        self
    }

    /// Turns off SSL verification.
    ///
    /// Note: you must still use `.with_ssl(true)` to turn on SSL.
    #[cfg(feature = "ssl")]
    pub fn with_insecure_ssl(mut self, ssl_insecure: bool) -> Self {
        self.ssl_insecure = Some(ssl_insecure);
        self
    }

    /// Enables debugging network calls by printing to stderr.
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = Some(debug);
        self
    }

    /// Builds the configuration with this builder.
    pub fn build(self) -> Config {
        Config::new(
            self.host.unwrap_or_default(),
            self.auth,
            self.timeout.unwrap_or_else(|| Duration::from_secs(5)),
            self.ssl.unwrap_or(false),
            self.ssl_insecure.unwrap_or(false),
            self.debug.unwrap_or(false),
        )
    }
}
