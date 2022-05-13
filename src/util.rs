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

/// A function to help serde parse a string as Option<f32>.
/// 
/// # Examples
/// ```
/// struct SomeStruct {
///     #[serde(deserialize_with = "deserialize_to_option_f32")]
///     float_inside_a_string: Option<f32>,
/// }
/// ```
pub fn deserialize_to_option_f32<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
    where
        D: Deserializer<'de>,
{
    let res = Deserialize::deserialize(deserializer)
        .map(|x: Option<_>| {
            x.unwrap_or("".to_string())
        });
    let res = res.ok();
    match res {
        Some(s) => Ok(f32::from_str(&s).ok()),
        None => Ok(None)
    }
}

pub fn deserialize_title<'de, D> (deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer)
        .map(|x: Option<_>| {
            x.unwrap_or("".to_string())
        })
}