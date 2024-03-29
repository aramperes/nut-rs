use std::convert::{TryFrom, TryInto};
use std::fmt;

/// The default upsd hostname.
pub const DEFAULT_HOSTNAME: &str = "localhost";
/// The default upsd port.
pub const DEFAULT_PORT: u16 = 3493;

/// TCP connection information for a upsd server.
///
/// The upsname is optional depending on context.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct UpsdName<'a> {
    /// The name of the ups device, if specified.
    pub upsname: Option<&'a str>,
    /// The hostname of the upsd server.
    pub hostname: &'a str,
    /// The port of the upsd server.
    pub port: u16,
}

impl<'a> Default for UpsdName<'a> {
    fn default() -> Self {
        UpsdName {
            upsname: None,
            hostname: DEFAULT_HOSTNAME,
            port: DEFAULT_PORT,
        }
    }
}

impl<'a> TryFrom<&'a str> for UpsdName<'a> {
    type Error = crate::ClientError;

    fn try_from(value: &'a str) -> crate::Result<UpsdName<'a>> {
        let mut upsname: Option<&str> = None;
        let mut hostname = DEFAULT_HOSTNAME;
        let mut port = DEFAULT_PORT;

        if value.contains(':') {
            let mut split = value.splitn(2, ':');
            let prefix = split.next().unwrap();
            port = split
                .next()
                .unwrap()
                .parse::<u16>()
                .map_err(|_| crate::ClientError::generic("Invalid port number"))?;
            if prefix.contains('@') {
                let mut split = prefix.splitn(2, '@');
                upsname = Some(split.next().unwrap());
                hostname = split.next().unwrap();
            } else {
                hostname = prefix;
            }
        } else if value.contains('@') {
            let mut split = value.splitn(2, '@');
            upsname = Some(split.next().unwrap());
            hostname = split.next().unwrap();
        } else {
            upsname = Some(value);
        }

        Ok(UpsdName {
            upsname,
            hostname,
            port,
        })
    }
}

impl<'a> TryInto<crate::Host> for UpsdName<'a> {
    type Error = crate::ClientError;

    fn try_into(self) -> crate::Result<crate::Host> {
        (self.hostname.to_owned(), self.port)
            .try_into()
            .map_err(|_| crate::ClientError::generic("Invalid hostname/port"))
    }
}

impl<'a> fmt::Display for UpsdName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(upsname) = self.upsname {
            write!(f, "{}@", upsname)?;
        }
        write!(f, "{}:{}", self.hostname, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::convert::TryInto;

    #[test]
    fn test_upsdname_parser_full() {
        let name: UpsdName = "ups@notlocal:1234".try_into().unwrap();
        assert_eq!(
            name,
            UpsdName {
                upsname: Some("ups"),
                hostname: "notlocal",
                port: 1234
            }
        );
        assert_eq!(format!("{}", name), "ups@notlocal:1234");
    }

    #[test]
    fn test_upsdname_parser_no_name() {
        let name: UpsdName = "notlocal:5678".try_into().unwrap();
        assert_eq!(
            name,
            UpsdName {
                upsname: None,
                hostname: "notlocal",
                port: 5678
            }
        );
        assert_eq!(format!("{}", name), "notlocal:5678");
    }

    #[test]
    fn test_upsdname_parser_no_port_no_hostname() {
        let name: UpsdName = "ups0".try_into().unwrap();
        assert_eq!(
            name,
            UpsdName {
                upsname: Some("ups0"),
                hostname: DEFAULT_HOSTNAME,
                port: DEFAULT_PORT
            }
        );
        assert_eq!(format!("{}", name), "ups0@localhost:3493");
    }

    #[test]
    fn test_upsdname_parser_no_port() {
        let name: UpsdName = "ups@notlocal".try_into().unwrap();
        assert_eq!(
            name,
            UpsdName {
                upsname: Some("ups"),
                hostname: "notlocal",
                port: DEFAULT_PORT
            }
        );
        assert_eq!(format!("{}", name), "ups@notlocal:3493");
    }
}
