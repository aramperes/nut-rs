use core::fmt;
use std::convert::TryFrom;

use crate::{ClientError, NutError, Variable, VariableDefinition, VariableRange};

#[derive(Debug, Clone)]
pub enum Command<'a> {
    Get(&'a [&'a str]),
    /// Passes the login username.
    SetUsername(&'a str),
    /// Passes the login password.
    SetPassword(&'a str),
    /// Queries for a list. Allows for any number of arguments, which forms a single query.
    List(&'a [&'a str]),
    /// Tells upsd to switch to TLS, so all future communications will be encrypted.
    StartTLS,
    /// Queries the network version.
    NetworkVersion,
    /// Queries the server version.
    Version,
    /// Gracefully shuts down the connection.
    Logout,
}

impl<'a> Command<'a> {
    /// The network identifier of the command.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Get(_) => "GET",
            Self::SetUsername(_) => "USERNAME",
            Self::SetPassword(_) => "PASSWORD",
            Self::List(_) => "LIST",
            Self::StartTLS => "STARTTLS",
            Self::NetworkVersion => "NETVER",
            Self::Version => "VER",
            Self::Logout => "LOGOUT",
        }
    }

    /// The arguments of the command to serialize.
    pub fn args(&self) -> Vec<&str> {
        match self {
            Self::Get(cmd) => cmd.to_vec(),
            Self::SetUsername(username) => vec![username],
            Self::SetPassword(password) => vec![password],
            Self::List(query) => query.to_vec(),
            _ => Vec::new(),
        }
    }
}

impl<'a> fmt::Display for Command<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut args = self.args();
        args.insert(0, self.name());
        write!(f, "{}", shell_words::join(args))
    }
}

#[derive(Debug, Clone)]
pub enum Response {
    /// A successful response.
    Ok,
    /// Marks the beginning of a list response.
    BeginList(String),
    /// Marks the end of a list response.
    EndList(String),
    /// A variable (VAR) response.
    ///
    /// Params: (var name, var value)
    Var(String, String),
    /// A UPS (UPS) response.
    ///
    /// Params: (device name, device description)
    Ups(String, String),
    /// A client (CLIENT) response.
    ///
    /// Params: (client IP)
    Client(String),
    /// A command (CMD) response.
    ///
    /// Params: (command name)
    Cmd(String),
    /// A command description (CMDDESC) response.
    ///
    /// Params: (command description)
    CmdDesc(String),
    /// A UPS description (UPSDESC) response.
    ///
    /// Params: (UPS description)
    UpsDesc(String),
    /// A mutable variable (RW) response.
    ///
    /// Params: (var name, var value)
    Rw(String, String),
    /// A variable description (DESC) response.
    ///
    /// Params: (variable description)
    Desc(String),
    /// A NUMLOGINS response.
    ///
    /// Params: (number of logins)
    NumLogins(i32),
    /// A variable type (TYPE) response.
    ///
    /// Params: (variable name, variable types)
    Type(String, Vec<String>),
    /// A variable range (RANGE) response.
    ///
    /// Params: (variable range)
    Range(VariableRange),
}

