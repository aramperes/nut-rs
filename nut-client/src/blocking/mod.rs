use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};

use crate::blocking::stream::ConnectionStream;
use crate::cmd::{Command, Response};
use crate::{ClientError, Config, Host, NutError, Variable};

mod stream;

/// A blocking NUT client connection.
pub enum Connection {
    /// A TCP connection.
    Tcp(TcpConnection),
}

impl Connection {
    /// Initializes a connection to a NUT server (upsd).
    pub fn new(config: &Config) -> crate::Result<Self> {
        match &config.host {
            Host::Tcp(socket_addr) => {
                Ok(Self::Tcp(TcpConnection::new(config.clone(), socket_addr)?))
            }
        }
    }

    /// Queries a list of UPS devices.
    pub fn list_ups(&mut self) -> crate::Result<Vec<(String, String)>> {
        match self {
            Self::Tcp(conn) => conn.list_ups(),
        }
    }

    /// Queries a list of client IP addresses connected to the given device.
    pub fn list_clients(&mut self, ups_name: &str) -> crate::Result<Vec<String>> {
        match self {
            Self::Tcp(conn) => conn.list_clients(ups_name),
        }
    }

    /// Queries the list of variables for a UPS device.
    pub fn list_vars(&mut self, ups_name: &str) -> crate::Result<Vec<Variable>> {
        match self {
            Self::Tcp(conn) => Ok(conn
                .list_vars(ups_name)?
                .into_iter()
                .map(|(key, val)| Variable::parse(key.as_str(), val))
                .collect()),
        }
    }

    /// Queries one variable for a UPS device.
    pub fn get_var(&mut self, ups_name: &str, variable: &str) -> crate::Result<Variable> {
        match self {
            Self::Tcp(conn) => {
                let var = conn.get_var(ups_name, variable)?;
                Ok(Variable::parse(var.0.as_str(), var.1))
            }
        }
    }
}

/// A blocking TCP NUT client connection.
pub struct TcpConnection {
    config: Config,
    pipeline: ConnectionStream,
}

impl TcpConnection {
    fn new(config: Config, socket_addr: &SocketAddr) -> crate::Result<Self> {
        // Create the TCP connection
        let tcp_stream = TcpStream::connect_timeout(socket_addr, config.timeout)?;
        let mut connection = Self {
            config,
            pipeline: ConnectionStream::Plain(tcp_stream),
        };

        // Initialize SSL connection
        connection = connection.enable_ssl()?;

        // Attempt login using `config.auth`
        connection.login()?;

        Ok(connection)
    }

    #[cfg(feature = "ssl")]
    fn enable_ssl(mut self) -> crate::Result<Self> {
        if self.config.ssl {
            // Send TLS request and check for 'OK'
            self.write_cmd(Command::StartTLS)?;
            self.read_response()
                .map_err(|e| {
                    if let ClientError::Nut(NutError::FeatureNotConfigured) = e {
                        ClientError::Nut(NutError::SslNotSupported)
                    } else {
                        e
                    }
                })?
                .expect_ok()?;

            let mut config = rustls::ClientConfig::new();
            config
                .dangerous()
                .set_certificate_verifier(std::sync::Arc::new(
                    crate::ssl::NutCertificateValidator::new(&self.config),
                ));

            // todo: this DNS name is temporary; should get from connection hostname? (#8)
            let dns_name = webpki::DNSNameRef::try_from_ascii_str("www.google.com").unwrap();
            let sess = rustls::ClientSession::new(&std::sync::Arc::new(config), dns_name);

            // Wrap and override the TCP stream
            self.pipeline = self.pipeline.upgrade_ssl(sess)?;

            // Send a test command
            self.get_network_version()?;
        }
        Ok(self)
    }

    #[cfg(not(feature = "ssl"))]
    fn enable_ssl(&mut self) -> crate::Result<()> {
        Ok(())
    }

    fn login(&mut self) -> crate::Result<()> {
        if let Some(auth) = self.config.auth.clone() {
            // Pass username and check for 'OK'
            self.write_cmd(Command::SetUsername(&auth.username))?;
            self.read_response()?.expect_ok()?;

            // Pass password and check for 'OK'
            if let Some(password) = &auth.password {
                self.write_cmd(Command::SetPassword(password))?;
                self.read_response()?.expect_ok()?;
            }
        }
        Ok(())
    }

    fn list_ups(&mut self) -> crate::Result<Vec<(String, String)>> {
        let query = &["UPS"];
        self.write_cmd(Command::List(query))?;

        let list = self.read_list(query)?;
        list.into_iter().map(|row| row.expect_ups()).collect()
    }

    fn list_clients(&mut self, ups_name: &str) -> crate::Result<Vec<String>> {
        let query = &["CLIENT", ups_name];
        self.write_cmd(Command::List(query))?;

        let list = self.read_list(query)?;
        list.into_iter().map(|row| row.expect_client()).collect()
    }

    fn list_vars(&mut self, ups_name: &str) -> crate::Result<Vec<(String, String)>> {
        let query = &["VAR", ups_name];
        self.write_cmd(Command::List(query))?;

        let list = self.read_list(query)?;
        list.into_iter().map(|row| row.expect_var()).collect()
    }

    fn get_var(&mut self, ups_name: &str, variable: &str) -> crate::Result<(String, String)> {
        let query = &["VAR", ups_name, variable];
        self.write_cmd(Command::Get(query))?;

        self.read_response()?.expect_var()
    }

    fn get_network_version(&mut self) -> crate::Result<String> {
        self.write_cmd(Command::NetworkVersion)?;
        self.read_plain_response()
    }

    fn write_cmd(&mut self, line: Command) -> crate::Result<()> {
        let line = format!("{}\n", line);
        if self.config.debug {
            eprint!("DEBUG -> {}", line);
        }
        self.pipeline.write_all(line.as_bytes())?;
        self.pipeline.flush()?;
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
            .map_err(|e| NutError::Generic(format!("Parsing server response failed: {}", e)))?;

        Ok(args)
    }

    fn read_response(&mut self) -> crate::Result<Response> {
        let mut reader = BufReader::new(&mut self.pipeline);
        let args = Self::parse_line(&mut reader, self.config.debug)?;
        Response::from_args(args)
    }

    fn read_plain_response(&mut self) -> crate::Result<String> {
        let mut reader = BufReader::new(&mut self.pipeline);
        let args = Self::parse_line(&mut reader, self.config.debug)?;
        Ok(args.join(" "))
    }

    fn read_list(&mut self, query: &[&str]) -> crate::Result<Vec<Response>> {
        let mut reader = BufReader::new(&mut self.pipeline);
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
