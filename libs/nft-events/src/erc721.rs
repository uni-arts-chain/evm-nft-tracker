//! This module is the entry point for tracking ERC721.
use crate::{erc721_evm, erc721_evm::Erc721Event, EvmClient};
use std::time::Duration;
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

/// Entry function for tracking ERC721.
/// If you only need to track ERC721, you can use this function directly.
pub async fn track_erc721_events(
    evm_client: &EvmClient,
    start_from: u64,
    step: u64,
    end_block: Option<u64>,
    callback: &mut dyn Erc721EventCallback,
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
                    debug!("Scan for ERC721 events in {} - {}({})", from, to, to - from + 1);
                    match erc721_evm::get_erc721_events(&evm_client, from, to).await {
                        Ok(events) => {

                            info!("{} ERC721 events were scanned in {} - {}({})", events.len(), from, to, to - from + 1);

                            for event in events {
                                process_event(evm_client, event.clone(), callback).await;
                            }

                            from = to + 1;

                        }
                        Err(err) => {
                            error!("Encountered an error when get ERC721 events: {:?}, wait for 30 seconds.", err);
                            sleep(Duration::from_secs(30)).await;
                        },
                    }
                } else {
                    debug!("Track ERC721 events too fast, wait for 30 seconds.");
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

async fn process_event(evm_client: &EvmClient, event: Erc721Event, callback: &mut dyn Erc721EventCallback) {
    let (name, symbol, token_uri) = get_metadata(evm_client, &event).await;

    // callback
    callback.on_erc721_event(
        event,
        name,
        symbol,
        token_uri,
    )
    .await;
}

async fn get_metadata(
    evm_client: &EvmClient,
    event: &Erc721Event,
) -> (String, String, String) {
    let name = evm_client.get_erc721_name(&event.address).await.unwrap_or("Unknown".to_owned());
    let symbol = evm_client.get_erc721_symbol(&event.address).await.unwrap_or("Unknown".to_owned());
    let token_uri = evm_client.get_erc721_token_uri(&event.address, &event.token_id).await.unwrap_or("Unknown".to_owned());
    (name, symbol, token_uri)
}


#[cfg(test)]
mod tests {
    use super::*;
    use web3::{transports::http::Http, Web3};

    struct EthereumErc721EventCallback {
        events: Vec<Erc721Event>,
    }

    #[async_trait]
    impl Erc721EventCallback for EthereumErc721EventCallback {
        async fn on_erc721_event(
            &mut self,
            event: Erc721Event,
            _name: String,
            _symbol: String,
            _total_supply: Option<u128>,
            _token_uri: String,
        ) -> Result<()> {
            self.events.push(event);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_track_erc721_events() {
        //
        let web3 = Web3::new(Http::new("https://main-light.eth.linkpool.io").unwrap());
        let client = EvmClient::new("Ethereum".to_owned(), web3);

        //
        let conn = Connection::open("./test7.db").unwrap();
        erc721_db::create_tables_if_not_exist(&conn).unwrap();

        //
        let mut callback = EthereumErc721EventCallback { events: vec![] };
        track_erc721_events(&client, &conn, 13015344, 1, Some(13015346), &mut callback).await;
        assert_eq!(15, callback.events.len());

        std::fs::remove_file("./test7.db").unwrap();
    }
}
