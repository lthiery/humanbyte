use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(HumanByte)]
pub fn humanbyte(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let mut combined = constructor_tokens(name);
    combined.extend(display_tokens(name));
    combined.extend(parse_tokens(name));
    combined.extend(ops_tokens(name));
    combined.extend(fromstr_tokens(name));
    if cfg!(feature = "serde") {
        combined.extend(serde_tokens(name));
    }
    if cfg!(feature = "schemars") {
        combined.extend(schemars_tokens(name));
    }
    TokenStream::from(combined)
}

#[proc_macro_derive(HumanByteConstructor)]
pub fn humanbyte_constructor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    TokenStream::from(constructor_tokens(&input.ident))
}

fn constructor_tokens(name: &syn::Ident) -> proc_macro2::TokenStream {
    // Define units with their multipliers and descriptions
    let units = vec![
        ("b", "1", "bytes"),
        ("kb", "::humanbyte::KB", "kilobytes"),
        ("kib", "::humanbyte::KIB", "kibibytes"),
        ("mb", "::humanbyte::MB", "megabytes"),
        ("mib", "::humanbyte::MIB", "mebibytes"),
        ("gb", "::humanbyte::GB", "gigabytes"),
        ("gib", "::humanbyte::GIB", "gibibytes"),
        ("tb", "::humanbyte::TB", "terabytes"),
        ("tib", "::humanbyte::TIB", "tebibytes"),
        ("pb", "::humanbyte::PB", "petabytes"),
        ("pib", "::humanbyte::PIB", "pebibytes"),
    ];

    // Generate methods
    let methods = units.iter().map(|(fn_name, multiplier, description)| {
        // Create an identifier for the method name
        let method_name = syn::Ident::new(fn_name, Span::call_site());

        // Parse the multiplier into an expression
        let multiplier_expr: syn::Expr = syn::parse_str(multiplier).unwrap();

        // Generate the documentation comment
        let doc_comment = format!(
            "Construct `{}` given an amount of {}.\n\nPanics if the total byte count overflows `u64`.",
            name, description
        );

        // Generate the method using quote!
        quote! {
            #[doc = #doc_comment]
            #[inline(always)]
            pub const fn #method_name(size: u64) -> Self {
                match size.checked_mul(#multiplier_expr) {
                    Some(bytes) => Self(bytes),
                    None => panic!("byte size overflows u64"),
                }
            }
        }
    });

    quote! {
        impl #name {
            #(#methods)*
        }

        impl From<u64> for #name {
            fn from(size: u64) -> #name {
                Self(size)
            }
        }
    }
}

#[proc_macro_derive(HumanByteOps)]
pub fn humanbyte_ops(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    TokenStream::from(ops_tokens(&input.ident))
}

