use core::fmt;
use std::collections::HashMap;

use anyhow::Error;
use ethers::types::H128;
use serde::{de::{self, Visitor}, Deserialize, Deserializer, Serialize};
use tracing::warn;

use crate::types::{Pair, Price};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SpotPriceData(First, Vec<PairPriceData>);

impl SpotPriceData {
    pub async fn get_pair_usdc_value(&self, pair: &Pair) -> Result<Price, Error> {
        let price = *self.get_name_to_price_map().get(&pair.name).unwrap();

        Ok(Price::new_spot(price))
    }

    pub fn get_pair_to_price_map(&self) -> HashMap<String, f64> {
        let index_to_name: HashMap<u16, String> = self.get_index_to_name_map();

        let name_to_price: HashMap<String, f64> = self.get_name_to_price_map();

        self.0
            .universe
            .iter()
            .map(|val| {
                let token_1_name = if let Some(name) = index_to_name.get(val.tokens.get(0).unwrap())
                {
                    name
                } else {
                    ""
                };
                let token_2_name = if let Some(name) = index_to_name.get(val.tokens.get(1).unwrap())
                {
                    name
                } else {
                    ""
                };

                let price = if let Some(price) = name_to_price.get(&val.name) {
                    *price
                } else {
                    warn!(
                        "There was an issue getting the price for the pair {}/{}",
                        token_1_name, token_2_name
                    );
                    0.0
                };

                (
                    format!("{}/{}", token_1_name, token_2_name).to_string(),
                    price,
                )
            })
            .collect()
    }

    pub fn get_pair_to_name_map(&self) -> HashMap<String, String> {
        let index_to_name: HashMap<u16, String> = self.get_index_to_name_map();

        self.0
            .universe
            .iter()
            .map(|val| {
                let token_1_name = if let Some(name) = index_to_name.get(val.tokens.get(0).unwrap())
                {
                    name
                } else {
                    ""
                };
                let token_2_name = if let Some(name) = index_to_name.get(val.tokens.get(1).unwrap())
                {
                    name
                } else {
                    ""
                };

                (
                    format!("{}/{}", token_1_name, token_2_name).to_string(),
                    val.name.clone(),
                )
            })
            .collect()
    }

    fn get_index_to_name_map(&self) -> HashMap<u16, String> {
        self.0
            .tokens
            .iter()
            .map(|info| (info.index, info.name.clone()))
            .collect()
    }

    pub fn get_name_to_price_map(&self) -> HashMap<String, f64> {
        self.1
            .iter()
            .map(|val| (val.coin.clone(), val.mark_px))
            .collect()
    }

    pub fn get_price_from_pair(&self, pair: String) -> f64 {
        let index_to_name: HashMap<u16, String> = self.get_index_to_name_map();

        let name_to_price: HashMap<String, f64> = self.get_name_to_price_map();

        self.0
            .universe
            .iter()
            .filter(|val| {
                let token_1_name = if let Some(name) = index_to_name.get(val.tokens.get(0).unwrap())
                {
                    name
                } else {
                    ""
                };
                let token_2_name = if let Some(name) = index_to_name.get(val.tokens.get(1).unwrap())
                {
                    name
                } else {
                    ""
                };

                format!("{}/{}", token_1_name, token_2_name) == pair
            })
            .map(|val| {
                if let Some(price) = name_to_price.get(&val.name) {
                    *price
                } else {
                    0.0
                }
            })
            .next()
            .unwrap()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct First {
    pub universe: Vec<UniverseData>,
    pub tokens: Vec<UniverseTokensData>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct UniverseData {
    pub tokens: [u16; 2],
    pub name: String,
    pub index: u16,
    pub is_canonical: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct UniverseTokensData {
    pub name: String,
    pub sz_decimals: u16,
    pub wei_decimals: u16,
    pub index: u16,
    pub token_id: H128,
    pub is_canonical: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PairPriceData {
    #[serde(deserialize_with = "parse_string_to_float")]
    pub prev_day_px: f64,

    #[serde(deserialize_with = "parse_string_to_float")]
    pub mark_px: f64,

    #[serde(deserialize_with = "parse_string_to_float")]
    pub mid_px: f64,

    pub coin: String,
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

    use super::*;
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
