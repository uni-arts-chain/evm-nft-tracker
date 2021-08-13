use crate::{
    Result, Error, EvmClient
};
use web3::types::{H256, H160, Log, U256};
use array_bytes::hex2bytes_unchecked as bytes;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug)]
pub struct Erc721Event {
    pub block_number: Option<u64>,
    pub address: H160,
    pub transaction_hash: Option<H256>,
    pub from: H160,
    pub to: H160,
    pub token_id: U256,
}

pub trait Erc721EventCallback: Send {
    fn on_erc721_event(&mut self, event: Erc721Event);
}

pub async fn track_erc721_events(client: &EvmClient, start_from: u64, step: u64, end_block: Option<u64>, callback: &mut dyn Erc721EventCallback) {
    let mut step = step;
    let mut from = start_from;
    loop {
        match client.get_latest_block_number().await {
            Ok(latest_block_number) => {

                let to = std::cmp::min(from + step - 1, latest_block_number - 6);
                if let Some(end_block) = end_block {
                    if to > end_block {
                        break;
                    }
                }

                if to >= from {
                    debug!("Scan for {} ERC721 events in block range of {} - {}({})", client.chain_name, from, to, to - from + 1);
                    let start = Instant::now();
                    match get_erc721_events(&client, from, to).await {
                        Ok(events) => {
                            info!("{} {} ERC721 events were scanned in block range of {} - {}({})", events.len(), client.chain_name, from, to, to - from + 1);
                            for event in events {
                                callback.on_erc721_event(event);
                            }

                            from = to + 1;

                            let duration = start.elapsed();
                            debug!("Time elapsed is: {:?}", duration);

                            sleep(Duration::from_secs(5)).await;
                        },
                        Err(err) => {
                            match err {
                                Error::Web3Error(web3::Error::Rpc(e)) => {
                                    if e.message.contains("more than") {
                                        error!("{}", e.message);
                                        step = std::cmp::max(step / 2, 1);
                                    } else {
                                        error!("Encountered an error when get ERC721 events from {}: {:?}, wait for 30 seconds.", client.chain_name, e);
                                        sleep(Duration::from_secs(30)).await;
                                    }
                                },
                                _ => {
                                    error!("Encountered an error when get ERC721 events from {}: {:?}, wait for 30 seconds.", client.chain_name, err);
                                    sleep(Duration::from_secs(30)).await;
                                }
                            }
                        },
                    }
                } else {
                    debug!("Track {} ERC721 events too fast, wait for 30 seconds.", client.chain_name);
                    sleep(Duration::from_secs(30)).await;
                }

            },
            Err(err) => {
                error!("Encountered an error when get latest_block_number from {}: {:?}, wait for 30 seconds.", client.chain_name, err);
                sleep(Duration::from_secs(30)).await;
            }
        }
    }
}

/// Get all erc721 events between `from` and `to`
/// the `from` and `to` blocks are included.
pub async fn get_erc721_events(client: &EvmClient, from: u64, to: u64) -> Result<Vec<Erc721Event>> {
    let transfer_topic = H256::from_slice(&bytes("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"));
    let logs = client.get_logs(None, vec![transfer_topic], from, to).await?;
    let mut events = vec![];
    for log in logs {
        if log.topics.len() == 4 && client.is_erc721(log.address).await? {
            events.push(build_event(&log));
        }
    }

    Ok(events)
}

fn build_event(log: &Log) -> Erc721Event {
    let from = H160::from(log.topics[1]);
    let to = H160::from(log.topics[2]);
    let token_id = U256::from(log.topics[3].0);
    Erc721Event {
        block_number: log.block_number.map(|b| b.as_u64()),
        address: log.address,
        transaction_hash: log.transaction_hash,
        from,
        to,
        token_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use web3::{
        transports::http::Http,
        types::{BlockNumber, FilterBuilder, Log, SyncState, H160, H256, U64, U256},
        Web3,
        contract::{Contract, Options},
    };

    #[tokio::test]
    async fn test_get_erc721_events() {
        let web3 = Web3::new(
            Http::new("https://main-light.eth.linkpool.io").unwrap(),
        );
        let client = EvmClient::new("Ethereum", web3);

        let events = get_erc721_events(&client, 13015344, 13015344).await.unwrap();
        assert_eq!(10, events.len());
    }


    struct EthereumErc721EventCallback {
        events: Vec<Erc721Event>,
    }

    impl Erc721EventCallback for EthereumErc721EventCallback {
        fn on_erc721_event(&mut self, event: Erc721Event) {
            self.events.push(event);
        }
    }

    #[tokio::test]
    async fn test_track_erc721_events() {
        let web3 = Web3::new(
            Http::new("https://main-light.eth.linkpool.io").unwrap(),
        );
        let client = EvmClient::new("Ethereum", web3);

        let mut callback = EthereumErc721EventCallback {
            events: vec![],
        };

        track_erc721_events(&client, 13015344, 1, Some(13015346), &mut callback).await;

        assert_eq!(15, callback.events.len());

    }

}

