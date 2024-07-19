use std::collections::HashMap;

use ethers::types::H128;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::types::{Meta, NameToPriceMap, Price, SpotAssetMeta};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SpotMeta {
    universe: Vec<UniverseData>,
    tokens: Vec<UniverseTokensData>,
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

impl SpotMeta {
    fn get_index_to_name_map(&self) -> HashMap<u16, String> {
        self.tokens
            .iter()
            .map(|info| (info.index, info.name.clone()))
            .collect()
    }

    pub fn get_spot_price_data(self, prices: HashMap<String, f64>) -> SpotPriceData {
        let res: NameToPriceMap = self.universe
            .iter()
            .map(|uni| {
                let price = prices[&uni.name];

                let quote_spot_context: SpotAssetMeta = self
                    .tokens
                    .iter()
                    .find_map(|token| {
                        if token.index == uni.tokens[0] {
                            Some(SpotAssetMeta {
                                sz_decimals: token.sz_decimals,
                                wei_decimals: token.wei_decimals,
                                index: token.index,
                                name: token.name.clone(),
                            })
                        } else {
                            None
                        }
                    })
                    .unwrap();

                let base_spot_context: SpotAssetMeta = self
                    .tokens
                    .iter()
                    .find_map(|token| {
                        if token.index == uni.tokens[1] {
                            Some(SpotAssetMeta {
                                sz_decimals: token.sz_decimals,
                                wei_decimals: token.wei_decimals,
                                index: token.index,
                                name: token.name.clone(),
                            })
                        } else {
                            None
                        }
                    })
                    .unwrap();

                (
                    uni.name.clone(),
                    Price::new_spot(
                        price,
                        Meta::Spot {
                            name: uni.name.clone(),
                            quote: quote_spot_context,
                            base: base_spot_context,
                        },
                    ),
                )
            })
            .collect();

        SpotPriceData {
            meta: self,
            map: res
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpotPriceData {
    meta: SpotMeta,
    pub map: NameToPriceMap
}

impl SpotPriceData {
    pub fn get_pair_to_raw_price_map(&self) -> HashMap<String, f64> {
        let index_to_name: HashMap<u16, String> = self.meta.get_index_to_name_map();

        self.meta
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

                let price = if let Some(price) = self.map.get(&val.name) {
                    price.get_value()
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
        let index_to_name: HashMap<u16, String> = self.meta.get_index_to_name_map();

        self.meta
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

    pub fn update(&mut self, price_map: HashMap<String, f64>) {
        for (name, price) in self.map.iter_mut() {
            price.update_price(price_map[name])
        }
    }

    pub fn get_price_from_pair(&self, pair: String) -> f64 {
        let index_to_name: HashMap<u16, String> = self.meta.get_index_to_name_map();

        self.meta
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
                if let Some(price) = self.map.get(&val.name) {
                    price.get_value()
                } else {
                    0.0
                }
            })
            .next()
            .unwrap()
    }
}
