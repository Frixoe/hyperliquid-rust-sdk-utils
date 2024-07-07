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

#[derive(Clone, PartialEq, Debug)]
pub struct SpotContext {
    pub name: String,
    pub quote: SpotAssetContext,
    pub base: SpotAssetContext,
}

#[derive(Clone, PartialEq, Debug)]
pub struct SpotAssetContext {
    pub sz_decimals: u16,
    pub wei_decimals: u16,
    pub name: String,
    pub index: u16,
}

#[derive(Clone, PartialEq, Debug)]
pub struct PerpContext {
    pub name: String,
    pub sz_decimals: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Price {
    Spot { pair: String, price: f64, context: SpotContext },
    Perp { price: f64, context: PerpContext },
}

impl Price {
    pub fn new_spot(price: f64, context: SpotContext) -> Self {
        let pair = format!("{}/{}", context.quote.name, context.base.name);

        if price == 0.0 {
            return Price::Spot { pair, price, context };
        }

        Price::Spot {
            pair,
            price: Self::round_price(price, 8, context.quote.sz_decimals),
            context,
        }
    }

    pub fn new_perp(price: f64, context: PerpContext) -> Self {
        if price == 0.0 {
            return Price::Perp { price, context };
        }

        Price::Perp {
            price: Self::round_price(price, 6, context.sz_decimals),
            context,
        }
    }

    fn round_price(price: f64, max_decimals: u16, sz_decimals: u16) -> f64 {
        let order_of_magnitude = price.abs().log10().floor() as i32;
        let significant_digits = 5;

        // Determine the maximum number of decimal places allowed
        let max_decimal_places = max_decimals - sz_decimals;

        // Calculate needed decimal places to maintain 5 significant digits
        let needed_decimal_places = (significant_digits - order_of_magnitude - 1) as u16;

        // Determine actual decimal places, considering the maximum limit
        let actual_decimal_places = if needed_decimal_places > max_decimal_places {
            max_decimal_places
        } else {
            needed_decimal_places
        };

        // Format the number with the appropriate number of decimal places
        format!("{:.*}", actual_decimal_places as usize, price)
            .parse::<f64>()
            .unwrap()
    }

    pub fn get_value(&self) -> f64 {
        match self {
            Price::Spot { price, .. } => *price,
            Price::Perp { price, .. } => *price,
        }
    }

    /// .
    ///
    /// # Gets True Size
    ///
    /// Formats the size according to the asset's sz_decimals and any other info required
    pub fn get_true_size(&self, size: f64) -> f64 {
        match self {
            Price::Spot { context, .. } => {
                format!("{:.*}", context.quote.sz_decimals as usize, size)
                    .parse::<f64>()
                    .unwrap()
            }
            Price::Perp { context, .. } => format!("{:.*}", context.sz_decimals as usize, size)
                .parse::<f64>()
                .unwrap(),
        }
    }

    /// Receives the USDC size and converts into asset denominated size at the current price of the
    /// asset.
    /// Example:
    /// Buying 100 usdc worth ETH @ 3230.2
    ///
    /// price.get_asset_denom_size(100) would give 0.030957835428146865 then you return the
    /// formatted size
    pub fn get_asset_denom_size(&self, size: f64) -> f64 {
        let ad_size = size / self.get_value();
        self.get_true_size(ad_size)
    }

    pub fn get_true_price_for_asset(&self, price: f64) -> f64 {
        match self {
            Price::Spot { context, .. } => Self::round_price(price, 8, context.quote.sz_decimals),
            Price::Perp { context, .. } => Self::round_price(price, 6, context.sz_decimals),
        }
    }

    pub fn to_string(&self) -> String {
        format!("{:?}", self.get_value())
    }

    pub fn update_price(&mut self, new_price: f64) {
        match self {
            Price::Spot { price, .. } => *price = new_price,
            Price::Perp { price, .. } => *price = new_price,
        }
    }
}

impl std::fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Price::Spot { price, .. } => {
                writeln!(f, "{}", price)
            }
            Price::Perp { price, .. } => {
                writeln!(f, "{}", price)
            }
        }
    }
}

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
