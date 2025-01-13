use serde::{Deserialize, Serialize};

/// Represents a single level in the orderbook with price and size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookLevel {
    pub price: f64,
    pub size: f64,
}

/// Represents the full orderbook state for a single coin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orderbook {
    /// Vector of bid levels sorted by price (highest to lowest)
    pub bids: Vec<OrderbookLevel>,
    /// Vector of ask levels sorted by price (lowest to highest)
    pub asks: Vec<OrderbookLevel>,
    /// The coin symbol this orderbook represents
    pub coin: String,
}

impl Orderbook {
    /// Creates a new empty orderbook for the given coin
    pub fn new(coin: String) -> Self {
        Orderbook {
            bids: Vec::new(),
            asks: Vec::new(),
            coin,
        }
    }

    /// Updates the orderbook with new bid and ask levels
    pub fn update(&mut self, bids: Vec<OrderbookLevel>, asks: Vec<OrderbookLevel>) {
        self.bids = bids;
        self.asks = asks;
    }

    /// Updates the orderbook from raw stream data where levels are (price, size) tuples
    pub fn update_from_stream(&mut self, bids: Vec<(f64, f64)>, asks: Vec<(f64, f64)>) {
        self.bids = bids
            .into_iter()
            .map(|(price, size)| OrderbookLevel { price, size })
            .collect();
        self.asks = asks
            .into_iter()
            .map(|(price, size)| OrderbookLevel { price, size })
            .collect();
    }

    /// Returns the best (highest) bid level if available
    pub fn best_bid(&self) -> Option<&OrderbookLevel> {
        self.bids.first()
    }

    /// Returns the best (lowest) ask level if available
    pub fn best_ask(&self) -> Option<&OrderbookLevel> {
        self.asks.first()
    }
}

/// Type alias for mapping coin symbols to their orderbooks
pub type NameToOrderbookMap = std::collections::HashMap<String, Orderbook>;
