use crate::proto::impl_sentences;

impl_sentences! {
    /// A generic successful response with no additional data.
    GenericOk (
        {
            0: Ok,
            1: EOL,
        },
        {}
    ),
    /// Forced shut down (FSD) successful.
    FsdOk (
        {
            0: Ok,
            1: FsdSet,
            2: EOL,
        },
        {}
    ),
    /// Server acknowledges TLS upgrade.
    StartTLSOk (
        {
            0: Ok,
            1: StartTLS,
            2: EOL,
        },
        {}
    ),
    /// Server confirms logout.
    LogoutOk (
        {
            0: Ok,
            1: Goodbye,
        },
        {}
    ),
    /// Server returns an error.
    RespondErr (
        {
            0: Err,
            1: Arg,
        },
        {
            /// The error code.
            1: message,
        },
        {
            /// Extra information about the error.
            2...: extras
        }
    ),
    /// Server responds with the number of prior logins to the given `ups_name` device.
    RespondNumLogins (
        {
            0: NumLogins,
            1: Arg,
            2: Arg,
            3: EOL,
        },
        {
            /// The name of the UPS device.
            1: ups_name,
            /// The number of logins to the UPS device.
            2: num_logins,
        }
    ),
    /// Server responds with the description of the UPS device.
    RespondUpsDesc (
        {
            0: UpsDesc,
            1: Arg,
            2: Arg,
            3: EOL,
        },
        {
            /// The name of the UPS device.
            1: ups_name,
            /// The description of the UPS device.
            2: description,
        }
    ),
    /// Server responds with the value of the given `var_name` variable for the UPS device.
    RespondVar (
        {
            0: Var,
            1: Arg,
            2: Arg,
            3: Arg,
            4: EOL,
        },
        {
            /// The name of the UPS device.
            1: ups_name,
            /// The name of the variable.
            2: var_name,
            /// The current value of the variable.
            3: value,
        }
    ),
    /// Server responds with the type of the given `var_name` variable for the UPS device.
    RespondType (
        {
            0: Type,
            1: Arg,
            2: Arg,
        },
        {
            /// The name of the UPS device.
            1: ups_name,
            /// The name of the variable.
            2: var_name,
        },
        {
            /// The variable definition (RW, ENUN, STRING...)
            3...: var_types
        }
    ),
    /// Server responds with the description of the given `var_name` variable for the UPS device.
    RespondDesc (
        {
            0: Desc,
            1: Arg,
            2: Arg,
            3: Arg,
            4: EOL,
        },
        {
            /// The name of the UPS device.
            1: ups_name,
            /// The name of the variable.
            2: var_name,
            /// The description of the variable.
            3: description,
        }
    ),
    /// Server responds with the description of the given `cmd_name` command for the UPS device.
    RespondCmdDesc (
        {
            0: CmdDesc,
            1: Arg,
            2: Arg,
            3: Arg,
            4: EOL,
        },
        {
            /// The name of the UPS device.
            1: ups_name,
            /// The name of the command.
            2: cmd_name,
            /// The description of the command.
            3: description,
        }
    ),
    /// Server responds with the name and description of a UPS device.
    RespondUps (
        {
            0: Ups,
            1: Arg,
            2: Arg,
            3: EOL,
        },
        {
            /// The name of the UPS device.
            1: ups_name,
            /// The name of the command.
            2: description,
        }
    ),
    /// Server responds with the name and description of a mutable variable.
    RespondRw (
        {
            0: Rw,
            1: Arg,
            2: Arg,
            3: Arg,
            4: EOL,
        },
        {
            /// The name of the UPS device.
            1: ups_name,
            /// The name of the variable.
            2: var_name,
            /// The current value of the variable.
            3: value,
        }
    ),
    /// Server responds with the name of a command.
    RespondCmd (
        {
            0: Cmd,
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
    /// Server responds with a possible value of an enumerable variable.
    RespondEnum (
        {
            0: Enum,
            1: Arg,
            2: Arg,
            3: Arg,
            4: EOL,
        },
        {
            /// The name of the UPS device.
            1: ups_name,
            /// The name of the variable.
            2: var_name,
            /// A possible value of the variable.
            3: enum_value,
        }
    ),
    /// Server responds with a possible range of an numeric variable.
    RespondRange (
        {
            0: Range,
            1: Arg,
            2: Arg,
            3: Arg,
            4: Arg,
            5: EOL,
        },
        {
            /// The name of the UPS device.
            1: ups_name,
            /// The name of the variable.
            2: var_name,
            /// The minimum value of the range.
            3: min_value,
            /// The maximum value of the range.
            4: max_value,
        }
    ),
    /// Server responds with a client connected to a UPS device.
    RespondClient (
        {
            0: Client,
            1: Arg,
            2: Arg,
            3: EOL,
        },
        {
            /// The name of the UPS device.
            1: ups_name,
            /// The IP address of the client.
            2: client_ip,
        }
    ),
    /// Server begins returning a list of UPS devices.
    BeginListUps (
        {
            0: Begin,
            1: List,
            2: Ups,
            3: EOL,
        },
        {}
    ),
    /// Server ends returning a list of UPS devices.
    EndListUps (
        {
            0: End,
            1: List,
            2: Ups,
            3: EOL,
        },
        {}
    ),
    /// Server begins returning a list of variables for a UPS device.
    BeginListVar (
        {
            0: Begin,
            1: List,
            2: Var,
            3: Arg,
            4: EOL,
        },
        {
            /// The name of the UPS device.
            3: ups_name,
        }
    ),
    /// Server ends returning a list of variables for a UPS device.
    EndListVar (
        {
            0: End,
            1: List,
            2: Var,
            3: Arg,
            4: EOL,
        },
        {
            /// The name of the UPS device.
            3: ups_name,
        }
    ),
    /// Server begins returning a list of mutable variables for a UPS device.
    BeginListRw (
        {
            0: Begin,
            1: List,
            2: Rw,
            3: Arg,
            4: EOL,
        },
        {
            /// The name of the UPS device.
            3: ups_name,
        }
    ),
    /// Server ends returning a list of mutable variables for a UPS device.
    EndListRw (
        {
            0: End,
            1: List,
            2: Rw,
            3: Arg,
            4: EOL,
        },
        {
            /// The name of the UPS device.
            3: ups_name,
        }
    ),
    /// Server begins returning a list of commands for a UPS device.
    BeginListCmd (
        {
            0: Begin,
            1: List,
            2: Cmd,
            3: Arg,
            4: EOL,
        },
        {
            /// The name of the UPS device.
            3: ups_name,
        }
    ),
    /// Server ends returning a list of commands for a UPS device.
    EndListCmd (
        {
            0: End,
            1: List,
            2: Cmd,
            3: Arg,
            4: EOL,
        },
        {
            /// The name of the UPS device.
            3: ups_name,
        }
    ),
    /// Server begins returning a list of possible values for an enumerable variable.
    BeginListEnum (
        {
            0: Begin,
            1: List,
            2: Enum,
            3: Arg,
            4: Arg,
            5: EOL,
        },
        {
            /// The name of the UPS device.
            3: ups_name,
            /// The name of the variable.
            4: var_name,
        }
    ),
    /// Server ends returning a list of possible values for an enumerable variable.
    EndListEnum (
        {
            0: End,
            1: List,
            2: Enum,
            3: Arg,
            4: Arg,
            5: EOL,
        },
        {
            /// The name of the UPS device.
            3: ups_name,
            /// The name of the variable.
            4: var_name,
        }
    ),
    /// Server begins returning a list of possible ranges for an enumerable variable.
    BeginListRange (
        {
            0: Begin,
            1: List,
            2: Range,
            3: Arg,
            4: Arg,
            5: EOL,
        },
        {
            /// The name of the UPS device.
            3: ups_name,
            /// The name of the variable.
            4: var_name,
        }
    ),
    /// Server ends returning a list of possible ranges for an enumerable variable.
    EndListRange (
        {
            0: End,
            1: List,
            2: Range,
            3: Arg,
            4: Arg,
            5: EOL,
        },
        {
            /// The name of the UPS device.
            3: ups_name,
            /// The name of the variable.
            4: var_name,
        }
    ),
    /// Server begins returning a list of clients for a UPS device.
    BeginListClient (
        {
            0: Begin,
            1: List,
            2: Client,
            3: Arg,
            4: EOL,
        },
        {
            /// The name of the UPS device.
            3: ups_name,
        }
    ),
    /// Server ends returning a list of clients for a UPS device.
    EndListClient (
        {
            0: End,
            1: List,
            2: Client,
            3: Arg,
            4: EOL,
        },
        {
            /// The name of the UPS device.
            3: ups_name,
        }
    ),
}

#[cfg(test)]
mod tests {
    use crate::proto::test_encode_decode;

    use super::Sentences;

    #[test]
    fn test_encode_decode() {
        test_encode_decode!(
            ["OK"] <=>
            Sentences::GenericOk {}
        );
        test_encode_decode!(
            ["OK", "FSD-SET"] <=>
            Sentences::FsdOk {}
        );
        test_encode_decode!(
            ["OK", "STARTTLS"] <=>
            Sentences::StartTLSOk {}
        );
        test_encode_decode!(
            ["OK", "Goodbye"] <=>
            Sentences::LogoutOk {}
        );
        test_encode_decode!(
            ["ERR", "ACCESS-DENIED"] <=>
            Sentences::RespondErr {
                message: "ACCESS-DENIED".into(),
                extras: vec![],
            }
        );
        test_encode_decode!(
            ["ERR", "ACCESS-DENIED", "extra1", "extra2"] <=>
            Sentences::RespondErr {
                message: "ACCESS-DENIED".into(),
                extras: vec!["extra1".into(), "extra2".into()],
            }
        );
        test_encode_decode!(
            ["NUMLOGINS", "nutdev", "42"] <=>
            Sentences::RespondNumLogins {
                ups_name: "nutdev".into(),
                num_logins: "42".into(),
            }
        );
        test_encode_decode!(
            ["UPSDESC", "nutdev", "Development box"] <=>
            Sentences::RespondUpsDesc {
                ups_name: "nutdev".into(),
                description: "Development box".into(),
            }
        );
        test_encode_decode!(
            ["VAR", "nutdev", "ups.status", "OL"] <=>
            Sentences::RespondVar {
                ups_name: "nutdev".into(),
                var_name: "ups.status".into(),
                value: "OL".into(),
            }
        );
        test_encode_decode!(
            ["TYPE", "nutdev", "input.transfer.low", "ENUM", "RW"] <=>
            Sentences::RespondType {
                ups_name: "nutdev".into(),
                var_name: "input.transfer.low".into(),
                var_types: vec!["ENUM".into(), "RW".into()],
            }
        );
        test_encode_decode!(
            ["DESC", "nutdev", "ups.status", "UPS status"] <=>
            Sentences::RespondDesc {
                ups_name: "nutdev".into(),
                var_name: "ups.status".into(),
                description: "UPS status".into(),
            }
        );
        test_encode_decode!(
            ["CMDDESC", "nutdev", "load.on", "Turn on the load immediately"] <=>
            Sentences::RespondCmdDesc {
                ups_name: "nutdev".into(),
                cmd_name: "load.on".into(),
                description: "Turn on the load immediately".into(),
            }
        );
        test_encode_decode!(
            ["UPS", "nutdev", "Development box"] <=>
            Sentences::RespondUps {
                ups_name: "nutdev".into(),
                description: "Development box".into(),
            }
        );
        test_encode_decode!(
            ["RW", "nutdev", "ups.mfr", "APC"] <=>
            Sentences::RespondRw {
                ups_name: "nutdev".into(),
                var_name: "ups.mfr".into(),
                value: "APC".into(),
            }
        );
        test_encode_decode!(
            ["CMD", "nutdev", "do.something"] <=>
            Sentences::RespondCmd {
                ups_name: "nutdev".into(),
                cmd_name: "do.something".into(),
            }
        );
        test_encode_decode!(
            ["ENUM", "nutdev", "input.transfer.low", "103"] <=>
            Sentences::RespondEnum {
                ups_name: "nutdev".into(),
                var_name: "input.transfer.low".into(),
                enum_value: "103".into(),
            }
        );
        test_encode_decode!(
            ["RANGE", "nutdev", "input.transfer.low", "90", "100"] <=>
            Sentences::RespondRange {
                ups_name: "nutdev".into(),
                var_name: "input.transfer.low".into(),
                min_value: "90".into(),
                max_value: "100".into(),
            }
        );
        test_encode_decode!(
            ["CLIENT", "nutdev", "127.0.0.1"] <=>
            Sentences::RespondClient {
                ups_name: "nutdev".into(),
                client_ip: "127.0.0.1".into(),
            }
        );
        test_encode_decode!(
            ["BEGIN", "LIST", "VAR", "nutdev"] <=>
            Sentences::BeginListVar {
                ups_name: "nutdev".into(),
            }
        );
        test_encode_decode!(
            ["END", "LIST", "VAR", "nutdev"] <=>
            Sentences::EndListVar {
                ups_name: "nutdev".into(),
            }
        );
        test_encode_decode!(
            ["BEGIN", "LIST", "RW", "nutdev"] <=>
            Sentences::BeginListRw {
                ups_name: "nutdev".into(),
            }
        );
        test_encode_decode!(
            ["END", "LIST", "RW", "nutdev"] <=>
            Sentences::EndListRw {
                ups_name: "nutdev".into(),
            }
        );
        test_encode_decode!(
            ["BEGIN", "LIST", "CMD", "nutdev"] <=>
            Sentences::BeginListCmd {
                ups_name: "nutdev".into(),
            }
        );
        test_encode_decode!(
            ["END", "LIST", "CMD", "nutdev"] <=>
            Sentences::EndListCmd {
                ups_name: "nutdev".into(),
            }
        );
        test_encode_decode!(
            ["BEGIN", "LIST", "ENUM", "nutdev", "test.var"] <=>
            Sentences::BeginListEnum {
                ups_name: "nutdev".into(),
                var_name: "test.var".into(),
            }
        );
        test_encode_decode!(
            ["END", "LIST", "ENUM", "nutdev", "test.var"] <=>
            Sentences::EndListEnum {
                ups_name: "nutdev".into(),
                var_name: "test.var".into(),
            }
        );
        test_encode_decode!(
            ["BEGIN", "LIST", "RANGE", "nutdev", "test.var"] <=>
            Sentences::BeginListRange {
                ups_name: "nutdev".into(),
                var_name: "test.var".into(),
            }
        );
        test_encode_decode!(
            ["END", "LIST", "RANGE", "nutdev", "test.var"] <=>
            Sentences::EndListRange {
                ups_name: "nutdev".into(),
                var_name: "test.var".into(),
            }
        );
        test_encode_decode!(
            ["BEGIN", "LIST", "CLIENT", "nutdev"] <=>
            Sentences::BeginListClient {
                ups_name: "nutdev".into(),
            }
        );
        test_encode_decode!(
            ["END", "LIST", "CLIENT", "nutdev"] <=>
            Sentences::EndListClient {
                ups_name: "nutdev".into(),
            }
        );
    }
}
