#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub use humanbyte::HumanByte;

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;

/// A new-type for byte sizes, providing convenient constructors, arithmetic operations, conversions,
/// and display.
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Default, HumanByte)]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
pub struct ByteScale(pub u64);

// Re-export for doc tests
#[doc(hidden)]
pub use self::ByteScale as _doc_ByteScale;

#[cfg(test)]
mod tests {
    use super::*;
    use humanbyte::{B, Format, KB, MB};

    #[test]
    fn test_arithmetic_op() {
        let mut x = ByteScale::mb(1);
        let y = ByteScale::kb(100);

        assert_eq!((x + y).as_u64(), 1_100_000u64);

        assert_eq!((x - y).as_u64(), 900_000u64);

        assert_eq!((x + (100 * 1000) as u64).as_u64(), 1_100_000);

        assert_eq!((x * 2u64).as_u64(), 2_000_000);

        x += y;
        assert_eq!(x.as_u64(), 1_100_000);
        x *= 2u64;
        assert_eq!(x.as_u64(), 2_200_000);
    }
    #[test]
    fn test_arithmetic_primitives() {
        let mut x = ByteScale::mb(1);

        assert_eq!((x + MB).as_u64(), 2_000_000);

        assert_eq!((x + MB as u32).as_u64(), 2_000_000);

        assert_eq!((x + KB as u16).as_u64(), 1_001_000);

        assert_eq!((x + B as u8).as_u64(), 1_000_001);

        assert_eq!((x - MB).as_u64(), 0);

        assert_eq!((x - MB as u32).as_u64(), 0);

        assert_eq!((x - KB as u32).as_u64(), 999_000);

        assert_eq!((x - B as u32).as_u64(), 999_999);

        x += MB;
        x += MB as u32;
        x += 10u16;
        x += 1u8;
        assert_eq!(x.as_u64(), 3_000_011);
    }

    #[test]
    fn test_comparison() {
        assert_eq!(ByteScale::mb(1), ByteScale::kb(1000));
        assert_eq!(ByteScale::mib(1), ByteScale::kib(1024));
        assert_ne!(ByteScale::mb(1), ByteScale::kib(1024));
        assert!(ByteScale::mb(1) < ByteScale::kib(1024));
        assert!(ByteScale::b(0) < ByteScale::tib(1));
    }

    macro_rules! assert_display {
        ($expected:expr, $bytescale:expr) => {
            assert_eq!($expected, format!("{}", $bytescale));
        };
    }

    #[test]
    fn test_display() {
        assert_display!("215 B", ByteScale::b(215));
        assert_display!("1.0 KiB", ByteScale::kib(1));
        assert_display!("301.0 KiB", ByteScale::kib(301));
        assert_display!("419.0 MiB", ByteScale::mib(419));
        assert_display!("518.0 GiB", ByteScale::gib(518));
        assert_display!("815.0 TiB", ByteScale::tib(815));
        assert_display!("609.0 PiB", ByteScale::pib(609));
    }

    #[test]
    fn test_display_alignment() {
        assert_eq!("|357 B     |", format!("|{:10}|", ByteScale(357)));
        assert_eq!("|     357 B|", format!("|{:>10}|", ByteScale(357)));
        assert_eq!("|357 B     |", format!("|{:<10}|", ByteScale(357)));
        assert_eq!("|  357 B   |", format!("|{:^10}|", ByteScale(357)));

        assert_eq!("|-----357 B|", format!("|{:->10}|", ByteScale(357)));
        assert_eq!("|357 B-----|", format!("|{:-<10}|", ByteScale(357)));
        assert_eq!("|--357 B---|", format!("|{:-^10}|", ByteScale(357)));
    }

    macro_rules! assert_to_string {
        ($expected:expr, $actual:expr, $si:expr) => {
            assert_eq!($expected.to_string(), $actual.to_string_as($si));
        };
    }

