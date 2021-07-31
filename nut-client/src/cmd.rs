use core::fmt;

use crate::{ClientError, NutError};

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

    pub fn expect_var(&self) -> crate::Result<(String, String)> {
        if let Self::Var(name, value) = &self {
            Ok((name.to_owned(), value.to_owned()))
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
}
