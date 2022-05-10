//! This module contains utility functions to help external
//! crates function. Basically, it extends the functionality
//! of external crates.
extern crate serde;

use serde::{de::Error, Deserialize, Deserializer};
use std::str::FromStr;

/// A function to help serde parse a string as f32.
/// 
/// # Examples
/// ```
/// struct SomeStruct {
///     #[serde(deserialize_with = "deserialize_to_f32")]
///     float_inside_a_string: f32,
/// }
/// ```
pub fn deserialize_to_f32<'de, D>(deserializer: D) -> Result<f32, D::Error>
    where
        D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    f32::from_str(s).map_err(D::Error::custom)
}