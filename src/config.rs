use core::fmt;
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;

/// A host specification.
#[derive(Clone, Debug)]
pub enum Host {
    /// A TCP hostname and port.
    Tcp(SocketAddr),
    // TODO: Support Unix socket streams.
}

impl Default for Host {
    fn default() -> Self {
        let addr = (String::from("localhost"), 3493)
            .to_socket_addrs()
            .expect("Failed to create local UPS socket address. This is a bug.")
            .next()
            .expect("Failed to create local UPS socket address. This is a bug.");
        Self::Tcp(addr)
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
}

impl Config {
    pub fn new(host: Host, auth: Option<Auth>, timeout: Duration) -> Self {
        Config {
            host,
            auth,
            timeout,
        }
    }
}

/// A builder for [`Config`].
#[derive(Clone, Debug, Default)]
pub struct ConfigBuilder {
    host: Option<Host>,
    auth: Option<Auth>,
    timeout: Option<Duration>,
}

impl ConfigBuilder {
    /// Initializes an empty builder for [`Config`].
    pub fn new() -> Self {
        ConfigBuilder::default()
    }

    pub fn with_host(mut self, host: Host) -> Self {
        self.host = Some(host);
        self
    }

    pub fn with_auth(mut self, auth: Option<Auth>) -> Self {
        self.auth = auth;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn build(self) -> Config {
        Config::new(
            self.host.unwrap_or_default(),
            self.auth,
            self.timeout.unwrap_or_else(|| Duration::from_secs(5)),
        )
    }
}
