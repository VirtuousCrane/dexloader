use serde::{de::Error, Deserialize, Deserializer};
use std::str::FromStr;

pub fn deserialize_to_f32<'de, D>(deserializer: D) -> Result<f32, D::Error>
    where
        D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    f32::from_str(s).map_err(D::Error::custom)
}