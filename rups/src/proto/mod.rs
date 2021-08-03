///! NUT protocol implementation (v1.2).
///! Reference: https://networkupstools.org/docs/developer-guide.chunked/ar01s09.html

macro_rules! impl_words {
    (
        $(
            $(#[$attr:meta])+
            $name:ident($word:tt),
        )*
    ) => {
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
        QueryVersion (
            {
                0: Version,
                1: EOL,
            },
            {}
        ),
        QueryListVar (
            {
                0: List,
                1: Var,
                2: Arg,
                3: EOL,
            },
            {
                2: ups_name: String,
            }
        )
    }

    // TODO: Macro
    #[derive(Debug, Clone, Eq, PartialEq)]
    pub enum Sentences {
        QueryVersion {},
        QueryNetworkVersion {},
        QueryHelp {},
        QueryListUps {},
        QueryListVar { ups_name: String },
        QueryListRw { ups_name: String },
    }

    // TODO: Macro
    impl Sentences {
        pub(crate) fn decode(raw: Vec<String>) -> Option<Sentences> {
            use super::{Word::*, *};
            use Sentences::*;
            let words = Word::decode_words(raw.as_slice());

            if Version.matches(words.get(0)) && EOL.matches(words.get(1)) {
                return Some(QueryVersion {});
            }
            if List.matches(words.get(0))
                && Var.matches(words.get(1))
                && Arg.matches(words.get(2))
                && EOL.matches(words.get(3))
            {
                return Some(QueryListVar {
                    ups_name: raw[2].to_owned(),
                });
            }

            None
        }
    }
}

// TODO Macro
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_serverbound_decode() {
        assert_eq!(
            server_bound::Sentences::decode(vec!["VERSION".into()]),
            Some(server_bound::Sentences::QueryVersion {})
        );
        assert_eq!(
            server_bound::Sentences::decode(vec!["LIST".into(), "VAR".into(), "nutdev".into()]),
            Some(server_bound::Sentences::QueryListVar {
                ups_name: "nutdev".into()
            })
        );
        assert_eq!(
            server_bound::Sentences::decode(vec!["LIST".into(), "RW".into(), "nutdev".into()]),
            Some(server_bound::Sentences::QueryListRw {
                ups_name: "nutdev".into()
            })
        );
    }
}