impl Response {
    pub fn from_args(mut args: Vec<String>) -> crate::Result<Response> {
        if args.is_empty() {
            return Err(
                NutError::Generic("Parsing server response failed: empty line".into()).into(),
            );
        }
        let cmd_name = args.remove(0);
        match cmd_name.as_str() {
            "OK" => Ok(Self::Ok),
            "ERR" => {
                if args.is_empty() {
                    Err(NutError::Generic("Unspecified server error".into()).into())
                } else {
                    let err_type = args.remove(0);
                    match err_type.as_str() {
                        "ACCESS-DENIED" => Err(NutError::AccessDenied.into()),
                        "UNKNOWN-UPS" => Err(NutError::UnknownUps.into()),
                        "FEATURE-NOT-CONFIGURED" => Err(NutError::FeatureNotConfigured.into()),
                        _ => Err(NutError::Generic(format!(
                            "Server error: {} {}",
                            err_type,
                            args.join(" ")
                        ))
                        .into()),
                    }
                }
            }
            "BEGIN" => {
                if args.is_empty() {
                    Err(NutError::Generic("Unspecified BEGIN type".into()).into())
                } else {
                    let begin_type = args.remove(0);
                    if &begin_type != "LIST" {
                        Err(
                            NutError::Generic(format!("Unexpected BEGIN type: {}", begin_type))
                                .into(),
                        )
                    } else {
                        let args = shell_words::join(args);
                        Ok(Response::BeginList(args))
                    }
                }
            }
            "END" => {
                if args.is_empty() {
                    Err(NutError::Generic("Unspecified END type".into()).into())
                } else {
                    let begin_type = args.remove(0);
                    if &begin_type != "LIST" {
                        Err(
                            NutError::Generic(format!("Unexpected END type: {}", begin_type))
                                .into(),
                        )
                    } else {
                        let args = shell_words::join(args);
                        Ok(Response::EndList(args))
                    }
                }
            }
            "VAR" => {
                let _var_device = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified VAR device name in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                let var_name = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified VAR name in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                let var_value = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified VAR value in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                Ok(Response::Var(var_name, var_value))
            }
            "RW" => {
                let _var_device = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified RW device name in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                let var_name = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified RW name in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                let var_value = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified RW value in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                Ok(Response::Rw(var_name, var_value))
            }
            "UPS" => {
                let name = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified UPS name in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                let description = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified UPS description in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                Ok(Response::Ups(name, description))
            }
            "CLIENT" => {
                let _device = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified CLIENT device in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                let ip_address = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified CLIENT IP in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                Ok(Response::Client(ip_address))
            }
            "CMD" => {
                let _device = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified CMD device in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                let name = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified CMD name in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                Ok(Response::Cmd(name))
            }
            "CMDDESC" => {
                let _device = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified CMDDESC device in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                let _name = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified CMDDESC name in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                let desc = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified CMDDESC description in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                Ok(Response::CmdDesc(desc))
            }
            "UPSDESC" => {
                let _device = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified UPSDESC device in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                let desc = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified UPSDESC description in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                Ok(Response::UpsDesc(desc))
            }
            "DESC" => {
                let _device = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified DESC device in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                let _name = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified DESC name in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                let desc = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified DESC description in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                Ok(Response::Desc(desc))
            }
            "NUMLOGINS" => {
                let _device = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified NUMLOGINS device in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                let num = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified NUMLOGINS number in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                let num = num.parse::<i32>().map_err(|_| {
                    ClientError::from(NutError::Generic(
                        "Invalid NUMLOGINS number in response".into(),
                    ))
                })?;
                Ok(Response::NumLogins(num))
            }
            "TYPE" => {
                let _device = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified TYPE device in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                let name = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified TYPE name in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                let types = args;
                Ok(Response::Type(name, types))
            }
            "RANGE" => {
                let _device = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified RANGE device in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                let _name = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified RANGE name in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                let min = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified RANGE min in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                let max = if args.is_empty() {
                    Err(ClientError::from(NutError::Generic(
                        "Unspecified RANGE max in response".into(),
                    )))
                } else {
                    Ok(args.remove(0))
                }?;
                Ok(Response::Range(VariableRange(min, max)))
            }
            _ => Err(NutError::UnknownResponseType(cmd_name).into()),
        }
    }

    pub fn expect_ok(&self) -> crate::Result<&Response> {
        match self {
            Self::Ok => Ok(self),
            _ => Err(NutError::UnexpectedResponse.into()),
        }
    }

    pub fn expect_begin_list(self, expected_args: &[&str]) -> crate::Result<Response> {
        let expected_args = shell_words::join(expected_args);
        if let Self::BeginList(args) = &self {
            if &expected_args == args {
                Ok(self)
            } else {
                Err(NutError::UnexpectedResponse.into())
            }
        } else {
            Err(NutError::UnexpectedResponse.into())
        }
    }

