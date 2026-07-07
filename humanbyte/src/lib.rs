#![no_std]

//! Common types and functions for byte size handling
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

/// Re-export of the `serde` crate for use by derive-generated code.
#[cfg(feature = "serde")]
#[doc(hidden)]
pub use ::serde as serde_crate;

/// Re-export of the `schemars` crate for use by derive-generated code.
#[cfg(feature = "schemars")]
#[doc(hidden)]
pub use ::schemars as schemars_crate;

// Re-export necessary types to avoid users needing explicit extern crate declarations
pub use alloc::borrow::Cow;
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
/// bytes size for 1 exabyte
pub const EB: u64 = 1_000_000_000_000_000_000;

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
/// bytes size for 1 exbibyte
pub const EIB: u64 = 1_152_921_504_606_846_976;

/// IEC (binary) units.
///
/// See <https://en.wikipedia.org/wiki/Kilobyte>.
const UNITS_IEC: &str = "KMGTPE";
/// SI (decimal) units.
///
/// See <https://en.wikipedia.org/wiki/Kilobyte>.
const UNITS_SI: &str = "kMGTPE";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Format {
    #[default]
    IEC,
    SI,
}

/// Formats `bytes` as a human-readable string with one decimal, e.g. `"1.5 KiB"`.
pub fn to_string(bytes: u64, format: Format) -> String {
    to_string_with_precision(bytes, format, 1)
}

/// Formats `bytes` as a human-readable string with `precision` decimals,
/// e.g. `to_string_with_precision(1 << 40, Format::IEC, 2)` is `"1.00 TiB"`.
pub fn to_string_with_precision(bytes: u64, format: Format, precision: usize) -> String {
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
            "{:.*} {}{}",
            precision,
            (bytes as f64 / unit.pow(exp) as f64),
            unit_prefix[(exp - 1) as usize] as char,
            unit_suffix
        )
    }
}

/// Parses a human-readable byte size string into a byte count, e.g. `"1.5 KiB"` to `1536`.
///
/// Accepts a plain integer, or a number followed by an optional SI/IEC unit
/// (case-insensitive): `"1024"`, `"1.5 KB"`, `"2MiB"`, `"3 g"`.
pub fn parse(value: &str) -> Result<u64, ParseError> {
    if let Ok(v) = value.parse::<u64>() {
        return Ok(v);
    }
    let number = take_while(value, |c| c.is_ascii_digit() || c == '.');
    match number.parse::<f64>() {
        Ok(v) => {
            let suffix = skip_while(&value[number.len()..], char::is_whitespace);
            match suffix.parse::<Unit>() {
                // Use exact integer arithmetic when the number has no fractional
                // part: f64 only has a 53-bit mantissa, so byte counts at or above
                // 2^53 would otherwise be silently rounded (e.g.
                // "9007199254740993B" to ...992).
                Ok(u) if !number.contains('.') => match number.parse::<u64>() {
                    Ok(n) => n.checked_mul(u64::from(u)).ok_or_else(|| {
                        ParseError(format!("{:?} overflows a u64 byte count", value))
                    }),
                    Err(_) => Ok((v * u64::from(u) as f64) as u64),
                },
                Ok(u) => Ok((v * u64::from(u) as f64) as u64),
                Err(error) => Err(ParseError(format!(
                    "couldn't parse {:?} into a known SI unit, {}",
                    suffix, error
                ))),
            }
        }
        Err(error) => Err(ParseError(format!(
            "couldn't parse {:?} into a byte size, {}",
            value, error
        ))),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Byte,
    // power of tens
    KiloByte,
    MegaByte,
    GigaByte,
    TeraByte,
    PetaByte,
    ExaByte,
    // power of twos
    KibiByte,
    MebiByte,
    GibiByte,
    TebiByte,
    PebiByte,
    ExbiByte,
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
            Unit::ExaByte => EB,
            // power of twos
            Unit::KibiByte => KIB,
            Unit::MebiByte => MIB,
            Unit::GibiByte => GIB,
            Unit::TebiByte => TIB,
            Unit::PebiByte => PIB,
            Unit::ExbiByte => EIB,
        }
    }
}

