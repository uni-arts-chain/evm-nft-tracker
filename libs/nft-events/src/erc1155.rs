//! This module is the entry point for tracking ERC1155.
use crate::{erc1155_evm, erc1155_evm::Erc1155Event, EvmClient};
use std::time::Duration;
use tokio::time::sleep;

/// When the ERC1155 event is fetched, the event will be exposed to the caller through this trait.
/// The caller needs to implement this trait and write the code on how to use the event.
/// The metadata is also passed along with it.
#[async_trait]
pub trait Erc1155EventCallback: Send {
    /// The callback function
    async fn on_erc1155_event(&mut self, event: Erc1155Event, token_uri: String);
}

/// Entry function for tracking ERC1155.
/// If you only need to track ERC1155, you can use this function directly.
pub async fn track_erc1155_events(
    evm_client: &EvmClient,
    start_from: u64,
    step: u64,
    end_block: Option<u64>,
    callback: &mut dyn Erc1155EventCallback,
) {
    let mut from = start_from;
    loop {
        match evm_client.get_latest_block_number().await {
            Ok(latest_block_number) => {
                let to = std::cmp::min(from + step - 1, latest_block_number - 6);
                if let Some(end_block) = end_block {
                    if to > end_block {
                        break;
                    }
                }

                if to >= from {
                    debug!("Scan for ERC1155 events in {} - {}({})", from, to, to - from + 1);
                    match erc1155_evm::get_erc1155_events(&evm_client, from, to).await {
                        Ok(events) => {

                            info!("{} ERC1155 events were scanned in {} - {}({})", events.len(), from, to, to - from + 1);

                            for event in events {
                                process_event(evm_client, event.clone(), callback).await;
                            }

                            from = to + 1;

                        }
                        Err(err) => {
                            error!("Encountered an error when get ERC1155 events: {:?}, wait for 30 seconds.", err);
                            sleep(Duration::from_secs(30)).await;
                        },
                    }
                } else {
                    debug!("Track ERC1155 events too fast, wait for 30 seconds.");
                    sleep(Duration::from_secs(30)).await;
                }
            }
            Err(err) => {
                error!("Encountered an error when get latest_block_number: {:?}, wait for 30 seconds.", err);
            }
        }
    }
}

async fn process_event(evm_client: &EvmClient, event: Erc1155Event, callback: &mut dyn Erc1155EventCallback) {
    let token_uri = get_token_uri(evm_client, &event).await;

    // callback
    callback.on_erc1155_event(event, token_uri).await
}

async fn get_token_uri(
    evm_client: &EvmClient,
    event: &Erc1155Event,
) -> String {
    let token_uri = evm_client.get_erc1155_token_uri(&event.address, &event.token_id).await.unwrap_or("Unknown".to_owned());
    token_uri
}

#[cfg(test)]
mod tests {
    use super::*;
    use web3::{transports::http::Http, Web3};

    struct EthereumErc1155EventCallback {
        events: Vec<Erc1155Event>,
    }

    #[async_trait]
    impl Erc1155EventCallback for EthereumErc1155EventCallback {
        async fn on_erc1155_event(
            &mut self,
            event: Erc1155Event,
            _token_uri: String,
        ) -> Result<()> {
            self.events.push(event);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_track_erc1155_events() {
        let web3 = Web3::new(Http::new("https://main-light.eth.linkpool.io").unwrap());
        let client = EvmClient::new("Ethereum".to_owned(), web3);

        //
        let conn = Connection::open("./test6.db").unwrap();
        erc1155_db::create_tables_if_not_exist(&conn).unwrap();

        //
        let mut callback = EthereumErc1155EventCallback { events: vec![] };
        track_erc1155_events(&client, &conn, 13015344, 1, Some(13015346), &mut callback).await;
        assert_eq!(5, callback.events.len());

        std::fs::remove_file("./test6.db").unwrap();
    }
}
