/// Client-bound protocol implementation.
///
/// "Client-bound" implies commands RECEIVED and DECODED by the client. The server implementation
/// must use the same messages to ENCODE and SEND.
pub mod client;
/// Server-bound protocol implementation.
///
/// "Server-bound" implies commands RECEIVED and DECODED by the server. The client implementation
/// must use the same messages to ENCODE and SEND.
pub mod server;
/// Utilities for encoding and decoding packets.
pub mod util;

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

            /// Whether the `Word` matches all words in the vec, starting at the given index.
            pub(crate) fn matches_until_end(&self, start: usize, others: &[Option<Self>]) -> bool {
                for i in start..others.len() {
                    if i == others.len() {
                        return others[i] == Some(Self::EOL);
                    }
                    if !self.matches(others.get(i)) {
                        return false;
                    }
                }
                true
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
                $(
                    ,{
                        $(#[$varargattr:meta])+
                        $varargidx:tt...: $vararg:ident
                    }
                )?
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
                    $(
                        $(#[$varargattr])*
                        $vararg: Vec<String>,
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
                    if true
                        $(&& $word.matches(words.get($wordidx)))*
                        $(&& Arg.matches_until_end($varargidx, &words))*
                    {
                        return Some($name {
                            $($arg: raw[$argidx].to_owned(),)*
                            $($vararg: raw[$varargidx..].to_owned(),)*
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
                            $($vararg,)*
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

                            $(
                                for vararg in $vararg {
                                    words.push(Some(vararg));
                                }
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
    /// Server confirms forced shut down (FSD).
    FsdSet("FSD-SET"),
    /// Serverbound query.
    Get("GET"),
    /// Server confirms logout (this is lower-case on purpose).
    Goodbye("Goodbye"),
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

pub(crate) use impl_sentences;
#[cfg(test)]
pub(crate) use test_encode_decode;
