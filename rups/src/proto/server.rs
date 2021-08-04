use crate::proto::impl_sentences;

impl_sentences! {
    /// Client requests the number of prior logins to the given `ups_name` device.
    QueryNumLogins (
        {
            0: Get,
            1: NumLogins,
            2: Arg,
            3: EOL,
        },
        {
            /// The name of the UPS device.
            2: ups_name,
        }
    ),
    /// Client requests the description of the given `ups_name` device.
    QueryUpsDesc (
        {
            0: Get,
            1: UpsDesc,
            2: Arg,
            3: EOL,
        },
        {
            /// The name of the UPS device.
            2: ups_name,
        }
    ),
    /// Client requests the value of the given `var_name` variable in the given `ups_name` device.
    QueryVar (
        {
            0: Get,
            1: Var,
            2: Arg,
            3: Arg,
            4: EOL,
        },
        {
            /// The name of the UPS device.
            2: ups_name,
            /// The name of the variable.
            3: var_name,
        }
    ),
    /// Client requests the type of the given `var_name` variable in the given `ups_name` device.
    QueryType (
        {
            0: Get,
            1: Type,
            2: Arg,
            3: Arg,
            4: EOL,
        },
        {
            /// The name of the UPS device.
            2: ups_name,
            /// The name of the variable.
            3: var_name,
        }
    ),
    /// Client requests the description of the given `var_name` variable in the given `ups_name` device.
    QueryDesc (
        {
            0: Get,
            1: Desc,
            2: Arg,
            3: Arg,
            4: EOL,
        },
        {
            /// The name of the UPS device.
            2: ups_name,
            /// The name of the variable.
            3: var_name,
        }
    ),
    /// Client requests the description of the given `cmd_name` command in the given `ups_name` device.
    QueryCmdDesc (
        {
            0: Get,
            1: CmdDesc,
            2: Arg,
            3: Arg,
            4: EOL,
        },
        {
            /// The name of the UPS device.
            2: ups_name,
            /// The name of the command.
            3: cmd_name,
        }
    ),
    /// Client requests the list of UPS devices.
    QueryListUps (
        {
            0: List,
            1: Ups,
            2: EOL,
        },
        {}
    ),
    /// Client requests the list of variables for the given `ups_name` device.
    QueryListVar (
        {
            0: List,
            1: Var,
            2: Arg,
            3: EOL,
        },
        {
            /// The name of the UPS device.
            2: ups_name,
        }
    ),
    /// Client requests the list of mutable variables for the given `ups_name` device.
    QueryListRw (
        {
            0: List,
            1: Rw,
            2: Arg,
            3: EOL,
        },
        {
            /// The name of the UPS device.
            2: ups_name,
        }
    ),
    /// Client requests the list of commands for the given `ups_name` device.
    QueryListCmd (
        {
            0: List,
            1: Cmd,
            2: Arg,
            3: EOL,
        },
        {
            /// The name of the UPS device.
            2: ups_name,
        }
    ),
    /// Client requests the list of possible values of the enumerable variable `var_name`
    /// for the given `ups_name` device.
    QueryListEnum (
        {
            0: List,
            1: Enum,
            2: Arg,
            3: Arg,
            4: EOL,
        },
        {
            /// The name of the UPS device.
            2: ups_name,
            /// The name of the variable.
            3: var_name,
        }
    ),
    /// Client requests the list of possible ranges of the numerical variable `var_name`
    /// for the given `ups_name` device.
    QueryListRange (
        {
            0: List,
            1: Range,
            2: Arg,
            3: Arg,
            4: EOL,
        },
        {
            /// The name of the UPS device.
            2: ups_name,
            /// The name of the variable.
            3: var_name,
        }
    ),
    /// Client requests the list of clients connected to the given `ups_name` device.
    QueryListClient (
        {
            0: List,
            1: Client,
            2: Arg,
            3: EOL,
        },
        {
            /// The name of the UPS device.
            2: ups_name,
        }
    ),
    /// Client requests to set the value `value` of the `var_name` variable on the `ups_name` device.
    ExecSetVar (
        {
            0: Set,
            1: Var,
            2: Arg,
            3: Arg,
            4: Arg,
            5: EOL,
        },
        {
            /// The name of the UPS device.
            2: ups_name,
            /// The name of the variable.
            3: var_name,
            /// The new value of the variable.
            4: value,
        }
    ),
    /// Client requests the execution of an instant command `cmd_name` on the `ups_name` device.
    ExecInstCmd (
        {
            0: InstCmd,
            1: Arg,
            2: Arg,
            3: EOL,
        },
        {
            /// The name of the UPS device.
            1: ups_name,
            /// The name of the command.
            2: cmd_name,
        }
    ),
    /// Client logs-out of the current UPS device.
    ExecLogout (
        {
            0: Logout,
            1: EOL,
        },
        {}
    ),
    /// Client logs-into the given `ups_name` device.
    ExecLogin (
        {
            0: Login,
            1: Arg,
            2: EOL,
        },
        {
            /// The name of the UPS device.
            1: ups_name,
        }
    ),
    /// Client asserts master-level access to the `ups_name` device.
    ExecMaster (
        {
            0: Master,
            1: Arg,
            2: EOL,
        },
        {
            /// The name of the UPS  device.
            1: ups_name,
        }
    ),
    /// Client requests the forced shut-down of the `ups_name` device.
    ExecForcedShutDown (
        {
            0: Fsd,
            1: Arg,
            2: EOL,
        },
        {
            /// The name of the UPS device.
            1: ups_name,
        }
    ),
    /// Client sets the password on the connection.
    SetPassword (
        {
            0: Password,
            1: Arg,
            2: EOL,
        },
        {
            /// The password to set.
            1: password,
        }
    ),
    /// Client sets the username on the connection.
    SetUsername (
        {
            0: Username,
            1: Arg,
            2: EOL,
        },
        {
            /// The username to set.
            1: username,
        }
    ),
    /// Client requests the connection be upgraded to TLS.
    ExecStartTLS (
        {
            0: StartTLS,
            1: EOL,
        },
        {}
    ),
    /// Client requests the list of commands supported by the server.
    QueryHelp (
        {
            0: Help,
            1: EOL,
        },
        {}
    ),
    /// Client requests the server version.
    QueryVersion (
        {
            0: Version,
            1: EOL,
        },
        {}
    ),
    /// Client requests the network version.
    QueryNetworkVersion (
        {
            0: NetworkVersion,
            1: EOL,
        },
        {}
    ),
}

