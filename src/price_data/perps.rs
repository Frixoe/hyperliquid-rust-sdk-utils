use core::fmt;
use std::collections::HashMap;

use anyhow::Error;
use serde::{de::{self, Visitor}, Deserialize, Deserializer, Serialize};

use crate::types::{Pair, Price};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PerpsPriceData(First, Vec<PairPriceData>);

impl PerpsPriceData {
    pub async fn get_pair_usdc_value(&self, pair: &Pair) -> Result<Price, Error> {
        let price = *self.get_name_to_price_map().get(&pair.name).unwrap();

        Ok(Price::new_spot(price))
    }

    pub fn get_name_to_price_map(&self) -> HashMap<String, f64> {
        let universe = &self.0.universe;
        let prices = &self.1;

        let mut result: HashMap<String, f64> = HashMap::new();

        for i in 0..prices.len() {
            result.insert(universe[i].name.clone(), prices[i].mark_px);
        }

        result
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct First {
    pub universe: Vec<UniverseData>,
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

#[cfg(test)]
mod tests {
    use tokio::sync::watch;

    use crate::prices::Prices;

    use std::time::Duration;

    #[tokio::test]
    async fn prices_are_being_returned_on_the_channel_with_no_issues() -> Result<(), anyhow::Error>
    {
        println!("starting test");
        let prices = Prices::new()?;

        let (price_sender, price_recv) = watch::channel(
            prices
                .get_all_price_info()
                .await
                .unwrap()
                .get_name_to_price_map(),
        );

        tokio::spawn(async move {
            let _ = prices.start_sending(price_sender).await;
        });

        for _ in 0..5 {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        for _ in 0..10 {
            let _ = *price_recv.borrow().get("@1").unwrap();
        }

        Ok(())
    }
}