fn ops_tokens(name: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        impl core::ops::Add<#name> for #name {
            type Output = #name;

            #[inline(always)]
            fn add(self, rhs: #name) -> #name {
                #name(self.0 + rhs.0)
            }
        }

        impl core::ops::AddAssign<#name> for #name {
            #[inline(always)]
            fn add_assign(&mut self, rhs: #name) {
                self.0 += rhs.0
            }
        }

        impl<T> core::ops::Add<T> for #name
        where
            T: Into<u64>,
        {
            type Output = #name;
            #[inline(always)]
            fn add(self, rhs: T) -> #name {
                #name(self.0 + (rhs.into()))
            }
        }

        impl<T> core::ops::AddAssign<T> for #name
        where
            T: Into<u64>,
        {
            #[inline(always)]
            fn add_assign(&mut self, rhs: T) {
                self.0 += rhs.into();
            }
        }

        impl core::ops::Sub<#name> for #name {
            type Output = #name;

            #[inline(always)]
            fn sub(self, rhs: #name) -> #name {
                #name(self.0 - rhs.0)
            }
        }

        impl core::ops::SubAssign<#name> for #name {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: #name) {
                self.0 -= rhs.0
            }
        }

        impl<T> core::ops::Sub<T> for #name
        where
            T: Into<u64>,
        {
            type Output = #name;

            #[inline(always)]
            fn sub(self, rhs: T) -> #name {
                #name(self.0 - (rhs.into()))
            }
        }

        impl<T> core::ops::SubAssign<T> for #name
        where
            T: Into<u64>,
        {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: T) {
                self.0 -= rhs.into();
            }
        }

        impl<T> core::ops::Mul<T> for #name
        where
            T: Into<u64>,
        {
            type Output = #name;
            #[inline(always)]
            fn mul(self, rhs: T) -> #name {
                #name(self.0 * rhs.into())
            }
        }

        impl<T> core::ops::MulAssign<T> for #name
        where
            T: Into<u64>,
        {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: T) {
                self.0 *= rhs.into();
            }
        }

        impl core::ops::Div<#name> for #name {
            /// Dividing two byte sizes yields a dimensionless count,
            /// e.g. `file_size / chunk_size` chunks.
            type Output = u64;

            #[inline(always)]
            fn div(self, rhs: #name) -> u64 {
                self.0 / rhs.0
            }
        }

        impl<T> core::ops::Div<T> for #name
        where
            T: Into<u64>,
        {
            type Output = #name;
            #[inline(always)]
            fn div(self, rhs: T) -> #name {
                #name(self.0 / rhs.into())
            }
        }

        impl<T> core::ops::DivAssign<T> for #name
        where
            T: Into<u64>,
        {
            #[inline(always)]
            fn div_assign(&mut self, rhs: T) {
                self.0 /= rhs.into();
            }
        }

        impl core::ops::Rem<#name> for #name {
            type Output = #name;

            #[inline(always)]
            fn rem(self, rhs: #name) -> #name {
                #name(self.0 % rhs.0)
            }
        }

        impl core::ops::Add<#name> for u64 {
            type Output = #name;
            #[inline(always)]
            fn add(self, rhs: #name) -> #name {
                #name(rhs.0 + self)
            }
        }

        impl core::ops::Add<#name> for u32 {
            type Output = #name;
            #[inline(always)]
            fn add(self, rhs: #name) -> #name {
                #name(rhs.0 + (self as u64))
            }
        }

        impl core::ops::Add<#name> for u16 {
            type Output = #name;
            #[inline(always)]
            fn add(self, rhs: #name) -> #name {
                #name(rhs.0 + (self as u64))
            }
        }

        impl core::ops::Add<#name> for u8 {
            type Output = #name;
            #[inline(always)]
            fn add(self, rhs: #name) -> #name {
                #name(rhs.0 + (self as u64))
            }
        }

        impl core::ops::Mul<#name> for u64 {
            type Output = #name;
            #[inline(always)]
            fn mul(self, rhs: #name) -> #name {
                #name(rhs.0 * self)
            }
        }

        impl core::ops::Mul<#name> for u32 {
            type Output = #name;
            #[inline(always)]
            fn mul(self, rhs: #name) -> #name {
                #name(rhs.0 * (self as u64))
            }
        }

        impl core::ops::Mul<#name> for u16 {
            type Output = #name;
            #[inline(always)]
            fn mul(self, rhs: #name) -> #name {
                #name(rhs.0 * (self as u64))
            }
        }

        impl core::ops::Mul<#name> for u8 {
            type Output = #name;
            #[inline(always)]
            fn mul(self, rhs: #name) -> #name {
                #name(rhs.0 * (self as u64))
            }
        }

        #[cfg(target_pointer_width = "64")]
        impl core::ops::Add<#name> for usize {
            type Output = #name;
            #[inline(always)]
            fn add(self, rhs: #name) -> #name {
                #name(rhs.0 + (self as u64))
            }
        }


        #[cfg(target_pointer_width = "64")]
        impl core::ops::Sub<#name> for usize {
            type Output = #name;
            #[inline(always)]
            fn sub(self, rhs: #name) -> #name {
                #name(self as u64 - rhs.0)
            }
        }

        #[cfg(target_pointer_width = "64")]
        impl core::ops::Mul<#name> for usize {
            type Output = #name;
            #[inline(always)]
            fn mul(self, rhs: #name) -> #name {
                #name(rhs.0 * (self as u64))
            }
        }

        impl #name {
            /// Provides `HumanByteRange` with explicit lower and upper bounds.
            pub fn range<I: Into<Self>>(start: I, stop: I) -> ::humanbyte::HumanByteRange<Self> {
                ::humanbyte::HumanByteRange::new(Some(start), Some(stop))
            }

            /// Provides `HumanByteRange` with explicit lower bound. Upper bound is set to `u64::MAX`.
            pub fn range_start<I: Into<Self>>(start: I) -> ::humanbyte::HumanByteRange<Self> {
                ::humanbyte::HumanByteRange::new(Some(start), None)
            }

            /// Provides `HumanByteRange` with explicit upper bound. Lower bound is set to `0`.
            pub fn range_stop<I: Into<Self>>(stop: I) -> ::humanbyte::HumanByteRange<Self> {
                ::humanbyte::HumanByteRange::new(None, Some(stop.into()))
            }
        }
    }
}

#[proc_macro_derive(HumanByteDisplay)]
pub fn humanbyte_display(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    TokenStream::from(display_tokens(&input.ident))
}

fn display_tokens(name: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        impl core::fmt::Display for #name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                f.pad(&::humanbyte::to_string(self.0, ::humanbyte::Format::IEC))
            }
        }

        impl core::fmt::Debug for #name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(f, "{}", self)
            }
        }
    }
}

