use crate::{
    Result, Error, EvmClient, erc1155_db, erc1155_evm, erc1155_evm::Erc1155Event
};
use web3::types::{H160, U256};
use std::time::Duration;
use tokio::time::sleep;

use rusqlite::Connection;

#[async_trait]
pub trait Erc1155EventCallback: Send {
    async fn on_erc1155_event(&mut self, event: Erc1155Event, token_uri: String) -> Result<()>;
}

pub async fn track_erc1155_events(evm_client: &EvmClient, db_conn: &Connection, start_from: u64, step: u64, end_block: Option<u64>, callback: &mut dyn Erc1155EventCallback) {
    let mut step = step;
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
                    debug!("Scan for {} ERC1155 events in block range of {} - {}({})", evm_client.chain_name, from, to, to - from + 1);
                    match erc1155_evm::get_erc1155_events(&evm_client, from, to).await {
                        Ok(events) => {
                            info!("{} {} ERC1155 events were scanned in block range of {} - {}({})", events.len(), evm_client.chain_name, from, to, to - from + 1);
                            for event in events {

                                // PROCESS AN EVENT
                                // ******************************************************
                                match get_token_uri(evm_client, db_conn, &event).await {
                                    Ok(token_uri) => {
                                        if let Err(err) = callback.on_erc1155_event(event.clone(), token_uri).await {
                                            error!("Encountered an error when process ERC1155 event {:?} from {}: {:?}.", event, evm_client.chain_name, err);
                                        }
                                    },
                                    Err(err) => {
                                        error!("Encountered an error when get metadata for ERC1155 event {:?} from {}: {:?}.", event, evm_client.chain_name, err);
                                    },
                                }
                                // ******************************************************

                            }

                            from = to + 1;

                            sleep(Duration::from_secs(5)).await;
                        },
                        Err(err) => {
                            match err {
                                Error::Web3Error(web3::Error::Rpc(e)) => {
                                    if e.message == "query returned more than 10000 results" {
                                        error!("{}", e.message);
                                        step = std::cmp::max(step / 2, 1);
                                    } else {
                                        error!("Encountered an error when get ERC1155 events from {}: {:?}, wait for 30 seconds.", evm_client.chain_name, e);
                                        sleep(Duration::from_secs(30)).await;
                                    }
                                },
                                _ => {
                                    error!("Encountered an error when get ERC1155 events from {}: {:?}, wait for 30 seconds.", evm_client.chain_name, err);
                                    sleep(Duration::from_secs(30)).await;
                                }
                            }
                        },
                    }
                } else {
                    debug!("Track {} ERC1155 events too fast, wait for 30 seconds.", evm_client.chain_name);
                    sleep(Duration::from_secs(30)).await;
                }

            },
            Err(err) => {
                error!("Encountered an error when get latest_block_number from {}: {:?}, wait for 30 seconds.", evm_client.chain_name, err);
                sleep(Duration::from_secs(30)).await;
            }
        }
    }
}

async fn get_token_uri(evm_client: &EvmClient, db_conn: &Connection, event: &Erc1155Event) -> Result<String> {
    save_metadata_to_db_if_not_exists(evm_client, db_conn, &event.address, &event.token_id).await?;
    let collection = erc1155_db::get_collection_from_db(db_conn, &format!("{:?}", event.address))?.unwrap();
    let token = erc1155_db::get_token_from_db(db_conn, collection.0, &event.token_id.to_string())?.unwrap();
    Ok(token.3.unwrap())
}

async fn save_metadata_to_db_if_not_exists(evm_client: &EvmClient, db_conn: &Connection, address: &H160, token_id: &U256) -> Result<()> {
    let address_string = format!("{:?}", address);
    let collection_id = if let Some(collection) = erc1155_db::get_collection_from_db(db_conn, &address_string)? {
        collection.0
    } else {
        erc1155_db::add_collection_to_db(db_conn, address_string.clone())?
    };

    let token = erc1155_db::get_token_from_db(db_conn, collection_id, &token_id.to_string())?;
    if token.is_none() {
        let token_uri = evm_client.get_erc1155_token_uri(address, token_id).await?;
        erc1155_db::add_token_to_db(db_conn, token_id.to_string(), collection_id, Some(token_uri))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use web3::{
        transports::http::Http,
        Web3,
    };

    struct EthereumErc1155EventCallback {
        events: Vec<Erc1155Event>,
    }

    #[async_trait]
    impl Erc1155EventCallback for EthereumErc1155EventCallback {
        async fn on_erc1155_event(&mut self, event: Erc1155Event, token_uri: String) -> Result<()> {
            self.events.push(event);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_track_erc1155_events() {
        std::env::set_var(
            "RUST_LOG",
            r#"
            nft_events=debug,
            "#,
        );
        env_logger::init();

        let web3 = Web3::new(
            Http::new("https://main-light.eth.linkpool.io").unwrap(),
        );
        let client = EvmClient::new("Ethereum", web3);

        let mut callback = EthereumErc1155EventCallback {
            events: vec![],
        };

        track_erc1155_events(&client, 13015344, 1, Some(13015346), &mut callback).await;

        assert_eq!(14, callback.events.len());

    }

}

