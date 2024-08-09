use std::{collections::HashMap, thread::sleep};

use anyhow::Error;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription};
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Client, Url,
};
use serde_json::json;
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    watch,
};
use tracing::error;

use crate::{
    price_data::{
        perps::{PerpsMeta, PerpsPriceData},
        spot::{SpotMeta, SpotPriceData},
    },
    types::NameToPriceMap,
};

pub struct Prices {
    client: Client,
    info_client: InfoClient,
    price_sender: UnboundedSender<Message>,
    price_receiver: UnboundedReceiver<Message>,
    sub_id: u32,
}

impl Prices {
    pub async fn new() -> Result<Self, Error> {
        let mut info_client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await.unwrap();

        let (sender, receiver) = unbounded_channel();
        let sub_id = info_client
            .subscribe(Subscription::AllMids, sender.clone())
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
            info_client,
            price_receiver: receiver,
            price_sender: sender,
            sub_id,
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

    pub async fn start_sending(
        &mut self,
        sender: watch::Sender<NameToPriceMap>,
    ) -> Result<(), Error> {
        let mut spot_price_data = self.get_spot_price_data().await?;

        let mut i = 0;

        // Every 20 hours
        while i < 100_000 {
            if i >= 99_999 {
                match self.info_client.unsubscribe(self.sub_id).await {
                    Ok(_) => {
                        sleep(std::time::Duration::from_secs(2));

                        self.sub_id = self
                            .info_client
                            .subscribe(Subscription::AllMids, self.price_sender.clone())
                            .await
                            .unwrap();
                    }
                    Err(err) => {
                        error!("Received an error while unsubscribing from spot channel: {err:?}");
                        return Err(err.into());
                    }
                }

                i = 0;
            }
            spot_price_data.update(self.get_all_prices().await?);
            let name_to_price_map = spot_price_data.map.clone();

            sender.send(name_to_price_map)?;
            sleep(std::time::Duration::from_millis(800));

            i += 1;
        }

        Ok(())
    }

    pub async fn start_sending_perps(
        &mut self,
        sender: watch::Sender<NameToPriceMap>,
    ) -> Result<(), Error> {
        let mut perps_price_data = self.get_perps_price_data().await?;

        let mut i = 0;

        // Every 20 hours
        while i < 100_000 {
            if i >= 99_999 {
                match self.info_client.unsubscribe(self.sub_id).await {
                    Ok(_) => {
                        sleep(std::time::Duration::from_secs(2));

                        self.sub_id = self
                            .info_client
                            .subscribe(Subscription::AllMids, self.price_sender.clone())
                            .await
                            .unwrap();
                    }
                    Err(err) => {
                        error!("Received an error while unsubscribing from perps channel: {err:?}");
                        return Err(err.into());
                    }
                }

                i = 0;
            }

            perps_price_data.update(self.get_all_prices().await?);

            let name_to_price_map = perps_price_data.map.clone();

            sender.send(name_to_price_map)?;
            sleep(std::time::Duration::from_millis(800));

            i += 1;
        }

        Ok(())
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

    pub async fn unsub(&mut self) -> anyhow::Result<()> {
        Ok(self.info_client.unsubscribe(self.sub_id).await?)
    }
}

pub async fn start_perps_sender_task(
    mut prices: Prices,
) -> anyhow::Result<watch::Receiver<NameToPriceMap>> {
    // TODO: Start returning an Arc<Mutex<watch::Receiver<..>>> so that you can create a new
    // connection efficiently from within the tokio task and update across all threads.
    let (price_sender, price_recv) =
        watch::channel(prices.get_perps_price_data().await?.map.clone());

    tokio::spawn(async move {
        match prices.start_sending_perps(price_sender).await {
            Ok(it) => it,
            Err(_) => {}
        };
    });

    Ok(price_recv)
}

pub async fn start_spot_sender_task(
    mut prices: Prices,
) -> anyhow::Result<watch::Receiver<NameToPriceMap>> {
    let (price_sender, price_recv) =
        watch::channel(prices.get_spot_price_data().await?.map.clone());

    tokio::spawn(async move {
        match prices.start_sending(price_sender).await {
            Ok(it) => it,
            Err(_) => {}
        };
    });

    Ok(price_recv)
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use log::info;

    use crate::prices::{start_perps_sender_task, start_spot_sender_task, Prices};

    static INIT: Once = Once::new();

    fn init_logger() {
        INIT.call_once(|| {
            env_logger::builder().is_test(true).init();
        });
    }

    #[tokio::test]
    async fn perps_prices_are_being_sent() -> anyhow::Result<()> {
        init_logger();

        let prices = Prices::new().await?;
        let receiver = start_perps_sender_task(prices).await?;

        for _ in 0..100 {
            let prices = receiver.borrow().clone();
            let price = prices.get(&"ETH".to_string()).unwrap();
            info!("{:?}", price);
        }

        Ok(())
    }

    #[tokio::test]
    async fn spot_prices_are_being_sent() -> anyhow::Result<()> {
        init_logger();

        let prices = Prices::new().await?;
        let receiver = start_spot_sender_task(prices).await?;

        for _ in 0..100 {
            let prices = receiver.borrow().clone();
            let price = prices.get(&"@2".to_string()).unwrap();
            info!("{:?}", price);
        }

        Ok(())
    }
}
