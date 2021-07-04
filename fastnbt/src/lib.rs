//! fastnbt aims for fast parsing of NBT data from *Minecraft: Java Edition*.
//! This format is used by the game to store various things, such as the world
//! data and player inventories.
//!
//! * For documentation and examples of serde deserialization, see [`de`].
//! * For a `serde_json`-like `Value` type see [`Value`].
//! * For NBT array types see [`ByteArray`], [`IntArray`], and [`LongArray`].
//! * For 'zero-copy' NBT array types see [`borrow`].
//!
//! Both this and related crates are under one [fastnbt Github
//! repository](https://github.com/owengage/fastnbt)
//!
//! ```toml
//! [dependencies]
//! fastnbt = "0.18"
//! ```
//!
//! # Byte, Int and Long array types
//!
//! To support `Value` capturing all NBT tag information, this deserializer
//! produces the `ByteArray`, `IntArray` and `LongArray` NBT data as a map
//! containing the original NBT tag and the data. In order to capture these
//! types in your own structs, use the appropriate type in this crate:
//! [`ByteArray`], [`IntArray`] and [`LongArray`]. These types have an `iter()`
//! method similar to [`Vec`][`std::vec::Vec`].
//!
//! # Quick example
//!
//! This example demonstrates printing out a players inventory and ender chest
//! contents from the [player dat
//! files](https://minecraft.gamepedia.com/Player.dat_format) found in worlds.
//!
//! Here we
//! * use serde's renaming attributes to have rustfmt conformant field names,
//! * use lifetimes to save on some string allocations, and
//! * use the `Value` type to deserialize a field we don't know the exact
//!   structure of.
//!
//!```no_run
//! use fastnbt::error::Result;
//! use fastnbt::{de::from_bytes, Value};
//! use flate2::read::GzDecoder;
//! use serde::Deserialize;
//! use std::io::Read;
//!
//! #[derive(Deserialize, Debug)]
//! #[serde(rename_all = "PascalCase")]
//! struct PlayerDat<'a> {
//!     data_version: i32,
//!
//!     #[serde(borrow)]
//!     inventory: Vec<InventorySlot<'a>>,
//!     ender_items: Vec<InventorySlot<'a>>,
//! }
//!
//! #[derive(Deserialize, Debug)]
//! struct InventorySlot<'a> {
//!     id: &'a str,        // We avoid allocating a string here.
//!     tag: Option<Value>, // Also get the less structured properties of the object.
//!
//!     // We need to rename fields a lot.
//!     #[serde(rename = "Count")]
//!     count: i8,
//! }
//!
//! fn main() {
//!     let args: Vec<_> = std::env::args().skip(1).collect();
//!     let file = std::fs::File::open(args[0].clone()).unwrap();
//!
//!     // Player dat files are compressed with GZip.
//!     let mut decoder = GzDecoder::new(file);
//!     let mut data = vec![];
//!     decoder.read_to_end(&mut data).unwrap();
//!
//!     let player: Result<PlayerDat> = from_bytes(data.as_slice());
//!
//!     println!("{:#?}", player);
//! }
//! ```
//!
//! # `Read` based parser
//!
//! A lower level parser also exists in the `stream` module that only requires
//! the `Read` trait on the input. This parser however doesn't support
//! deserializing to Rust objects directly.
//!

use serde::Deserialize;

pub mod borrow;
pub mod de;
pub mod error;
pub mod stream;

mod arrays;

pub use arrays::*;

pub(crate) mod de_arrays;

use std::{collections::HashMap, convert::TryFrom};

/// An NBT tag. This does not carry the value or the name of the data.
#[derive(Deserialize, Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum Tag {
    /// Represents the end of a Compound object.
    End = 0,
    /// Equivalent to i8.
    Byte = 1,
    /// Equivalent to i16.
    Short = 2,
    /// Equivalent to i32.
    Int = 3,
    /// Equivalent to i64
    Long = 4,
    /// Equivalent to f32.
    Float = 5,
    /// Equivalent to f64.
    Double = 6,
    /// Represents as array of Byte (i8).
    ByteArray = 7,
    /// Represents a Unicode string.
    String = 8,
    /// Represents a list of other objects, elements are not required to be the same type.
    List = 9,
    /// Represents a struct-like structure.
    Compound = 10,
    /// Represents as array of Int (i32).
    IntArray = 11,
    /// Represents as array of Long (i64).
    LongArray = 12,
}