    pub fn expect_var(&self) -> crate::Result<Variable> {
        if let Self::Var(name, value) = &self {
            Ok(Variable::parse(name, value.to_owned()))
        } else {
            Err(NutError::UnexpectedResponse.into())
        }
    }

    pub fn expect_rw(&self) -> crate::Result<Variable> {
        if let Self::Rw(name, value) = &self {
            Ok(Variable::parse(name, value.to_owned()))
        } else {
            Err(NutError::UnexpectedResponse.into())
        }
    }

    pub fn expect_ups(&self) -> crate::Result<(String, String)> {
        if let Self::Ups(name, description) = &self {
            Ok((name.to_owned(), description.to_owned()))
        } else {
            Err(NutError::UnexpectedResponse.into())
        }
    }

    pub fn expect_client(&self) -> crate::Result<String> {
        if let Self::Client(client_ip) = &self {
            Ok(client_ip.to_owned())
        } else {
            Err(NutError::UnexpectedResponse.into())
        }
    }

    pub fn expect_cmd(&self) -> crate::Result<String> {
        if let Self::Cmd(name) = &self {
            Ok(name.to_owned())
        } else {
            Err(NutError::UnexpectedResponse.into())
        }
    }

    pub fn expect_cmddesc(&self) -> crate::Result<String> {
        if let Self::CmdDesc(description) = &self {
            Ok(description.to_owned())
        } else {
            Err(NutError::UnexpectedResponse.into())
        }
    }

    pub fn expect_upsdesc(&self) -> crate::Result<String> {
        if let Self::UpsDesc(description) = &self {
            Ok(description.to_owned())
        } else {
            Err(NutError::UnexpectedResponse.into())
        }
    }

    pub fn expect_desc(&self) -> crate::Result<String> {
        if let Self::Desc(description) = &self {
            Ok(description.to_owned())
        } else {
            Err(NutError::UnexpectedResponse.into())
        }
    }

    pub fn expect_numlogins(&self) -> crate::Result<i32> {
        if let Self::NumLogins(num) = &self {
            Ok(*num)
        } else {
            Err(NutError::UnexpectedResponse.into())
        }
    }

    pub fn expect_type(&self) -> crate::Result<VariableDefinition> {
        if let Self::Type(name, types) = &self {
            VariableDefinition::try_from((
                name.to_owned(),
                types.iter().map(String::as_str).collect(),
            ))
        } else {
            Err(NutError::UnexpectedResponse.into())
        }
    }

    pub fn expect_range(&self) -> crate::Result<VariableRange> {
        if let Self::Range(range) = &self {
            Ok(range.to_owned())
        } else {
            Err(NutError::UnexpectedResponse.into())
        }
    }
}