    #[test]
    fn test_to_string_as() {
        use humanbyte::Format;
        assert_to_string!("215 B", ByteScale::b(215), Format::IEC);
        assert_to_string!("215 B", ByteScale::b(215), Format::SI);

        assert_to_string!("1.0 KiB", ByteScale::kib(1), Format::IEC);
        assert_to_string!("1.0 kB", ByteScale::kib(1), Format::SI);

        assert_to_string!("293.9 KiB", ByteScale::kb(301), Format::IEC);
        assert_to_string!("301.0 kB", ByteScale::kb(301), Format::SI);

        assert_to_string!("1.0 MiB", ByteScale::mib(1), Format::IEC);
        assert_to_string!("1.0 MB", ByteScale::mib(1), Format::SI);

        assert_to_string!("1.9 GiB", ByteScale::mib(1907), Format::IEC);
        assert_to_string!("2.0 GB", ByteScale::mib(1908), Format::SI);

        assert_to_string!("399.6 MiB", ByteScale::mb(419), Format::IEC);
        assert_to_string!("419.0 MB", ByteScale::mb(419), Format::SI);

        assert_to_string!("482.4 GiB", ByteScale::gb(518), Format::IEC);
        assert_to_string!("518.0 GB", ByteScale::gb(518), Format::SI);

        assert_to_string!("741.2 TiB", ByteScale::tb(815), Format::IEC);
        assert_to_string!("815.0 TB", ByteScale::tb(815), Format::SI);

        assert_to_string!("540.9 PiB", ByteScale::pb(609), Format::IEC);
        assert_to_string!("609.0 PB", ByteScale::pb(609), Format::SI);
    }

    #[test]
    fn when_err() {
        // shortcut for writing test cases
        fn parse(s: &str) -> Result<ByteScale, humanbyte::ParseError> {
            s.parse::<ByteScale>()
        }

        // the error type chains with `?` in std contexts
        fn assert_error<E: std::error::Error>(_: &E) {}
        assert_error(&parse("oops").unwrap_err());

        assert!(parse("").is_err());
        assert!(parse("a124GB").is_err());
        assert!(parse("1.3 42.0 B").is_err());
        assert!(parse("1.3 ... B").is_err());
        // The original implementation did not account for the possibility that users may
        // use whitespace to visually separate digits, thus treat it as an error
        assert!(parse("1 000 B").is_err());
    }

    #[test]
    fn test_div() {
        let file_size = ByteScale::gib(4);
        let chunk_size = ByteScale::mib(64);
        assert_eq!(file_size / chunk_size, 64u64);
        assert_eq!(file_size / 4u64, ByteScale::gib(1));
        assert_eq!(file_size % chunk_size, ByteScale::b(0));

        let mut x = ByteScale::mb(10);
        x /= 2u64;
        assert_eq!(x, ByteScale::mb(5));
    }

    #[test]
    fn test_to_string_with_precision() {
        use humanbyte::Format;
        let x = ByteScale::tib(1);
        assert_eq!(x.to_string_with_precision(Format::IEC, 2), "1.00 TiB");
        assert_eq!(x.to_string_with_precision(Format::IEC, 0), "1 TiB");
        assert_eq!(
            ByteScale::kib(1) + 512u64,
            "1.500 KiB".parse::<ByteScale>().unwrap()
        );
    }

    #[test]
    fn test_standalone_parse() {
        // no newtype required
        assert_eq!(humanbyte::parse("1.5 KiB"), Ok(1536));
    }

    #[test]
    fn test_sum() {
        let sizes = [ByteScale::kib(1), ByteScale::kib(2), ByteScale::kib(3)];
        assert_eq!(sizes.iter().sum::<ByteScale>(), ByteScale::kib(6));
        assert_eq!(sizes.into_iter().sum::<ByteScale>(), ByteScale::kib(6));
    }

    #[test]
    fn test_exabytes() {
        assert_eq!(ByteScale::eib(1).as_u64(), 1_152_921_504_606_846_976);
        assert_eq!(ByteScale::eb(1).as_u64(), 1_000_000_000_000_000_000);
        // the display/parse roundtrip works at the top of the u64 range
        let max = ByteScale(u64::MAX);
        assert_display!("16.0 EiB", max);
        assert_eq!("15 EiB".parse::<ByteScale>().unwrap(), ByteScale::eib(15));
        // 16 EiB is exactly 2^64: one past u64::MAX, so it must error
        assert!("16 EiB".parse::<ByteScale>().is_err());
    }

