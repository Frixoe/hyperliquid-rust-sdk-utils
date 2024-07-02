use std::{collections::HashMap, fmt};

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};

pub type PriceIsBuyAndAsset = (f64, bool, String);
pub type NameToPriceMap = HashMap<String, f64>;
pub const BOLD_START_ANSI: &str = "\x1b[1m";
pub const BOLD_END_ANSI: &str = "\x1b[0m";

#[derive(Debug, PartialEq)]
pub enum Price {
    Spot(f64),
    Perp(f64),
}

impl Default for Price {
    fn default() -> Self {
        Price::Spot(0.0_f64)
    }
}

impl Price {
    pub fn new_spot(num: f64) -> Self {
        // Do calculation to make sure the price is in correct format
        if num == 0.0 {
            return Price::Spot(0.0_f64); // Handle zero separately
        }

        let order_of_magnitude = num.abs().log10().floor() as i32;
        let significant_digits = 5;

        // Determine the maximum number of decimal places allowed
        let max_decimal_places = 8;

        // Calculate needed decimal places to maintain 5 significant digits
        let needed_decimal_places = significant_digits - order_of_magnitude - 1;

        // Determine actual decimal places, considering the maximum limit
        let actual_decimal_places = if needed_decimal_places > max_decimal_places {
            max_decimal_places
        } else if needed_decimal_places < 0 {
            0
        } else {
            needed_decimal_places
        };

        // Format the number with the appropriate number of decimal places
        let value = format!("{:.*}", actual_decimal_places as usize, num)
            .parse::<f64>()
            .unwrap();

        Price::Spot(value)
    }

    pub fn new_perp(value: f64) -> Self {
        // TODO: Do calculation to make sure the price is in the correct format
        Price::Perp(value)
    }

    pub fn get_value(&self) -> f64 {
        match self {
            Price::Spot(num) => *num,
            Price::Perp(num) => *num,
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
