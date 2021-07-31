use std::io;
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};

use crate::cmd::{Command, Response};
use crate::{Config, Host, NutError, Variable};

/// A blocking NUT client connection.
pub enum Connection {
    /// A TCP connection.
    Tcp(TcpConnection),
}

impl Connection {
    /// Initializes a connection to a NUT server (upsd).
    pub fn new(config: Config) -> crate::Result<Self> {
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
#[derive(Debug)]
pub struct TcpConnection {
    config: Config,
    tcp_stream: TcpStream,
}

impl TcpConnection {
    fn new(config: Config, socket_addr: &SocketAddr) -> crate::Result<Self> {
        // Create the TCP connection
        let tcp_stream = TcpStream::connect_timeout(socket_addr, config.timeout)?;
        let mut connection = Self { config, tcp_stream };

        // Attempt login using `config.auth`
        connection.login()?;

        Ok(connection)
    }

    fn login(&mut self) -> crate::Result<()> {
        if let Some(auth) = &self.config.auth {
            // Pass username and check for 'OK'
            Self::write_cmd(
                &mut self.tcp_stream,
                Command::SetUsername(&auth.username),
                self.config.debug,
            )?;
            Self::read_response(&mut self.tcp_stream, self.config.debug)?.expect_ok()?;

            // Pass password and check for 'OK'
            if let Some(password) = &auth.password {
                Self::write_cmd(
                    &mut self.tcp_stream,
                    Command::SetPassword(password),
                    self.config.debug,
                )?;
                Self::read_response(&mut self.tcp_stream, self.config.debug)?.expect_ok()?;
            }
        }
        Ok(())
    }

    fn list_ups(&mut self) -> crate::Result<Vec<(String, String)>> {
        Self::write_cmd(
            &mut self.tcp_stream,
            Command::List(&["UPS"]),
            self.config.debug,
        )?;
        let list = Self::read_list(&mut self.tcp_stream, &["UPS"], self.config.debug)?;

        list.into_iter().map(|row| row.expect_ups()).collect()
    }

    fn list_clients(&mut self, ups_name: &str) -> crate::Result<Vec<String>> {
        let query = &["CLIENT", ups_name];
        Self::write_cmd(
            &mut self.tcp_stream,
            Command::List(query),
            self.config.debug,
        )?;
        let list = Self::read_list(&mut self.tcp_stream, query, self.config.debug)?;

        list.into_iter().map(|row| row.expect_client()).collect()
    }

    fn list_vars(&mut self, ups_name: &str) -> crate::Result<Vec<(String, String)>> {
        let query = &["VAR", ups_name];
        Self::write_cmd(
            &mut self.tcp_stream,
            Command::List(query),
            self.config.debug,
        )?;
        let list = Self::read_list(&mut self.tcp_stream, query, self.config.debug)?;

        list.into_iter().map(|row| row.expect_var()).collect()
    }

    fn get_var(&mut self, ups_name: &str, variable: &str) -> crate::Result<(String, String)> {
        let query = &["VAR", ups_name, variable];
        Self::write_cmd(&mut self.tcp_stream, Command::Get(query), self.config.debug)?;

        let resp = Self::read_response(&mut self.tcp_stream, self.config.debug)?;
        resp.expect_var()
    }

    fn write_cmd(stream: &mut TcpStream, line: Command, debug: bool) -> crate::Result<()> {
        let line = format!("{}\n", line);
        if debug {
            eprint!("DEBUG -> {}", line);
        }
        stream.write_all(line.as_bytes())?;
        stream.flush()?;
        Ok(())
    }

    fn parse_line(
        reader: &mut BufReader<&mut TcpStream>,
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

    fn read_response(stream: &mut TcpStream, debug: bool) -> crate::Result<Response> {
        let mut reader = io::BufReader::new(stream);
        let args = Self::parse_line(&mut reader, debug)?;
        Response::from_args(args)
    }

    fn read_list(
        stream: &mut TcpStream,
        query: &[&str],
        debug: bool,
    ) -> crate::Result<Vec<Response>> {
        let mut reader = io::BufReader::new(stream);
        let args = Self::parse_line(&mut reader, debug)?;

        Response::from_args(args)?.expect_begin_list(query)?;
        let mut lines: Vec<Response> = Vec::new();

        loop {
            let args = Self::parse_line(&mut reader, debug)?;
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