impl FromStr for Unit {
    type Err = ParseError;

    fn from_str(unit: &str) -> Result<Self, Self::Err> {
        match unit.to_lowercase().as_str() {
            "b" => Ok(Self::Byte),
            // power of tens
            "k" | "kb" => Ok(Self::KiloByte),
            "m" | "mb" => Ok(Self::MegaByte),
            "g" | "gb" => Ok(Self::GigaByte),
            "t" | "tb" => Ok(Self::TeraByte),
            "p" | "pb" => Ok(Self::PetaByte),
            "e" | "eb" => Ok(Self::ExaByte),
            // power of twos
            "ki" | "kib" => Ok(Self::KibiByte),
            "mi" | "mib" => Ok(Self::MebiByte),
            "gi" | "gib" => Ok(Self::GibiByte),
            "ti" | "tib" => Ok(Self::TebiByte),
            "pi" | "pib" => Ok(Self::PebiByte),
            "ei" | "eib" => Ok(Self::ExbiByte),
            _ => Err(ParseError(format!("couldn't parse unit of {:?}", unit))),
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

/// Serde support for plain integer fields, without declaring a newtype.
///
/// Serializes as a human-readable string (`"1.5 KiB"`) in human-readable
/// formats (JSON, TOML, ...) and as a raw integer in binary formats.
/// Deserializes from either form.
///
/// ```ignore
/// #[derive(Serialize, Deserialize)]
/// struct Config {
///     #[serde(with = "humanbyte::serde")]
///     buffer_size: usize,
///     #[serde(with = "humanbyte::serde::map_keys")]
///     pools: BTreeMap<u64, PoolConfig>,
/// }
/// ```
#[cfg(feature = "serde")]
pub mod serde {
    use ::serde::{Deserializer, Serializer, de};

    use crate::{Format, to_string};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Copy + TryInto<u64>,
        S: Serializer,
    {
        let value: u64 = (*value)
            .try_into()
            .map_err(|_| ::serde::ser::Error::custom("byte size doesn't fit in u64"))?;
        if serializer.is_human_readable() {
            serializer.serialize_str(&to_string(value, Format::IEC))
        } else {
            serializer.serialize_u64(value)
        }
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: TryFrom<u64>,
        D: Deserializer<'de>,
    {
        let value = if deserializer.is_human_readable() {
            deserializer.deserialize_any(ByteVisitor)?
        } else {
            deserializer.deserialize_u64(ByteVisitor)?
        };
        T::try_from(value).map_err(|_| de::Error::custom("byte size overflows target type"))
    }

    struct ByteVisitor;

    impl de::Visitor<'_> for ByteVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            formatter.write_str("an integer or a byte size string")
        }

        fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
            u64::try_from(value).map_err(|_| {
                E::invalid_value(de::Unexpected::Signed(value), &"a non-negative integer")
            })
        }

        fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
            Ok(value)
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
            crate::parse(value)
                .map_err(|_| E::invalid_value(de::Unexpected::Str(value), &"a byte size string"))
        }
    }

    /// Like the parent module, but for `BTreeMap`s keyed by byte sizes.
    pub mod map_keys {
        use alloc::collections::BTreeMap;

        use ::serde::ser::SerializeMap;
        use ::serde::{Deserialize, Deserializer, Serialize, Serializer, de};

        use super::{ByteVisitor, Format, to_string};