pub(crate) const BYTE_ARRAY_TAG: u8 = 7;
pub(crate) const INT_ARRAY_TAG: u8 = 11;
pub(crate) const LONG_ARRAY_TAG: u8 = 12;

/// Value is a complete NBT value. It owns it's data. The Byte, Short, Int and
/// Long NBT types are all deserialized into `i64`. Compounds and Lists are
/// resursively deserialized.
///
/// ```no_run
/// # use fastnbt::Value;
/// # use fastnbt::error::Result;
/// # use std::collections::HashMap;
/// #
/// # fn main() -> Result<()> {
/// #   let mut buf = vec![];
///     let compound: HashMap<String, Value> = fastnbt::de::from_bytes(buf.as_slice())?;
///     match compound["DataVersion"] {
///         Value::Int(ver) => println!("Version: {}", ver),
///         _ => {},
///     }
///     println!("{:#?}", compound);
/// #   Ok(())
/// # }
/// ```
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Value {
    #[serde(deserialize_with = "strict_i8")]
    Byte(i8),
    #[serde(deserialize_with = "strict_i16")]
    Short(i16),
    #[serde(deserialize_with = "strict_i32")]
    Int(i32),
    Long(i64),
    Double(f64),
    Float(f32),
    String(String),
    ByteArray(ByteArray),
    IntArray(IntArray),
    LongArray(LongArray),
    List(Vec<Value>),
    Compound(HashMap<String, Value>),
}

fn strict_i8<'de, D>(de: D) -> std::result::Result<i8, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    struct StrictI8Visitor;
    impl<'de> serde::de::Visitor<'de> for StrictI8Visitor {
        type Value = i8;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "expecting exactly i8")
        }

        fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v)
        }
    }

    de.deserialize_i8(StrictI8Visitor)
}

fn strict_i16<'de, D>(de: D) -> std::result::Result<i16, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    struct Stricti16Visitor;
    impl<'de> serde::de::Visitor<'de> for Stricti16Visitor {
        type Value = i16;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "expecting exactly i16")
        }

        fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v)
        }
    }

    de.deserialize_i16(Stricti16Visitor)
}

fn strict_i32<'de, D>(de: D) -> std::result::Result<i32, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    struct Stricti32Visitor;
    impl<'de> serde::de::Visitor<'de> for Stricti32Visitor {
        type Value = i32;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "expecting exactly i32")
        }

        fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v)
        }
    }

    de.deserialize_i32(Stricti32Visitor)
}

// Crates exist to generate this code for us, but would add to our compile
// times, so we instead right it out manually, the tags will very rarely change
// so isn't a massive burden, but saves a significant amount of compile time.
impl TryFrom<u8> for Tag {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, ()> {
        use Tag::*;
        Ok(match value {
            0 => End,
            1 => Byte,
            2 => Short,
            3 => Int,
            4 => Long,
            5 => Float,
            6 => Double,
            7 => ByteArray,
            8 => String,
            9 => List,
            10 => Compound,
            11 => IntArray,
            12 => LongArray,
            13..=u8::MAX => return Err(()),
        })
    }
}

impl From<Tag> for u8 {
    fn from(tag: Tag) -> Self {
        match tag {
            Tag::End => 0,
            Tag::Byte => 1,
            Tag::Short => 2,
            Tag::Int => 3,
            Tag::Long => 4,
            Tag::Float => 5,
            Tag::Double => 6,
            Tag::ByteArray => 7,
            Tag::String => 8,
            Tag::List => 9,
            Tag::Compound => 10,
            Tag::IntArray => 11,
            Tag::LongArray => 12,
        }
    }
}

#[cfg(test)]
mod test;

#[derive(Debug, Clone, Copy)]
pub(crate) struct CompTag<const N: u8>;

impl<'de, const N: u8> Deserialize<'de> for CompTag<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let tag: u8 = Deserialize::deserialize(deserializer)?;
        if tag != N {
            Err(serde::de::Error::custom("unexpected array type"))
        } else {
            Ok(Self)
        }
    }
}
