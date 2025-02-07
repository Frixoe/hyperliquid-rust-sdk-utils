mod meta;
mod orderbook;
mod price;

pub use meta::*;
pub use orderbook::*;
pub use price::*;

use std::{collections::HashMap, fmt};

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};

pub type PriceIsBuyAndAsset = (f64, bool, String);
pub type NameToPriceMap = HashMap<String, Price>;
pub type CoinToOiValueMap = HashMap<String, f64>;
pub const BOLD_START_ANSI: &str = "\x1b[1m";
pub const BOLD_END_ANSI: &str = "\x1b[0m";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Pair {
    #[serde(deserialize_with = "parse_pair_to_name")]
    pub name: String,
    pub size: f64,
}

impl Pair {
    pub fn convert_to_name(&self, pair_to_name_map: &HashMap<String, String>) -> Self {
        Pair {
            name: pair_to_name_map
                .get(&self.name)
                .unwrap_or(&"".to_string())
                .to_string(),
            size: self.size,
        }
    }
}

// TODO: This is for the future. Need to deserialize from pair to name here.
fn parse_pair_to_name<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringToStringVisitor;

    impl<'de> Visitor<'de> for StringToStringVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a pair with a corresponding name in the API")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            // TODO: Deserialize into the actual name of the pair
            Ok(value.to_string())
        }
    }

    deserializer.deserialize_str(StringToStringVisitor)
}
