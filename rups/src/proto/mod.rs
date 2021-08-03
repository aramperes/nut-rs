///! NUT protocol implementation (v1.2).
///! Documentation: https://networkupstools.org/docs/developer-guide.chunked/ar01s09.html

macro_rules! impl_words {
    (
        $(
            $(#[$attr:meta])+
            $name:ident($word:tt),
        )*
    ) => {
        pub(crate) enum Word {
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
            pub(crate) fn decode_words(raw: &[&str]) -> Vec<Option<Self>> {
                let mut words = Vec::new();
                for r in raw.iter() {
                    words.push(Self::decode(Some(r)));
                }
                words.push(Some(Self::EOL));
                words
            }

            /// Encodes a `Word` into a string.
            /// This function cannot encode `Arg` or `EOL` (either returns `None`).
            pub(crate) fn encode(&self) -> Option<&str> {
                match self {
                    Self::EOL => None,
                    $(Self::$name => Some($word),)*
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

// impl_serverbound! {
//     QueryVersion(Version, EOL),
//     QueryNetworkVersion(NetworkVersion, EOL),
//     QueryHelp(Help, EOL),
//     QueryListUps(List, Ups, EOL),
//     QueryListVar(List, Var, Arg(ups_name), EOL),
//     QueryListRw(List, Rw, Arg(ups_name), EOL),
// }

pub(crate) enum ServerboundSentence {
    QueryVersion,
    QueryNetworkVersion,
    QueryHelp,
    QueryListUps,
    QueryListVar { ups_name: String },
    QueryListRw { ups_name: String },
}