/// A macro for implementing `LIST` commands.
///
/// Each function should return a 2-tuple with
///     (1) the query to pass to `LIST`
///     (2) a closure for mapping each `Response` row to the return type
macro_rules! implement_list_commands {
    (
        $(
            $(#[$attr:meta])+
            $vis:vis fn $name:ident($($argname:ident: $argty:ty),*) -> $retty:ty {
                (
                    $query:block,
                    $mapper:block,
                )
            }
        )*
    ) => {
        impl crate::blocking::Connection {
            $(
                $(#[$attr])*
                #[allow(dead_code)]
                $vis fn $name(&mut self$(, $argname: $argty)*) -> crate::Result<$retty> {
                    match self {
                        Self::Tcp(conn) => {
                            conn.write_cmd(Command::List($query))?;
                            let list = conn.read_list($query)?;
                            list.into_iter().map($mapper).collect()
                        },
                    }
                }
            )*
        }

        #[cfg(feature = "async")]
        impl crate::tokio::Connection {
            $(
                $(#[$attr])*
                #[allow(dead_code)]
                $vis async fn $name(&mut self$(, $argname: $argty)*) -> crate::Result<$retty> {
                    match self {
                        Self::Tcp(conn) => {
                            conn.write_cmd(Command::List($query)).await?;
                            let list = conn.read_list($query).await?;
                            list.into_iter().map($mapper).collect()
                        },
                    }
                }
            )*
        }
    };
}

/// A macro for implementing `GET` commands.
///
/// Each function should return a 2-tuple with
///     (1) the query to pass to `GET`
///     (2) a closure for mapping the `Response` row to the return type
macro_rules! implement_get_commands {
    (
        $(
            $(#[$attr:meta])+
            $vis:vis fn $name:ident($($argname:ident: $argty:ty),*) -> $retty:ty {
                (
                    $query:block,
                    $mapper:block,
                )
            }
        )*
    ) => {
        impl crate::blocking::Connection {
            $(
                $(#[$attr])*
                #[allow(dead_code)]
                $vis fn $name(&mut self$(, $argname: $argty)*) -> crate::Result<$retty> {
                    match self {
                        Self::Tcp(conn) => {
                            conn.write_cmd(Command::Get($query))?;
                            ($mapper)(conn.read_response()?)
                        },
                    }
                }
            )*
        }

        #[cfg(feature = "async")]
        impl crate::tokio::Connection {
            $(
                $(#[$attr])*
                #[allow(dead_code)]
                $vis async fn $name(&mut self$(, $argname: $argty)*) -> crate::Result<$retty> {
                    match self {
                        Self::Tcp(conn) => {
                            conn.write_cmd(Command::Get($query)).await?;
                            ($mapper)(conn.read_response().await?)
                        },
                    }
                }
            )*
        }
    };
}

/// A macro for implementing simple/direct commands.
///
/// Each function should return a 2-tuple with
///     (1) the command to pass
///     (2) a closure for mapping the `String` row to the return type
macro_rules! implement_simple_commands {
    (
        $(
            $(#[$attr:meta])+
            $vis:vis fn $name:ident($($argname:ident: $argty:ty),*) -> $retty:ty {
                (
                    $cmd:block,
                    $mapper:block,
                )
            }
        )*
    ) => {
        impl crate::blocking::Connection {
            $(
                $(#[$attr])*
                #[allow(dead_code)]
                $vis fn $name(&mut self$(, $argname: $argty)*) -> crate::Result<$retty> {
                    match self {
                        Self::Tcp(conn) => {
                            conn.write_cmd($cmd)?;
                            ($mapper)(conn.read_plain_response()?)
                        },
                    }
                }
            )*
        }

        #[cfg(feature = "async")]
        impl crate::tokio::Connection {
            $(
                $(#[$attr])*
                #[allow(dead_code)]
                $vis async fn $name(&mut self$(, $argname: $argty)*) -> crate::Result<$retty> {
                    match self {
                        Self::Tcp(conn) => {
                            conn.write_cmd($cmd).await?;
                            ($mapper)(conn.read_plain_response().await?)
                        },
                    }
                }
            )*
        }
    };
}

/// A macro for implementing action commands that return `OK`.
///
/// Each function should return the command to pass.
macro_rules! implement_action_commands {
    (
        $(
            $(#[$attr:meta])+
            $vis:vis fn $name:ident($($argname:ident: $argty:ty),*) $cmd:block
        )*
    ) => {
        impl crate::blocking::Connection {
            $(
                $(#[$attr])*
                #[allow(dead_code)]
                $vis fn $name(&mut self$(, $argname: $argty)*) -> crate::Result<()> {
                    match self {
                        Self::Tcp(conn) => {
                            conn.write_cmd($cmd)?;
                            conn.read_response()?.expect_ok()?;
                            Ok(())
                        },
                    }
                }
            )*
        }

        #[cfg(feature = "async")]
        impl crate::tokio::Connection {
            $(
                $(#[$attr])*
                #[allow(dead_code)]
                $vis async fn $name(&mut self$(, $argname: $argty)*) -> crate::Result<()> {
                    match self {
                        Self::Tcp(conn) => {
                            conn.write_cmd($cmd).await?;
                            conn.read_response().await?.expect_ok()?;
                            Ok(())
                        },
                    }
                }
            )*
        }
    };
}

implement_list_commands! {
    /// Queries a list of UPS devices.
    pub fn list_ups() -> Vec<(String, String)> {
        (
            { &["UPS"] },
            { |row: Response| row.expect_ups() },
        )
    }

    /// Queries the list of client IP addresses connected to the given device.
    pub fn list_clients(ups_name: &str) -> Vec<String> {
        (
            { &["CLIENT", ups_name] },
            { |row: Response| row.expect_client() },
        )
    }

    /// Queries the list of variables for a UPS device.
    pub fn list_vars(ups_name: &str) -> Vec<Variable> {
        (
            { &["VAR", ups_name] },
            { |row: Response| row.expect_var() },
        )
    }

    /// Queries the list of mutable variables for a UPS device.
    pub fn list_mutable_vars(ups_name: &str) -> Vec<Variable> {
        (
            { &["RW", ups_name] },
            { |row: Response| row.expect_rw() },
        )
    }

    /// Queries the list of commands available for the given device.
    pub fn list_commands(ups_name: &str) -> Vec<String> {
        (
            { &["CMD", ups_name] },
            { |row: Response| row.expect_cmd() },
        )
    }

    /// Queries the possible ranges of a UPS variable.
    pub fn list_var_range(ups_name: &str, variable: &str) -> Vec<VariableRange> {
        (
            { &["RANGE", ups_name, variable] },
            { |row: Response| row.expect_range() },
        )
    }
}

implement_get_commands! {
    /// Queries one variable for a UPS device.
    pub fn get_var(ups_name: &str, variable: &str) -> Variable {
        (
            { &["VAR", ups_name, variable] },
            { |row: Response| row.expect_var() },
        )
    }

    /// Queries the description of a UPS variable.
    pub fn get_var_description(ups_name: &str, variable: &str) -> String {
        (
            { &["DESC", ups_name, variable] },
            { |row: Response| row.expect_desc() },
        )
    }

    /// Queries the type of a UPS variable.
    pub fn get_var_type(ups_name: &str, variable: &str) -> VariableDefinition {
        (
            { &["TYPE", ups_name, variable] },
            { |row: Response| row.expect_type() },
        )
    }

    /// Queries the description of a UPS command.
    pub fn get_command_description(ups_name: &str, variable: &str) -> String {
        (
            { &["CMDDESC", ups_name, variable] },
            { |row: Response| row.expect_cmddesc() },
        )
    }

    /// Queries the description of a UPS device.
    pub fn get_ups_description(ups_name: &str) -> String {
        (
            { &["UPSDESC", ups_name] },
            { |row: Response| row.expect_upsdesc() },
        )
    }

    /// Queries the number of logins to the specified UPS.
    pub fn get_num_logins(ups_name: &str) -> i32 {
        (
            { &["NUMLOGINS", ups_name] },
            { |row: Response| row.expect_numlogins() },
        )
    }
}

implement_simple_commands! {
    /// Queries the network protocol version.
    pub fn get_network_version() -> String {
        (
            { Command::NetworkVersion },
            { |row: String| Ok(row) },
        )
    }

    /// Queries the server NUT version.
    pub fn get_server_version() -> String {
        (
            { Command::Version },
            { |row: String| Ok(row) },
        )
    }
}

implement_action_commands! {
    /// Sends the login username.
    pub(crate) fn set_username(username: &str) {
        Command::SetUsername(username)
    }

    /// Sends the login password.
    pub(crate) fn set_password(password: &str) {
        Command::SetPassword(password)
    }

    /// Gracefully shuts down the connection.
    pub(crate) fn logout() {
        Command::Logout
    }
}