#[proc_macro_derive(HumanByteFromStr)]
pub fn humanbyte_fromstr(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    TokenStream::from(fromstr_tokens(&input.ident))
}

fn fromstr_tokens(name: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        impl core::str::FromStr for #name {
            type Err = ::humanbyte::ParseError;

            fn from_str(value: &str) -> core::result::Result<Self, Self::Err> {
                ::humanbyte::parse(value).map(Self)
            }
        }
    }
}

#[proc_macro_derive(HumanByteParse)]
pub fn humanbyte_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    TokenStream::from(parse_tokens(&input.ident))
}

fn parse_tokens(name: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        impl #name {
            /// Returns the size as a string with an optional SI unit.
            #[inline(always)]
            pub fn to_string_as(&self, format: ::humanbyte::Format) -> ::humanbyte::String {
                ::humanbyte::to_string(self.0, format)
            }

            /// Returns the size as a string with the given number of decimals.
            #[inline(always)]
            pub fn to_string_with_precision(
                &self,
                format: ::humanbyte::Format,
                precision: usize,
            ) -> ::humanbyte::String {
                ::humanbyte::to_string_with_precision(self.0, format, precision)
            }

            /// Returns the inner u64 value.
            #[inline(always)]
            pub const fn as_u64(&self) -> u64 {
                self.0
            }

            /// Returns the inner value as usize.
            #[cfg(target_pointer_width = "64")]
            #[inline(always)]
            pub const fn as_usize(&self) -> usize {
                self.0 as usize
            }
        }
    }
}

#[proc_macro_derive(HumanByteSerde)]
pub fn humanbyte_serde(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    TokenStream::from(serde_tokens(&input.ident))
}

fn serde_tokens(name: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        impl<'de> ::humanbyte::serde_crate::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
            where
                D: ::humanbyte::serde_crate::Deserializer<'de>,
            {
                struct ByteSizeVisitor;

                impl<'de> ::humanbyte::serde_crate::de::Visitor<'de> for ByteSizeVisitor {
                    type Value = #name;

                    fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                        formatter.write_str("an integer or string")
                    }

                    fn visit_i64<E: ::humanbyte::serde_crate::de::Error>(self, value: i64) -> core::result::Result<Self::Value, E> {
                        if let Ok(val) = u64::try_from(value) {
                            Ok(#name(val))
                        } else {
                            Err(E::invalid_value(
                                ::humanbyte::serde_crate::de::Unexpected::Signed(value),
                                &"integer overflow",
                            ))
                        }
                    }

                    fn visit_u64<E: ::humanbyte::serde_crate::de::Error>(self, value: u64) -> core::result::Result<Self::Value, E> {
                        Ok(#name(value))
                    }

                    fn visit_str<E: ::humanbyte::serde_crate::de::Error>(self, value: &str) -> core::result::Result<Self::Value, E> {
                        if let Ok(val) = value.parse() {
                            Ok(val)
                        } else {
                            Err(E::invalid_value(
                                ::humanbyte::serde_crate::de::Unexpected::Str(value),
                                &"parsable string",
                            ))
                        }
                    }
                }

                if deserializer.is_human_readable() {
                    deserializer.deserialize_any(ByteSizeVisitor)
                } else {
                    deserializer.deserialize_u64(ByteSizeVisitor)
                }
            }
        }
        impl ::humanbyte::serde_crate::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
            where
                S: ::humanbyte::serde_crate::Serializer,
            {
                if serializer.is_human_readable() {
                    <str>::serialize(self.to_string().as_str(), serializer)
                } else {
                    self.0.serialize(serializer)
                }
            }
        }
    }
}

#[proc_macro_derive(HumanByteSchema)]
pub fn humanbyte_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    TokenStream::from(schemars_tokens(&input.ident))
}

fn schemars_tokens(name: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        impl ::humanbyte::schemars_crate::JsonSchema for #name {
            fn schema_name() -> ::humanbyte::Cow<'static, str> {
                ::humanbyte::Cow::Borrowed(stringify!(#name))
            }

            fn schema_id() -> ::humanbyte::Cow<'static, str> {
                ::humanbyte::Cow::Borrowed(concat!(module_path!(), "::", stringify!(#name)))
            }

            fn json_schema(
                _generator: &mut ::humanbyte::schemars_crate::SchemaGenerator,
            ) -> ::humanbyte::schemars_crate::Schema {
                ::humanbyte::schemars_crate::json_schema!({
                    "type": ["string", "integer"],
                    "description": "A byte size, as either a human-readable string (e.g. \"1.5 KiB\") or a number of bytes",
                })
            }
        }
    }
}
