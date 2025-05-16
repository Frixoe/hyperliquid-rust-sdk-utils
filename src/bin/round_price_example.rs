use hyperliquid_rust_sdk_utils::types::{Meta, Price};

#[tokio::main]
async fn main() {
    let btc_price = Price::new_perp(
        103020.32323,
        Meta::Perp {
            name: "BTC".to_string(),
            sz_decimals: 5,
            max_leverage: 100,
            only_isolated: None,
            is_delisted: None,
        },
    );

    dbg!(&btc_price);

    let price = btc_price.get_value_after_slippage(0.5, true);

    dbg!(price);
}
