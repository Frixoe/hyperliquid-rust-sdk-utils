use std::{collections::HashMap, thread::sleep};

use anyhow::Error;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription};
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Client, Url,
};
use serde_json::json;
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver},
    watch,
};

use crate::{
    price_data::{
        perps::{PerpsMeta, PerpsPriceData},
        spot::{SpotMeta, SpotPriceData},
    },
    types::NameToPriceMap,
};

#[derive(Debug)]
pub struct Prices {
    client: Client,
    price_receiver: UnboundedReceiver<Message>,
}

impl Prices {
    pub async fn new() -> Result<Self, Error> {
        let mut info_client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await.unwrap();

        let (sender, receiver) = unbounded_channel();
        let _ = info_client
            .subscribe(Subscription::AllMids, sender)
            .await
            .unwrap();

        let mut headers = HeaderMap::new();

        headers.append(
            CONTENT_TYPE,
            HeaderValue::from_str("application/json").unwrap(),
        );

        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()?;

        Ok(Prices {
            client,
            price_receiver: receiver,
        })
    }

    pub async fn get_all_spot_meta(&self) -> Result<SpotMeta, Error> {
        let data = json!({ "type": "spotMeta" });

        let response = self
            .client
            .post(Url::parse("https://api-ui.hyperliquid.xyz/info")?)
            .json(&data)
            .send()
            .await?;

        let bytes = response.bytes().await?;

        // Deserializing this way seems to be more reliable
        let response = serde_json::from_slice::<SpotMeta>(&bytes)?;

        Ok(response)
    }

    pub async fn start_sending(&mut self, sender: watch::Sender<NameToPriceMap>) -> Result<(), Error> {
        let mut spot_price_data = self.get_spot_price_data().await?;

        loop {
            spot_price_data.update(self.get_all_prices().await?);
            let name_to_price_map = spot_price_data.map.clone();

            sender.send(name_to_price_map)?;
            sleep(std::time::Duration::from_millis(800));
        }
    }

    pub async fn start_sending_perps(
        &mut self,
        sender: watch::Sender<NameToPriceMap>,
    ) -> Result<(), Error> {
        let mut perps_price_data = self.get_perps_price_data().await?;

        loop {
            perps_price_data.update(self.get_all_prices().await?);

            let name_to_price_map = perps_price_data.map.clone();

            sender.send(name_to_price_map)?;
            sleep(std::time::Duration::from_millis(800));
        }
    }

    pub async fn get_all_perps_meta(&self) -> Result<PerpsMeta, Error> {
        let data = json!({ "type": "meta" });

        let response = self
            .client
            .post(Url::parse("https://api-ui.hyperliquid.xyz/info")?)
            //.post(Url::parse("https://api.hyperliquid-testnet.xyz/info")?)
            .json(&data)
            .send()
            .await?;

        let bytes = response.bytes().await?;

        // Deserializing this way seems to be more reliable
        let response = serde_json::from_slice::<PerpsMeta>(&bytes)?;

        Ok(response)
    }

    pub async fn get_all_prices(&mut self) -> anyhow::Result<HashMap<String, f64>> {
        let all_prices: HashMap<String, f64> =
            if let Some(Message::AllMids(all_mids)) = self.price_receiver.recv().await {
                all_mids
                    .data
                    .mids
                    .into_iter()
                    .map(|(k, v)| (k, v.parse::<f64>().unwrap_or(0.0_f64)))
                    .collect()
            } else {
                HashMap::new()
            };

        Ok(all_prices)
    }

    pub async fn get_perps_price_data(&mut self) -> anyhow::Result<PerpsPriceData> {
        Ok(self
            .get_all_perps_meta()
            .await?
            .get_perps_prices_data(self.get_all_prices().await?))
    }

    pub async fn get_spot_price_data(&mut self) -> anyhow::Result<SpotPriceData> {
        Ok(self
            .get_all_spot_meta()
            .await?
            .get_spot_price_data(self.get_all_prices().await?))
    }
}
