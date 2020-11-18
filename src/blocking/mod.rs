use std::io;
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};

use crate::cmd::{Command, Response};
use crate::{ClientError, Config, Host, NutError};

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

    /// Queries the list of variables for a UPS device.
    pub fn list_vars(&mut self, ups_name: &str) -> crate::Result<Vec<(String, String)>> {
        match self {
            Self::Tcp(conn) => conn.list_vars(ups_name),
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
            Self::write_cmd(&mut self.tcp_stream, Command::SetUsername(&auth.username))?;
            Self::read_response(&mut self.tcp_stream)?.expect_ok()?;

            // Pass password and check for 'OK'
            if let Some(password) = &auth.password {
                Self::write_cmd(&mut self.tcp_stream, Command::SetPassword(&password))?;
                Self::read_response(&mut self.tcp_stream)?.expect_ok()?;
            }
        }
        Ok(())
    }

    fn list_ups(&mut self) -> crate::Result<Vec<(String, String)>> {
        Self::write_cmd(&mut self.tcp_stream, Command::List(&["UPS"]))?;
        let list = Self::read_list(&mut self.tcp_stream, &["UPS"])?;

        Ok(list
            .into_iter()
            .map(|mut row| (row.remove(0), row.remove(0)))
            .collect())
    }

    fn list_vars(&mut self, ups_name: &str) -> crate::Result<Vec<(String, String)>> {
        let query = &["VAR", ups_name];
        Self::write_cmd(&mut self.tcp_stream, Command::List(query))?;
        let list = Self::read_list(&mut self.tcp_stream, query)?;

        Ok(list
            .into_iter()
            .map(|mut row| (row.remove(0), row.remove(0)))
            .collect())
    }

    fn write_cmd(stream: &mut TcpStream, line: Command) -> crate::Result<()> {
        let line = format!("{}\n", line);
        stream.write_all(line.as_bytes())?;
        stream.flush()?;
        Ok(())
    }

    fn parse_line(reader: &mut BufReader<&mut TcpStream>) -> crate::Result<Vec<String>> {
        let mut raw = String::new();
        reader.read_line(&mut raw)?;
        raw = raw[..raw.len() - 1].to_string(); // Strip off \n

        // Parse args by splitting whitespace, minding quotes for args with multiple words
        let args = shell_words::split(&raw)
            .map_err(|e| NutError::Generic(format!("Parsing server response failed: {}", e)))?;

        Ok(args)
    }

    fn read_response(stream: &mut TcpStream) -> crate::Result<Response> {
        let mut reader = io::BufReader::new(stream);
        let args = Self::parse_line(&mut reader)?;
        Response::from_args(args)
    }

    fn read_list(stream: &mut TcpStream, query: &[&str]) -> crate::Result<Vec<Vec<String>>> {
        let mut reader = io::BufReader::new(stream);
        let args = Self::parse_line(&mut reader)?;

        Response::from_args(args)?.expect_begin_list(query)?;
        let mut lines: Vec<Vec<String>> = Vec::new();

        loop {
            let mut args = Self::parse_line(&mut reader)?;
            let resp = Response::from_args(args.clone());

            if let Ok(resp) = resp {
                resp.expect_end_list(query)?;
                break;
            } else {
                let err = resp.unwrap_err();
                if let ClientError::Nut(err) = err {
                    if let NutError::UnknownResponseType(_) = err {
                        // Likely an item entry, let's check...
                        if args.len() < query.len() || &args[0..query.len()] != query {
                            return Err(ClientError::Nut(err));
                        } else {
                            let args = args.drain(query.len()..).collect();
                            lines.push(args);
                            continue;
                        }
                    } else {
                        return Err(ClientError::Nut(err));
                    }
                } else {
                    return Err(err);
                }
            }
        }
        Ok(lines)
    }
}
