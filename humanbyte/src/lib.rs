#![no_std]

//! Common types and functions for byte size handling
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "serde")]
pub use serde;

// Re-export necessary types to avoid users needing explicit extern crate declarations
pub use alloc::{
    format,
    string::{String, ToString},
};
pub use core::str::FromStr;

#[cfg(feature = "derive")]
pub use humanbyte_derive::*;

/// byte size for 1 byte
pub const B: u64 = 1;
/// bytes size for 1 kilobyte
pub const KB: u64 = 1_000;
/// bytes size for 1 megabyte
pub const MB: u64 = 1_000_000;
/// bytes size for 1 gigabyte
pub const GB: u64 = 1_000_000_000;
/// bytes size for 1 terabyte
pub const TB: u64 = 1_000_000_000_000;
/// bytes size for 1 petabyte
pub const PB: u64 = 1_000_000_000_000_000;

/// bytes size for 1 kibibyte
pub const KIB: u64 = 1_024;
/// bytes size for 1 mebibyte
pub const MIB: u64 = 1_048_576;
/// bytes size for 1 gibibyte
pub const GIB: u64 = 1_073_741_824;
/// bytes size for 1 tebibyte
pub const TIB: u64 = 1_099_511_627_776;
/// bytes size for 1 pebibyte
pub const PIB: u64 = 1_125_899_906_842_624;

/// IEC (binary) units.
///
/// See <https://en.wikipedia.org/wiki/Kilobyte>.
const UNITS_IEC: &str = "KMGTPE";
/// SI (decimal) units.
///
/// See <https://en.wikipedia.org/wiki/Kilobyte>.
const UNITS_SI: &str = "kMGTPE";
#[derive(Debug, Clone, Default)]
pub enum Format {
    #[default]
    IEC,
    SI,
}

pub fn to_string(bytes: u64, format: Format) -> String {
    let unit = match format {
        Format::IEC => KIB,
        Format::SI => KB,
    };
    let unit_prefix = match format {
        Format::IEC => UNITS_IEC.as_bytes(),
        Format::SI => UNITS_SI.as_bytes(),
    };
    let unit_suffix = match format {
        Format::IEC => "iB",
        Format::SI => "B",
    };
    if bytes < unit {
        format!("{} B", bytes)
    } else {
        // Integer log: largest exp such that bytes >= unit^exp.
        // (f64::ln is unavailable in core, and this is exact at boundaries.)
        let mut exp = 0u32;
        let mut n = bytes;
        while n >= unit {
            n /= unit;
            exp += 1;
        }
        format!(
            "{:.1} {}{}",
            (bytes as f64 / unit.pow(exp) as f64),
            unit_prefix[(exp - 1) as usize] as char,
            unit_suffix
        )
    }
}

#[derive(Debug)]
pub struct ParseError(pub String);

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParseError {}

pub fn take_while<P>(s: &str, mut predicate: P) -> &str
where
    P: FnMut(char) -> bool,
{
    let offset = s
        .chars()
        .take_while(|ch| predicate(*ch))
        .map(|ch| ch.len_utf8())
        .sum();
    &s[..offset]
}

pub fn skip_while<P>(s: &str, mut predicate: P) -> &str
where
    P: FnMut(char) -> bool,
{
    let offset: usize = s
        .chars()
        .skip_while(|ch| predicate(*ch))
        .map(|ch| ch.len_utf8())
        .sum();
    &s[(s.len() - offset)..]
}

pub enum Unit {
    Byte,
    // power of tens
    KiloByte,
    MegaByte,
    GigaByte,
    TeraByte,
    PetaByte,
    // power of twos
    KibiByte,
    MebiByte,
    GibiByte,
    TebiByte,
    PebiByte,
}

impl From<Unit> for u64 {
    fn from(unit: Unit) -> u64 {
        match unit {
            Unit::Byte => B,
            // power of tens
            Unit::KiloByte => KB,
            Unit::MegaByte => MB,
            Unit::GigaByte => GB,
            Unit::TeraByte => TB,
            Unit::PetaByte => PB,
            // power of twos
            Unit::KibiByte => KIB,
            Unit::MebiByte => MIB,
            Unit::GibiByte => GIB,
            Unit::TebiByte => TIB,
            Unit::PebiByte => PIB,
        }
    }
}

impl FromStr for Unit {
    type Err = String;

    fn from_str(unit: &str) -> Result<Self, Self::Err> {
        match unit.to_lowercase().as_str() {
            "b" => Ok(Self::Byte),
            // power of tens
            "k" | "kb" => Ok(Self::KiloByte),
            "m" | "mb" => Ok(Self::MegaByte),
            "g" | "gb" => Ok(Self::GigaByte),
            "t" | "tb" => Ok(Self::TeraByte),
            "p" | "pb" => Ok(Self::PetaByte),
            // power of twos
            "ki" | "kib" => Ok(Self::KibiByte),
            "mi" | "mib" => Ok(Self::MebiByte),
            "gi" | "gib" => Ok(Self::GibiByte),
            "ti" | "tib" => Ok(Self::TebiByte),
            "pi" | "pib" => Ok(Self::PebiByte),
            _ => Err(format!("couldn't parse unit of {:?}", unit)),
        }
    }
}

pub struct HumanByteRange<T: From<u64>> {
    start: T,
    stop: T,
}

impl<T: From<u64>> HumanByteRange<T> {
    pub fn new<I: Into<T>>(start: Option<I>, stop: Option<I>) -> Self {
        HumanByteRange {
            start: start.map(Into::into).unwrap_or(0.into()),
            stop: stop.map(Into::into).unwrap_or(u64::MAX.into()),
        }
    }
}

impl<T: From<u64>> core::ops::RangeBounds<T> for HumanByteRange<T> {
    fn start_bound(&self) -> core::ops::Bound<&T> {
        core::ops::Bound::Included(&self.start)
    }

    fn end_bound(&self) -> core::ops::Bound<&T> {
        core::ops::Bound::Included(&self.stop)
    }
}
