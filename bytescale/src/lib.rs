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
        fn parse(s: &str) -> Result<ByteScale, String> {
            s.parse::<ByteScale>()
        }

        assert!(parse("").is_err());
        assert!(parse("a124GB").is_err());
        assert!(parse("1.3 42.0 B").is_err());
        assert!(parse("1.3 ... B").is_err());
        // The original implementation did not account for the possibility that users may
        // use whitespace to visually separate digits, thus treat it as an error
        assert!(parse("1 000 B").is_err());
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
}
