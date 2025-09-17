use std::{collections::HashMap, thread::sleep};

use anyhow::Error;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription};
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver},
    watch,
};
use tracing::{error, info};

use crate::types::{NameToOrderbookMap, Orderbook};

/// Manages a websocket connection for streaming L2 orderbook data for a single coin
pub struct OrderbookStream {
    info_client: InfoClient,
    orderbook_receiver: UnboundedReceiver<Message>,
    sub_id: u32,
    coin: String,
}

impl OrderbookStream {
    /// Creates a new orderbook stream for the specified coin
    /// Initializes the websocket connection and subscribes to L2 book updates
    pub async fn new(coin: String) -> Result<Self, Error> {
        let mut info_client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await?;

        let (sender, receiver) = unbounded_channel();
        let sub_id = info_client
            .subscribe(Subscription::L2Book { coin: coin.clone() }, sender)
            .await?;

        Ok(OrderbookStream {
            info_client,
            orderbook_receiver: receiver,
            sub_id,
            coin,
        })
    }

    /// Starts processing the orderbook stream and sending updates through the watch channel
    /// Handles parsing of raw level data and maintains the current orderbook state
    pub async fn start_sending(
        &mut self,
        sender: watch::Sender<NameToOrderbookMap>,
    ) -> Result<(), Error> {
        let mut orderbook = Orderbook::new(self.coin.clone());
        let mut orderbook_map = HashMap::new();
        orderbook_map.insert(self.coin.clone(), orderbook.clone());

        let mut i = 0;

        // Every 20 hours, we will reconnect to avoid limits
        while i < 100_000 {
            match self.orderbook_receiver.recv().await {
                Some(msg) => match msg {
                    Message::L2Book(order_book) => {
                        // Parse bid levels from raw string data
                        let bids: Vec<(f64, f64)> = order_book.data.levels[0]
                            .iter()
                            .filter_map(|level| {
                                let px = level.px.parse().ok()?;
                                let sz = level.sz.parse().ok()?;
                                Some((px, sz))
                            })
                            .collect();

                        // Parse ask levels from raw string data
                        let asks: Vec<(f64, f64)> = order_book.data.levels[1]
                            .iter()
                            .filter_map(|level| {
                                let px = level.px.parse().ok()?;
                                let sz = level.sz.parse().ok()?;
                                Some((px, sz))
                            })
                            .collect();

                        orderbook.update_from_stream(bids, asks);
                        orderbook_map.insert(self.coin.clone(), orderbook.clone());
                        sender.send(orderbook_map.clone())?;
                    }
                    Message::NoData => {
                        error!("No orderbook data received");
                    }
                    Message::HyperliquidError(err) => {
                        error!("Hyperliquid error while getting orderbook data: {err:?}");
                    }
                    _ => {
                        tracing::debug!("Received message: {:?}", msg);
                    }
                },
                None => {
                    error!("Failed to receive orderbook data");
                    break;
                }
            }

            sleep(std::time::Duration::from_millis(800));
            i += 1;
        }

        Ok(())
    }

    /// Unsubscribes from the L2 book websocket stream
    pub async fn unsub(&mut self) -> anyhow::Result<()> {
        Ok(self.info_client.unsubscribe(self.sub_id).await?)
    }
}

/// Starts a background task that maintains an orderbook stream for the specified coin
/// Returns a receiver that provides real-time orderbook updates
/// The task will automatically reconnect if the connection is lost
pub async fn start_orderbook_stream_task(
    coin: String,
) -> anyhow::Result<watch::Receiver<NameToOrderbookMap>> {
    let (orderbook_sender, orderbook_recv) = watch::channel(HashMap::new());

    let coin_clone = coin.clone();
    tokio::spawn(async move {
        let o_s = orderbook_sender;
        loop {
            info!("orderbook_stream_task: Starting for {}", coin_clone);
            let mut new_stream = OrderbookStream::new(coin_clone.clone()).await.unwrap();
            match new_stream.start_sending(o_s.clone()).await {
                Ok(_) => {}
                Err(err) => {
                    error!("orderbook_stream_task: Error: {err:?}");
                }
            };
            info!("orderbook_stream_task: Resetting...");

            let _ = new_stream.unsub().await;
            sleep(std::time::Duration::from_secs(5));
        }
    });

    Ok(orderbook_recv)
}

#[cfg(test)]
mod tests {
    use std::sync::Once;
    use tracing_subscriber::{fmt, EnvFilter};

    use super::*;

    static INIT: Once = Once::new();

    /// Initialize tracing subscriber for tests with debug level enabled
    fn init_logger() {
        INIT.call_once(|| {
            fmt()
                .with_env_filter(
                    EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()),
                )
                .with_test_writer()
                .init();
        });
    }

    #[tokio::test]
    async fn orderbook_data_is_being_sent() -> anyhow::Result<()> {
        init_logger();

        let receiver = start_orderbook_stream_task("ETH".to_string()).await?;
        info!("Started orderbook stream for ETH");

        for i in 0..10 {
            let orderbooks = receiver.borrow().clone();
            info!("Received orderbooks: {:?}", orderbooks);
            // let orderbook = orderbooks.get(&"ETH".to_string()).unwrap();
            // info!(
            //     "ETH Orderbook - Best bid: {:?}, Best ask: {:?}",
            //     orderbook.best_bid(),
            //     orderbook.best_ask()
            // );
            sleep(std::time::Duration::from_secs(1));
        }

        Ok(())
    }
}
