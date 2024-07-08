use core::fmt;
use std::collections::HashMap;

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::types::{Context, NameToPriceMap, Price};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PerpsMeta {
    universe: Vec<UniverseData>,
}

impl PerpsMeta {
    pub fn get_perps_prices_data(self, prices: HashMap<String, f64>) -> PerpsPriceData {
        let universe = &self.universe;

        let name_to_universe_map: HashMap<&str, &UniverseData> = self
            .universe
            .iter()
            .map(|uni| (uni.name.as_str(), uni))
            .collect();

        let mut result: HashMap<String, Price> = HashMap::new();

        for i in 0..universe.len() {
            let universe_data = name_to_universe_map[universe[i].name.as_str()];

            result.insert(
                universe[i].name.clone(),
                Price::new_perp(
                    prices[&universe_data.name],
                    Context::Perp {
                        name: universe[i].name.clone(),
                        sz_decimals: universe_data.sz_decimals,
                    },
                ),
            );
        }

        PerpsPriceData {
            map: result,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerpsPriceData {
    pub map: NameToPriceMap,
}

impl PerpsPriceData {
    pub fn update(&mut self, price_map: HashMap<String, f64>) {
        for (name, price) in self.map.iter_mut() {
            price.update_price(price_map[name])
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct UniverseData {
    pub name: String,
    pub sz_decimals: u16,
    pub max_leverage: u16,
    pub only_isolated: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PairPriceData {
    #[serde(deserialize_with = "parse_string_to_float")]
    pub funding: f64,

    #[serde(deserialize_with = "parse_string_to_float")]
    pub open_interest: f64,

    #[serde(deserialize_with = "parse_string_to_float")]
    pub prev_day_px: f64,

    #[serde(deserialize_with = "parse_string_to_float")]
    pub day_ntl_vlm: f64,

    #[serde(deserialize_with = "parse_string_to_float")]
    pub premium: f64,

    #[serde(deserialize_with = "parse_string_to_float")]
    pub oracle_px: f64,

    #[serde(deserialize_with = "parse_string_to_float")]
    pub mark_px: f64,

    #[serde(deserialize_with = "parse_string_to_float")]
    pub mid_px: f64,

    pub impact_pxs: Option<Vec<String>>,
}

fn parse_string_to_float<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringToFloatVisitor;

    impl<'de> Visitor<'de> for StringToFloatVisitor {
        type Value = f64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string that can be parsed into a float or null")
        }

        // Handles valid string inputs.
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            value.parse::<f64>().map_err(de::Error::custom)
        }

        // Handles null input by providing a default value.
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(0.0) // default value when null is encountered
        }

        // Handles empty input as a default value.
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(0.0) // default value when null is encountered
        }
    }

    // Accepts either a valid string or null (handled as unit).
    deserializer.deserialize_any(StringToFloatVisitor)
}