    #[test]
    fn test_float_accessors() {
        assert_eq!(ByteScale::kib(1).as_kib(), 1.0);
        assert_eq!(ByteScale::kib(1).as_kb(), 1.024);
        assert_eq!(ByteScale::gib(3).as_mib(), 3072.0);
    }

    #[test]
    fn test_display_precision() {
        let x = ByteScale::kib(1) + 512u64;
        assert_eq!(format!("{x:.2}"), "1.50 KiB");
        assert_eq!(format!("{x:.0}"), "2 KiB");
        assert_eq!(format!("{x}"), "1.5 KiB");
        // precision composes with width/alignment
        assert_eq!(format!("|{x:>10.2}|"), "|  1.50 KiB|");
    }

    #[test]
    #[should_panic(expected = "byte size overflows u64")]
    fn test_constructor_overflow() {
        // 20,000 PB doesn't fit in u64
        let _ = ByteScale::pb(20_000);
    }

    #[test]
    fn test_default() {
        assert_eq!(ByteScale::b(0), ByteScale::default());
    }

    #[test]
    fn test_to_string() {
        assert_to_string!("609.0 PB", ByteScale::pb(609), Format::SI);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde() {
        use serde::{Deserialize, Serialize};
        use serde_json;
        use toml;

        #[derive(Serialize, Deserialize)]
        struct S {
            x: ByteScale,
        }

        let s: S = serde_json::from_str(r#"{ "x": "5 B" }"#).unwrap();
        assert_eq!(s.x, ByteScale(5));

        let s: S = serde_json::from_str(r#"{ "x": 1048576 }"#).unwrap();
        assert_eq!(s.x, "1 MiB".parse::<ByteScale>().unwrap());

        let s: S = toml::from_str(r#"x = "2.5 MiB""#).unwrap();
        assert_eq!(s.x, "2.5 MiB".parse::<ByteScale>().unwrap());

        // i64 MAX
        let s: S = toml::from_str(r#"x = "9223372036854775807""#).unwrap();
        assert_eq!(s.x, "9223372036854775807".parse::<ByteScale>().unwrap());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_with_plain_integers() {
        use std::collections::BTreeMap;

        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Config {
            #[serde(with = "humanbyte::serde")]
            buffer_size: usize,
            #[serde(with = "humanbyte::serde")]
            max_size: u64,
            #[serde(with = "humanbyte::serde::map_keys")]
            pools: BTreeMap<u64, String>,
        }

        let config: Config = serde_json::from_str(
            r#"{
                "buffer_size": "1.5 KiB",
                "max_size": 1048576,
                "pools": { "4 KiB": "small", "2 MiB": "large" }
            }"#,
        )
        .unwrap();
        assert_eq!(config.buffer_size, 1536);
        assert_eq!(config.max_size, 1_048_576);
        assert_eq!(config.pools[&4096], "small");
        assert_eq!(config.pools[&2_097_152], "large");

        // roundtrip: serializes human-readable, parses back to the same values
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains(r#""buffer_size":"1.5 KiB""#));
        assert!(json.contains(r#""4.0 KiB":"small""#));
        let back: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(back, config);

        // toml too
        let config: Config = toml::from_str(
            "buffer_size = \"2 KiB\"\nmax_size = \"1 MiB\"\n[pools]\n\"64 KiB\" = \"medium\"",
        )
        .unwrap();
        assert_eq!(config.buffer_size, 2048);
        assert_eq!(config.pools[&65536], "medium");
    }

    #[cfg(feature = "schemars")]
    #[test]
    fn test_json_schema() {
        let schema = schemars::schema_for!(ByteScale);
        let json = serde_json::to_value(&schema).unwrap();
        assert_eq!(
            json["type"],
            serde_json::json!(["string", "integer"]),
            "schema should accept both forms: {json}"
        );
    }
}
