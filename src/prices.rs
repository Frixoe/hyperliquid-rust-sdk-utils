use std::thread::sleep;

use anyhow::Error;
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Client, Url,
};
use serde_json::json;
use tokio::sync::watch;

use crate::{
    price_data::{perps::PerpsPriceData, spot::SpotPriceData},
    types::{NameToPriceMap, CoinToOiValueMap},
};

#[derive(Debug)]
pub struct Prices {
    client: Client,
}

impl Prices {
    pub fn new() -> Result<Self, Error> {
        let mut headers = HeaderMap::new();

        headers.append(
            CONTENT_TYPE,
            HeaderValue::from_str("application/json").unwrap(),
        );

        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()?;

        Ok(Prices { client })
    }

    pub async fn get_all_price_info(&self) -> Result<SpotPriceData, Error> {
        let data = json!({ "type": "spotMetaAndAssetCtxs" });

        let response = self
            .client
            .post(Url::parse("https://api-ui.hyperliquid.xyz/info")?)
            .json(&data)
            .send()
            .await?;

        let bytes = response.bytes().await?;

        // Deserializing this way seems to be more reliable
        let response = serde_json::from_slice::<SpotPriceData>(&bytes)?;

        Ok(response)
    }

    pub async fn start_sending(
        &self,
        sender: watch::Sender<NameToPriceMap>,
    ) -> Result<(), Error> {
        loop {
            let name_to_price_map = self.get_all_price_info().await?.get_name_to_price_map();

            sender.send(name_to_price_map)?;
            sleep(std::time::Duration::from_millis(800));
        }
    }

    pub async fn start_sending_perps(
        &self,
        sender: watch::Sender<NameToPriceMap>,
    ) -> Result<(), Error> {
        loop {
            let name_to_price_map = self.get_all_perps_info().await?.get_name_to_price_map();
            sender.send(name_to_price_map)?;
            sleep(std::time::Duration::from_millis(800));
        }
    }

    pub async fn start_sending_perps_oi(
        &self,
        sender: watch::Sender<CoinToOiValueMap>,
    ) -> Result<(), Error> {
        loop {
            let coin_to_oi_value_map = self.get_all_perps_info().await?.get_coin_to_oi_value_map();
            sender.send(coin_to_oi_value_map)?;
            sleep(std::time::Duration::from_millis(800));
        }
    }

    pub async fn start_sending_perps_oi_and_price(
        &self,
        price_sender: watch::Sender<NameToPriceMap>,
        oi_sender: watch::Sender<CoinToOiValueMap>,
    ) -> Result<(), Error> {
        loop {
            let all_perps_info = self.get_all_perps_info().await?;

            let coin_to_oi_value_map = all_perps_info.get_coin_to_oi_value_map();
            oi_sender.send(coin_to_oi_value_map)?;

            let name_to_price_map = all_perps_info.get_name_to_price_map();
            price_sender.send(name_to_price_map)?;

            sleep(std::time::Duration::from_millis(800));
        }
    }

    pub async fn get_all_perps_info(&self) -> Result<PerpsPriceData, Error> {
        let data = json!({ "type": "metaAndAssetCtxs" });

        let response = self
            .client
            .post(Url::parse("https://api.hyperliquid-testnet.xyz/info")?)
            .json(&data)
            .send()
            .await?;

        let bytes = response.bytes().await?;

        // Deserializing this way seems to be more reliable
        let response = serde_json::from_slice::<PerpsPriceData>(&bytes)?;

        Ok(response)
    }
}
