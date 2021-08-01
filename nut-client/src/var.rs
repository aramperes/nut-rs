use core::fmt;
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