#[allow(clippy::from_over_into)]
impl Into<crate::Result<Self>> for Sentences {
    fn into(self) -> crate::Result<Sentences> {
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::Sentences;
    use crate::proto::{test_encode_decode, Sentence};
    #[test]
    fn test_encode_decode() {
        test_encode_decode!(
            ["GET", "NUMLOGINS", "nutdev"] <=>
            Sentences::QueryNumLogins {
                ups_name: "nutdev".into(),
            }
        );
        test_encode_decode!(
            ["GET", "NUMLOGINS", "nutdev"] <=>
            Sentences::QueryNumLogins {
                ups_name: "nutdev".into(),
            }
        );
        test_encode_decode!(
            ["GET", "UPSDESC", "nutdev"] <=>
            Sentences::QueryUpsDesc {
                ups_name: "nutdev".into(),
            }
        );
        test_encode_decode!(
            ["GET", "VAR", "nutdev", "test.var"] <=>
            Sentences::QueryVar {
                ups_name: "nutdev".into(),
                var_name: "test.var".into(),
            }
        );
        test_encode_decode!(
            ["GET", "TYPE", "nutdev", "test.var"] <=>
            Sentences::QueryType {
                ups_name: "nutdev".into(),
                var_name: "test.var".into(),
            }
        );
        test_encode_decode!(
            ["GET", "DESC", "nutdev", "test.var"] <=>
            Sentences::QueryDesc {
                ups_name: "nutdev".into(),
                var_name: "test.var".into(),
            }
        );
        test_encode_decode!(
            ["GET", "CMDDESC", "nutdev", "test.cmd"] <=>
            Sentences::QueryCmdDesc {
                ups_name: "nutdev".into(),
                cmd_name: "test.cmd".into(),
            }
        );
        test_encode_decode!(
            ["LIST", "VAR", "nutdev"] <=>
            Sentences::QueryListVar {
                ups_name: "nutdev".into(),
            }
        );
        test_encode_decode!(
            ["LIST", "UPS"] <=>
            Sentences::QueryListUps {}
        );
        test_encode_decode!(
            ["LIST", "RW", "nutdev"] <=>
            Sentences::QueryListRw {
                ups_name: "nutdev".into(),
            }
        );
        test_encode_decode!(
            ["LIST", "CMD", "nutdev"] <=>
            Sentences::QueryListCmd {
                ups_name: "nutdev".into(),
            }
        );
        test_encode_decode!(
            ["LIST", "ENUM", "nutdev", "test.var"] <=>
            Sentences::QueryListEnum {
                ups_name: "nutdev".into(),
                var_name: "test.var".into(),
            }
        );
        test_encode_decode!(
            ["LIST", "RANGE", "nutdev", "test.var"] <=>
            Sentences::QueryListRange {
                ups_name: "nutdev".into(),
                var_name: "test.var".into(),
            }
        );
        test_encode_decode!(
            ["LIST", "CLIENT", "nutdev"] <=>
            Sentences::QueryListClient {
                ups_name: "nutdev".into(),
            }
        );
        test_encode_decode!(
            ["SET", "VAR", "nutdev", "test.var", "something"] <=>
            Sentences::ExecSetVar {
                ups_name: "nutdev".into(),
                var_name: "test.var".into(),
                value: "something".into(),
            }
        );
        test_encode_decode!(
            ["INSTCMD", "nutdev", "test.cmd"] <=>
            Sentences::ExecInstCmd {
                ups_name: "nutdev".into(),
                cmd_name: "test.cmd".into(),
            }
        );
        test_encode_decode!(
            ["LOGOUT"] <=>
            Sentences::ExecLogout {}
        );
        test_encode_decode!(
            ["LOGIN", "nutdev"] <=>
            Sentences::ExecLogin {
                ups_name: "nutdev".into(),
            }
        );
        test_encode_decode!(
            ["MASTER", "nutdev"] <=>
            Sentences::ExecMaster {
                ups_name: "nutdev".into(),
            }
        );
        test_encode_decode!(
            ["FSD", "nutdev"] <=>
            Sentences::ExecForcedShutDown {
                ups_name: "nutdev".into(),
            }
        );
        test_encode_decode!(
            ["PASSWORD", "topsecret"] <=>
            Sentences::SetPassword {
                password: "topsecret".into(),
            }
        );
        test_encode_decode!(
            ["USERNAME", "john"] <=>
            Sentences::SetUsername {
                username: "john".into(),
            }
        );
        test_encode_decode!(
            ["STARTTLS"] <=>
            Sentences::ExecStartTLS {}
        );
        test_encode_decode!(
            ["HELP"] <=>
            Sentences::QueryHelp {}
        );
        test_encode_decode!(
            ["VERSION"] <=>
            Sentences::QueryVersion {}
        );
        test_encode_decode!(
            ["NETVER"] <=>
            Sentences::QueryNetworkVersion {}
        );
    }
}
