[![Continuous Integration](https://github.com/lthiery/humanbyte/actions/workflows/rust.yaml/badge.svg)](https://github.com/lthiery/humanbyte/actions/workflows/rust.yaml)
[![Crates.io Version](https://img.shields.io/crates/v/humanbyte.svg)](https://crates.io/crates/humanbyte)

# HumanByte

HumanByte is a procedural macro crate for deriving `HumanByte` functions for new types of the form `NewType(u64)`. It is
inspired by the [`bytesize`][bytesize] crate (which is replicated here by our example [`bytescale`] crate).

It provides human-friendly way of constructing and displaying the type with byte units.

## Usage

Add this to your `Cargo.toml`:

### with serde support

```toml
[dependencies]
humanbyte = { version = "0.1", features = ["serde"] }
```

### without serde support

```toml
[dependencies]
humanbyte = { version = "0.1" }
```

### no_std compatible

```toml
[dependencies]
humanbyte = { version = "0.1", no-default-features = true }
```

Define your new type and derive `HumanByte` for it. This will derive all the necessary functions for your new type. You
can then use it like this:

```rust
use humanbyte_derive::HumanByte;

#[derive(HumanByte)]
pub struct NewType(u64);

fn main() {
    let size = NewType::kib(1);
    assert_eq!(size.to_string(), "1.0 KiB");
}
```

See the documentation for [bytescale][bytescale] to see more examples of what you can do with the HumanByte new type.

In addition, if you only want some of the HumanByte functions derived, you can use any of the following derives in an
a la carte fashion:
* HumanByteConstructor
* HumanByteDisplay
* HumanByteParse
* HumanByteOps
* HumanByteFromStr
* HumanByteSerde (requires the `serde` feature)
* HumanByteSchema (requires the `schemars` feature)

## Without a newtype

Plain `u64`/`usize` fields can use human-readable serde directly — no newtype required:

```rust,ignore
#[derive(Serialize, Deserialize)]
struct Config {
    #[serde(with = "humanbyte::serde")]
    buffer_size: usize,
    #[serde(with = "humanbyte::serde::map_keys")]
    pools: BTreeMap<u64, PoolConfig>,
}
```

And free functions mirror the derived methods:

```rust
assert_eq!(humanbyte::parse("1.5 KiB"), Ok(1536));
assert_eq!(humanbyte::to_string(1536, humanbyte::Format::IEC), "1.5 KiB");
assert_eq!(
    humanbyte::to_string_with_precision(1536, humanbyte::Format::IEC, 2),
    "1.50 KiB"
);
```

## JSON schema

With the `schemars` feature, derived types implement `schemars::JsonSchema` (accepting a
string like `"1.5 KiB"` or a raw byte count), so config types need no manual
`#[schemars(with = "String")]` annotations.

[bytescale]: https://docs.rs/bytescale/latest/bytescale
[bytesize]: https://docs.rs/bytesize/latest/bytesize