        pub fn serialize<K, V, S>(map: &BTreeMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
        where
            K: Copy + TryInto<u64>,
            V: Serialize,
            S: Serializer,
        {
            let human_readable = serializer.is_human_readable();
            let mut ser = serializer.serialize_map(Some(map.len()))?;
            for (key, value) in map {
                let key: u64 = (*key)
                    .try_into()
                    .map_err(|_| ::serde::ser::Error::custom("byte size doesn't fit in u64"))?;
                if human_readable {
                    ser.serialize_entry(&to_string(key, Format::IEC), value)?;
                } else {
                    ser.serialize_entry(&key, value)?;
                }
            }
            ser.end()
        }

        pub fn deserialize<'de, K, V, D>(deserializer: D) -> Result<BTreeMap<K, V>, D::Error>
        where
            K: TryFrom<u64> + Ord,
            V: Deserialize<'de>,
            D: Deserializer<'de>,
        {
            struct Key<K>(K);

            impl<'de, K: TryFrom<u64>> Deserialize<'de> for Key<K> {
                fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                    let value = if deserializer.is_human_readable() {
                        deserializer.deserialize_any(ByteVisitor)?
                    } else {
                        deserializer.deserialize_u64(ByteVisitor)?
                    };
                    K::try_from(value)
                        .map(Key)
                        .map_err(|_| de::Error::custom("byte size overflows target type"))
                }
            }

            struct MapVisitor<K, V>(core::marker::PhantomData<(K, V)>);

            impl<'de, K, V> de::Visitor<'de> for MapVisitor<K, V>
            where
                K: TryFrom<u64> + Ord,
                V: Deserialize<'de>,
            {
                type Value = BTreeMap<K, V>;

                fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    formatter.write_str("a map keyed by byte sizes")
                }

                fn visit_map<A: de::MapAccess<'de>>(
                    self,
                    mut access: A,
                ) -> Result<Self::Value, A::Error> {
                    let mut map = BTreeMap::new();
                    while let Some((Key(key), value)) = access.next_entry::<Key<K>, V>()? {
                        map.insert(key, value);
                    }
                    Ok(map)
                }
            }

            deserializer.deserialize_map(MapVisitor(core::marker::PhantomData))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(parse("1024"), Ok(1024));
        assert_eq!(parse("1 KiB"), Ok(1024));
        assert_eq!(parse("1.5KiB"), Ok(1536));
        assert_eq!(parse("2 mb"), Ok(2_000_000));
        assert_eq!(parse("3 g"), Ok(3_000_000_000));
        assert_eq!(parse("1 EiB"), Ok(EIB));
        assert_eq!(parse("2 eb"), Ok(2 * EB));
        assert!(parse("").is_err());
        assert!(parse("1.5 XB").is_err());
    }

    #[test]
    fn test_parse_integer_exactness() {
        // 2^53 + 1 is not representable as f64; the suffix path must not
        // round it (upstream bytesize#171)
        assert_eq!(parse("9007199254740993B"), Ok(9_007_199_254_740_993));
        assert_eq!(parse("9007199254740993 B"), Ok(9_007_199_254_740_993));
        assert_eq!(parse("18446744073709551615B"), Ok(u64::MAX));
        // integer overflow errors instead of silently wrapping/saturating
        assert!(parse("20000 PB").is_err());
        // fractional values still take the f64 path
        assert_eq!(parse("1.5 KiB"), Ok(1536));
    }

    #[test]
    fn test_display_parse_roundtrip() {
        // everything we can display must parse back (u64::MAX shows as EiB)
        for bytes in [0, 1, 999, 1024, KIB, MIB, GIB, TIB, PIB, EIB, u64::MAX] {
            for format in [Format::IEC, Format::SI] {
                let s = to_string(bytes, format);
                assert!(parse(&s).is_ok(), "couldn't parse back {s:?}");
            }
        }
    }

    #[test]
    fn test_precision() {
        assert_eq!(to_string_with_precision(TIB, Format::IEC, 2), "1.00 TiB");
        assert_eq!(to_string_with_precision(1536, Format::IEC, 0), "2 KiB");
        assert_eq!(to_string_with_precision(1536, Format::IEC, 3), "1.500 KiB");
        // sub-unit values are plain byte counts regardless of precision
        assert_eq!(to_string_with_precision(215, Format::IEC, 2), "215 B");
        assert_eq!(to_string(TIB, Format::IEC), "1.0 TiB");
    }
}
