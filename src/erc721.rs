use crate::{
    Result, Error, EvmClient
};
use web3::types::{H256, H160, Log, U256};
use array_bytes::hex2bytes_unchecked as bytes;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug)]
pub struct Erc721Event {
    pub address: H160,
    pub transaction_hash: Option<H256>,
    pub from: H160,
    pub to: H160,
    pub token_id: U256,
}

pub trait Erc721EventCallback {
    fn on_erc721_event(&self, event: Erc721Event);
}

pub async fn track_erc721_events(client: &EvmClient, start_from: u64, step: u64, callback: Box<dyn Erc721EventCallback>) {
    let mut step = step;
    let mut from = start_from;
    loop {
        match client.get_latest_block_number().await {
            Ok(latest_block_number) => {

                let to = std::cmp::min(from + step - 1, latest_block_number - 6);

                if to >= from {
                    debug!("Scan for {} ERC721 events in block range of {} - {}({})", client.chain_name, from, to, to - from + 1);
                    let start = Instant::now();
                    match get_events(&client, from, to).await {
                        Ok(events) => {
                            debug!("{} {} ERC721 events were scanned", client.chain_name, events.len());
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
                                    if e.message == "query returned more than 10000 results" {
                                        step = std::cmp::max(step / 2, 1);
                                    }
                                },
                                _ => {
                                    debug!("Encountered an error when get ERC721 events from {}: {:?}, wait for 30 seconds.", client.chain_name, err);
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
                println!("Encountered an error when get latest_block_number from {}: {:?}, wait for 30 seconds.", client.chain_name, err);
                sleep(Duration::from_secs(30)).await;
            }
        }
    }
}

pub async fn get_events(client: &EvmClient, from: u64, to: u64) -> Result<Vec<Erc721Event>> {
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
        address: log.address,
        transaction_hash: log.transaction_hash,
        from,
        to,
        token_id
    }
}
