use crate::{Error as MyError, events_helper, Event, Erc721Event, Erc1155Event, EvmClient};
use std::{error::Error, time::Duration};
use tokio::time::sleep;

/// When the ERC721 event is fetched, the event will be exposed to the caller through this trait.
/// The caller needs to implement this trait and write the code on how to use the event.
/// The metadata is also passed along with it.
#[async_trait]
pub trait Erc721EventCallback: Send {
    /// The callback function
    async fn on_erc721_event(
        &mut self,
        event: Erc721Event,
        name: String,
        symbol: String,
        token_uri: String,
    );
}

/// When the ERC1155 event is fetched, the event will be exposed to the caller through this trait.
/// The caller needs to implement this trait and write the code on how to use the event.
/// The metadata is also passed along with it.
#[async_trait]
pub trait Erc1155EventCallback: Send {
    /// The callback function
    async fn on_erc1155_event(
        &mut self, 
        event: Erc1155Event, 
        token_uri: String
    );
}

/// Entry function for tracking events.
/// If you only need to track events, you can use this function directly.
pub async fn track_events(
    evm_client: &EvmClient,
    start_from: u64,
    step: u64,
    end_block: Option<u64>,
    erc721_cb: &mut dyn Erc721EventCallback,
    erc1155_cb: &mut dyn Erc1155EventCallback,
) {
    let mut from = start_from;
    loop {
                // let latest_block_number = evm_client.get_latest_block_number().await.unwrap();
        match evm_client.get_latest_block_number().await {
            Ok(latest_block_number) => {
                let to = std::cmp::min(from + step - 1, latest_block_number - 6);
                if let Some(end_block) = end_block {
                    if to > end_block {
                        break;
                    }
                }

                if to >= from {
                    info!("Scan in {} - {}({})", from, to, to - from + 1);

                            // let events = events_helper::get_events(&evm_client, from,
                            // to).await.unwrap();
                    match events_helper::get_events(&evm_client, from, to).await {
                        Ok(events) => {

                            info!("{} events found", events.len());

                            for event in events {
                                match event {
                                    Event::Erc721(e) => {
                                        process_erc721_event(evm_client, e, erc721_cb).await;
                                    },
                                    Event::Erc1155(e) => {
                                        process_erc1155_event(evm_client, e, erc1155_cb).await;
                                    }
                                }
                            }

                            from = to + 1;

                        }
                        Err(err) => {
                            process_err(err).await;
                        },
                    }
                } else {
                    debug!("Track events too fast, wait for 30 seconds.");
                    sleep(Duration::from_secs(30)).await;
                }
            }
            Err(err) => {
                error!("Encountered an error when get latest_block_number: {:?}, wait for 30 seconds.", err);
                sleep(Duration::from_secs(30)).await;
            }
        }
    }
}

async fn process_err(err: MyError) {
    error!("Encountered an error when get events: {:?}, wait for 30 seconds.", err);
    sleep(Duration::from_secs(30)).await;
}

async fn process_erc721_event(evm_client: &EvmClient, event: Erc721Event, callback: &mut dyn Erc721EventCallback) {
    let (name, symbol, token_uri) = get_erc721_metadata(evm_client, &event).await;

    // callback
    callback.on_erc721_event(
        event,
        name,
        symbol,
        token_uri,
    )
    .await;
}

async fn get_erc721_metadata(
    evm_client: &EvmClient,
    event: &Erc721Event,
) -> (String, String, String) {
    let name = evm_client.get_erc721_name(&event.address).await.unwrap_or("Unknown".to_owned());
    let symbol = evm_client.get_erc721_symbol(&event.address).await.unwrap_or("Unknown".to_owned());
    let token_uri = evm_client.get_erc721_token_uri(&event.address, &event.token_id).await.unwrap_or("Unknown".to_owned());
    (name, symbol, token_uri)
}

async fn process_erc1155_event(evm_client: &EvmClient, event: Erc1155Event, callback: &mut dyn Erc1155EventCallback) {
    let token_uri = get_erc1155_metadata(evm_client, &event).await;

    // callback
    callback.on_erc1155_event(event, token_uri).await
}

async fn get_erc1155_metadata(
    evm_client: &EvmClient,
    event: &Erc1155Event,
) -> String {
    let token_uri = evm_client.get_erc1155_token_uri(&event.address, &event.token_id).await.unwrap_or("Unknown".to_owned());
    token_uri
}
