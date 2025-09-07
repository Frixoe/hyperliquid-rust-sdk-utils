use serde::{Deserialize, Serialize};

use crate::types::Meta;
use core::fmt;

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub enum Price {
    #[default]
    None,
    Spot {
        price: f64,
        meta: Meta,
    },
    Perp {
        price: f64,
        meta: Meta,
    },
}

impl Price {
    pub fn from_meta(price: f64, meta: &Meta) -> Self {
        match meta {
            Meta::Spot { .. } => Price::new_spot(price, meta.clone()),
            Meta::Perp { .. } => Price::new_perp(price, meta.clone()),
        }
    }

    pub fn new_spot(price: f64, meta: Meta) -> Self {
        assert!(meta.is_spot());

        if price == 0.0 {
            return Price::Spot { price, meta };
        }

        Price::Spot {
            price: Self::round_price(price, 8, meta.get_sz_decimals()),
            meta,
        }
    }

    pub fn new_perp(price: f64, meta: Meta) -> Self {
        assert!(meta.is_perp());

        if price == 0.0 {
            return Price::Perp { price, meta };
        }

        Price::Perp {
            price: Self::round_price(price, 6, meta.get_sz_decimals()),
            meta,
        }
    }

    fn round_price(price: f64, max_decimals: u16, sz_decimals: u16) -> f64 {
        let order_of_magnitude = price.abs().log10().floor() as i32;
        let significant_digits = 5;

        // Determine the maximum number of decimal places allowed
        let max_decimal_places = max_decimals - sz_decimals;

        // Calculate needed decimal places to maintain 5 significant digits
        let needed_decimal_places = significant_digits - order_of_magnitude - 1;
        let needed_decimal_places = if needed_decimal_places < 0 {
            0
        } else {
            needed_decimal_places
        };

        // Determine actual decimal places, considering the maximum limit
        let actual_decimal_places = if needed_decimal_places > max_decimal_places as i32 {
            max_decimal_places as i32
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
            Price::None => 0.0_f64,
        }
    }

    pub fn get_value_after_slippage(&self, slippage: f64, is_buy: bool) -> f64 {
        let price = self.get_value();

        let after_slippage = if is_buy {
            price * (1.0 + slippage)
        } else {
            price * (1.0 - slippage)
        };

        match self {
            Price::None => 0.0_f64,
            Price::Spot { meta, .. } => {
                Self::round_price(after_slippage, 8, meta.get_sz_decimals())
            }
            Price::Perp { meta, .. } => {
                Self::round_price(after_slippage, 6, meta.get_sz_decimals())
            }
        }
    }

    /// .
    ///
    /// # Gets True Size
    ///
    /// Formats the size according to the asset's sz_decimals and any other info required
    pub fn get_true_size(&self, size: f64) -> f64 {
        match self {
            Price::Spot { meta, .. } => format!("{:.*}", meta.get_sz_decimals() as usize, size)
                .parse::<f64>()
                .unwrap(),
            Price::Perp { meta, .. } => format!("{:.*}", meta.get_sz_decimals() as usize, size)
                .parse::<f64>()
                .unwrap(),
            Price::None => 0.0_f64,
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

    pub fn get_asset_denom_size_at_price(&self, size: f64, price: f64) -> f64 {
        let ad_size = size / price;
        self.get_true_size(ad_size)
    }

    pub fn get_true_price_for_asset(&self, price: f64) -> f64 {
        match self {
            Price::Spot { meta, .. } => Self::round_price(price, 8, meta.get_sz_decimals()),
            Price::Perp { meta, .. } => Self::round_price(price, 6, meta.get_sz_decimals()),
            Price::None => 0.0_f64,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{:?}", self.get_value())
    }

    pub fn update_price(&mut self, new_price: f64) {
        match self {
            Price::Spot { price, meta } => {
                *price = Self::round_price(new_price, 8, meta.get_sz_decimals())
            }
            Price::Perp { price, meta } => {
                *price = Self::round_price(new_price, 6, meta.get_sz_decimals())
            }
            Price::None => (),
        }
    }

    pub fn from_new_price(self, new_price: f64) -> Price {
        match self {
            Price::Spot { meta, .. } => Price::new_spot(new_price, meta),
            Price::Perp { meta, .. } => Price::new_perp(new_price, meta),
            Price::None => Price::None,
        }
    }

    pub fn get_meta(&self) -> &Meta {
        match self {
            Price::None => panic!("Tried to get meta for no price..."),
            Price::Spot { meta, .. } => meta,
            Price::Perp { meta, .. } => meta,
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
            Price::None => writeln!(f, "0.0"),
        }
    }
}
