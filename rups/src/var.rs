use core::fmt;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::time::Duration;

/// Well-known variable keys for NUT UPS devices.
///
/// List retrieved from: https://networkupstools.org/docs/user-manual.chunked/apcs01.html
pub mod key {
    /// Device model.
    pub const DEVICE_MODEL: &str = "device.model";
    /// Device manufacturer.
    pub const DEVICE_MANUFACTURER: &str = "device.mfr";
    /// Device serial number.
    pub const DEVICE_SERIAL: &str = "device.serial";
    /// Device type.
    pub const DEVICE_TYPE: &str = "device.type";
    /// Device description.
    pub const DEVICE_DESCRIPTION: &str = "device.description";
    /// Device administrator name.
    pub const DEVICE_CONTACT: &str = "device.contact";
    /// Device physical location.
    pub const DEVICE_LOCATION: &str = "device.location";
    /// Device part number.
    pub const DEVICE_PART: &str = "device.part";
    /// Device MAC address.
    pub const DEVICE_MAC_ADDRESS: &str = "device.macaddr";
    /// Device uptime.
    pub const DEVICE_UPTIME: &str = "device.uptime";
}

/// Well-known variables for NUT UPS devices.
///
/// List retrieved from: https://networkupstools.org/docs/user-manual.chunked/apcs01.html
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Variable {
    /// Device model.
    DeviceModel(String),
    /// Device manufacturer.
    DeviceManufacturer(String),
    /// Device serial number.
    DeviceSerial(String),
    /// Device type.
    DeviceType(DeviceType),
    /// Device description.
    DeviceDescription(String),
    /// Device administrator name.
    DeviceContact(String),
    /// Device physical location.
    DeviceLocation(String),
    /// Device part number.
    DevicePart(String),
    /// Device MAC address.
    DeviceMacAddress(String),
    /// Device uptime.
    DeviceUptime(Duration),

    /// Any other variable. Value is a tuple of (key, value).
    Other((String, String)),
}

impl Variable {
    /// Parses a variable from its key and value.
    pub fn parse(name: &str, value: String) -> Variable {
        use self::key::*;

        match name {
            DEVICE_MODEL => Self::DeviceModel(value),
            DEVICE_MANUFACTURER => Self::DeviceManufacturer(value),
            DEVICE_SERIAL => Self::DeviceSerial(value),
            DEVICE_TYPE => Self::DeviceType(DeviceType::from(value)),
            DEVICE_DESCRIPTION => Self::DeviceDescription(value),
            DEVICE_CONTACT => Self::DeviceContact(value),
            DEVICE_LOCATION => Self::DeviceLocation(value),
            DEVICE_PART => Self::DevicePart(value),
            DEVICE_MAC_ADDRESS => Self::DeviceMacAddress(value),
            DEVICE_UPTIME => Self::DeviceUptime(Duration::from_secs(
                value.parse().expect("invalid uptime value"),
            )),

            _ => Self::Other((name.into(), value)),
        }
    }

    /// Returns the NUT name of the variable.
    pub fn name(&self) -> &str {
        use self::key::*;
        match self {
            Self::DeviceModel(_) => DEVICE_MODEL,
            Self::DeviceManufacturer(_) => DEVICE_MANUFACTURER,
            Self::DeviceSerial(_) => DEVICE_SERIAL,
            Self::DeviceType(_) => DEVICE_TYPE,
            Self::DeviceDescription(_) => DEVICE_DESCRIPTION,
            Self::DeviceContact(_) => DEVICE_CONTACT,
            Self::DeviceLocation(_) => DEVICE_LOCATION,
            Self::DevicePart(_) => DEVICE_PART,
            Self::DeviceMacAddress(_) => DEVICE_MAC_ADDRESS,
            Self::DeviceUptime(_) => DEVICE_UPTIME,
            Self::Other((name, _)) => name.as_str(),
        }
    }

    /// Returns the value of the NUT variable.
    pub fn value(&self) -> String {
        match self {
            Self::DeviceModel(value) => value.clone(),
            Self::DeviceManufacturer(value) => value.clone(),
            Self::DeviceSerial(value) => value.clone(),
            Self::DeviceType(value) => value.to_string(),
            Self::DeviceDescription(value) => value.clone(),
            Self::DeviceContact(value) => value.clone(),
            Self::DeviceLocation(value) => value.clone(),
            Self::DevicePart(value) => value.clone(),
            Self::DeviceMacAddress(value) => value.clone(),
            Self::DeviceUptime(value) => value.as_secs().to_string(),
            Self::Other((_, value)) => value.clone(),
        }
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name(), self.value())
    }
}

/// NUT device type.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DeviceType {
    /// UPS (Uninterruptible Power Supply)
    Ups,
    /// PDU (Power Distribution Unit)
    Pdu,
    /// SCD (Solar Controller Device)
    Scd,
    /// PSU (Power Supply Unit)
    Psu,
    /// ATS (Automatic Transfer Switch)
    Ats,
    /// Other device type.
    Other(String),
}

