/// Macro that implements the list of "words" in the NUT network protocol.
macro_rules! impl_words {
    (
        $(
            $(#[$attr:meta])+
            $name:ident($word:tt),
        )*
    ) => {
        #[allow(clippy::upper_case_acronyms)]
        #[derive(Debug, Copy, Clone, Eq, PartialEq)]
        pub(crate) enum Word {
            /// A string argument.
            Arg,
            /// End-of-line.
            EOL,
            $(
                /// Protocol word.
                $(#[$attr])*
                #[allow(dead_code)]
                $name,
            )*
        }

        impl Word {
            /// Matches a raw string into the corresponding word.
            /// Passing `None` will always return `EOL`. Passing an unrecognized
            /// string returns `None`.
            pub(crate) fn decode(raw: Option<&str>) -> Option<Self> {
                if let Some(raw) = raw {
                    match raw {
                        $($word => Some(Self::$name),)*
                        _ => None
                    }
                } else {
                    Some(Self::EOL)
                }
            }

            /// Decodes a sequence of words.
            /// Unrecognized words will be `None`
            /// Returns a `Vec` of the same length as the given slice.
            pub(crate) fn decode_words<T: AsRef<str>>(raw: &[T]) -> Vec<Option<Self>> {
                let mut words = Vec::new();
                for r in raw.iter() {
                    words.push(Self::decode(Some(r.as_ref())));
                }
                words.push(Some(Self::EOL));
                words
            }

            /// Encodes a `Word` into a string.
            /// This function cannot encode `Arg` or `EOL` (either returns `None`).
            pub(crate) fn encode(&self) -> Option<&str> {
                match self {
                    Self::Arg | Self::EOL => None,
                    $(Self::$name => Some($word),)*
                }
            }

            /// Whether the `Word` matches another.
            pub(crate) fn matches(&self, other: Option<&Option<Self>>) -> bool {
                if let Some(other) = other {
                    if self == &Word::Arg {
                        true
                    } else if let Some(other) = other {
                        self == other
                    } else {
                        self == &Word::EOL
                    }
                } else {
                    false
                }
            }
        }
    };
}

/// Implements the list of sentences, which are combinations
/// of words that form commands (serverbound) and responses (clientbound).
macro_rules! impl_sentences {
    (
        $(
            $(#[$attr:meta])+
            $name:ident(
                {
                    $($wordidx:tt: $word:ident,)*
                },
                {
                    $(
                        $(#[$argattr:meta])+
                        $argidx:tt: $arg:ident,
                    )*
                }
            ),
        )*
    ) => {
        /// Protocol sentences.
        #[derive(Debug, Clone, Eq, PartialEq)]
        pub enum Sentences {
            $(
                $(#[$attr])*
                $name {
                    $(
                        $(#[$argattr])*
                        $arg: String,
                    )*
                },
            )*
        }

        impl Sentences {
            /// Decodes a sentence. Returns `None` if the pattern cannot be recognized.
            pub(crate) fn decode(raw: Vec<String>) -> Option<Sentences> {
                use super::{Word::*, *};
                use Sentences::*;
                let words = Word::decode_words(raw.as_slice());

                $(
                    if true $(&& $word.matches(words.get($wordidx)))* {
                        return Some($name {
                            $($arg: raw[$argidx].to_owned(),)*
                        })
                    }
                )*

                None
            }

            /// Encodes the sentence.
            pub(crate) fn encode(&self) -> Vec<&str> {
                use super::Word::*;
                match self {
                    $(
                        Self::$name {
                            $($arg,)*
                        } => {
                            #[allow(unused_mut)]
                            let mut words = vec![
                                $(
                                    $word.encode(),
                                )*
                            ];

                            $(
                                words[$argidx] = Some($arg);
                            )*

                            words
                                .into_iter()
                                .flatten()
                                .collect()
                        }
                    )*
                }
            }
        }
    };
}

/// Macro that asserts the encoding and decoding of a valid sentence.
///
/// The two arguments, separated by `<=>`, are:
/// 1. the encoded sentence, e.g. `["GET", "VAR", "nutdev", "test.var"]`
/// 2. the decoded sentence
///
/// ```
///  test_encode_decode!(
///      ["GET", "VAR", "nutdev", "test.var"] <=>
///      Sentences::QueryVar {
///          ups_name: "nutdev".into(),
///          var_name: "test.var".into(),
///      }
///  );
/// ```
#[allow(unused_macros)]
macro_rules! test_encode_decode {
    ([$($word:expr$(,)?)+] <=> $expected:expr) => {
        assert_eq!(
            Sentences::decode(vec![
                $(String::from($word),)*
            ]),
            Some($expected)
        );
        assert_eq!(
            vec![
                $(String::from($word),)*
            ],
            $expected.encode()
        );
    };
}

impl_words! {
    /// Begins a `LIST`.
    Begin("BEGIN"),
    /// Describes a client connected to a UPS.
    Client("CLIENT"),
    /// Represents an executable command.
    Cmd("CMD"),
    /// Describes a command (`CMD`).
    CmdDesc("CMDDESC"),
    /// Describes a variable (`VAR` or `RW`).
    Desc("DESC"),
    /// Ends a block of sentences.
    End("END"),
    /// An enumerable type.
    Enum("ENUM"),
    /// An error response.
    Err("ERR"),
    /// Executes a forced shut down (FSD).
    Fsd("FSD"),
    /// Serverbound query.
    Get("GET"),
    /// Client requesting a list of commands supported by the server.
    Help("HELP"),
    /// Executes an instant command.
    InstCmd("INSTCMD"),
    /// Queries or describes a list.
    List("LIST"),
    /// Client requests login to a UPS device.
    Login("LOGIN"),
    /// Client logs out.
    Logout("LOGOUT"),
    /// Client verifying it has master-level access to the UPS device.
    Master("MASTER"),
    /// Client requests the network version.
    NetworkVersion("NETVER"),
    /// Represents the amount of logins to a UPS device.
    NumLogins("NUMLOGINS"),
    /// Clientbound response for a good outcome.
    Ok("OK"),
    /// Client setting password.
    Password("PASSWORD"),
    /// Represents a range of numerical values.
    Range("RANGE"),
    /// Represents a mutable variable.
    Rw("RW"),
    /// Client requests to set the value of a mutable variable.
    Set("SET"),
    /// Client requests the connection be upgraded to TLS.
    StartTLS("STARTTLS"),
    /// Represents the type of a variable.
    Type("TYPE"),
    /// Represents a UPS device.
    Ups("UPS"),
    /// Represents the description of a UPS device.
    UpsDesc("UPSDESC"),
    /// Client setting username.
    Username("USERNAME"),
    /// Represents a variable.
    Var("VAR"),
    /// Client requests the server version.
    Version("VERSION"),
}

/// Messages decoded by the server.
pub mod server_bound {
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

    #[cfg(test)]
    mod tests {
        use super::*;
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
}