impl DeviceType {
    /// Convert from string.
    pub fn from(v: String) -> DeviceType {
        match v.as_str() {
            "ups" => Self::Ups,
            "pdu" => Self::Pdu,
            "scd" => Self::Scd,
            "psu" => Self::Psu,
            "ats" => Self::Ats,
            _ => Self::Other(v),
        }
    }
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ups => write!(f, "ups"),
            Self::Pdu => write!(f, "pdu"),
            Self::Scd => write!(f, "scd"),
            Self::Psu => write!(f, "psu"),
            Self::Ats => write!(f, "ats"),
            Self::Other(val) => write!(f, "other({})", val),
        }
    }
}

/// NUT Variable type
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[allow(dead_code)]
pub(crate) enum VariableType {
    /// A mutable variable (`RW`).
    Rw,
    /// An enumerated type, which supports a few specific values (`ENUM`).
    Enum,
    /// A string with a maximum size (`STRING:n`).
    String(usize),
    /// A numeric type, either integer or float, comprised in the range defined by `LIST RANGE`.
    Range,
    /// A simple numeric value, either integer or float.
    Number,
}

impl TryFrom<&str> for VariableType {
    type Error = crate::ClientError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "RW" => Ok(Self::Rw),
            "ENUM" => Ok(Self::Enum),
            "RANGE" => Ok(Self::Range),
            "NUMBER" => Ok(Self::Number),
            other => {
                if other.starts_with("STRING:") {
                    let size = other
                        .splitn(2, ':')
                        .nth(1)
                        .map(|s| s.parse().ok())
                        .flatten()
                        .ok_or_else(|| crate::ClientError::generic("Invalid STRING definition"))?;
                    Ok(Self::String(size))
                } else {
                    Err(crate::ClientError::generic(format!(
                        "Unrecognized variable type: {}",
                        value
                    )))
                }
            }
        }
    }
}

/// NUT Variable definition.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VariableDefinition(String, HashSet<VariableType>);

impl VariableDefinition {
    /// The name of this variable.
    pub fn name(&self) -> &str {
        self.0.as_str()
    }

    /// Whether this variable is mutable.
    pub fn is_mutable(&self) -> bool {
        self.1.contains(&VariableType::Rw)
    }

    /// Whether this variable is an enumerated type.
    pub fn is_enum(&self) -> bool {
        self.1.contains(&VariableType::Enum)
    }

    /// Whether this variable is a String type
    pub fn is_string(&self) -> bool {
        self.1.iter().any(|t| matches!(t, VariableType::String(_)))
    }

    /// Whether this variable is a numeric type,
    /// either integer or float, comprised in a range
    pub fn is_range(&self) -> bool {
        self.1.contains(&VariableType::Range)
    }

    /// Whether this variable is a numeric type, either integer or float.
    pub fn is_number(&self) -> bool {
        self.1.contains(&VariableType::Number)
    }

    /// Returns the max string length, if applicable.
    pub fn get_string_length(&self) -> Option<usize> {
        self.1.iter().find_map(|t| match t {
            VariableType::String(n) => Some(*n),
            _ => None,
        })
    }
}

impl<A: ToString> TryFrom<(A, Vec<&str>)> for VariableDefinition {
    type Error = crate::ClientError;

    fn try_from(value: (A, Vec<&str>)) -> Result<Self, Self::Error> {
        Ok(VariableDefinition(
            value.0.to_string(),
            value
                .1
                .iter()
                .map(|s| VariableType::try_from(*s))
                .collect::<crate::Result<HashSet<VariableType>>>()?,
        ))
    }
}

/// A range of values for a variable.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VariableRange(pub String, pub String);

#[cfg(test)]
mod tests {
    use std::iter::FromIterator;

    use super::*;

    #[test]
    fn test_parse_variable_definition() {
        assert_eq!(
            VariableDefinition::try_from(("var0", vec![])).unwrap(),
            VariableDefinition("var0".into(), HashSet::new())
        );

        assert_eq!(
            VariableDefinition::try_from(("var1", vec!["RW"])).unwrap(),
            VariableDefinition(
                "var1".into(),
                HashSet::from_iter(vec![VariableType::Rw].into_iter())
            )
        );

        assert_eq!(
            VariableDefinition::try_from(("var1", vec!["RW", "STRING:123"])).unwrap(),
            VariableDefinition(
                "var1".into(),
                HashSet::from_iter(vec![VariableType::Rw, VariableType::String(123)].into_iter())
            )
        );

        assert!(
            VariableDefinition::try_from(("var1", vec!["RW", "STRING:123"]))
                .unwrap()
                .is_mutable()
        );
        assert!(
            VariableDefinition::try_from(("var1", vec!["RW", "STRING:123"]))
                .unwrap()
                .is_string()
        );
        assert!(
            !VariableDefinition::try_from(("var1", vec!["RW", "STRING:123"]))
                .unwrap()
                .is_enum()
        );
        assert!(
            !VariableDefinition::try_from(("var1", vec!["RW", "STRING:123"]))
                .unwrap()
                .is_number()
        );
        assert!(
            !VariableDefinition::try_from(("var1", vec!["RW", "STRING:123"]))
                .unwrap()
                .is_range()
        );
        assert_eq!(
            VariableDefinition::try_from(("var1", vec!["RW", "STRING:123"]))
                .unwrap()
                .get_string_length(),
            Some(123)
        );
    }
}
