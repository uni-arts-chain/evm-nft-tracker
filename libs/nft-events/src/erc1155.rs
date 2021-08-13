use crate::{
    Result, Error, EvmClient
};
use web3::types::{H256, H160, Log, U256, Bytes};
use array_bytes::hex2bytes_unchecked as bytes;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug)]
pub struct Erc1155Event {
    pub block_number: Option<u64>,
    pub address: H160,
    pub transaction_hash: Option<H256>,
    pub operator: H160,
    pub from: H160,
    pub to: H160,
    pub token_id: U256,
    pub amount: U256,
}

pub trait Erc1155EventCallback: Send {
    fn on_erc1155_event(&mut self, event: Erc1155Event);
}

pub async fn track_erc1155_events(client: &EvmClient, start_from: u64, step: u64, end_block: Option<u64>, callback: &mut dyn Erc1155EventCallback) {
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
                    debug!("Scan for {} ERC1155 events in block range of {} - {}({})", client.chain_name, from, to, to - from + 1);
                    match get_erc1155_events(&client, from, to).await {
                        Ok(events) => {
                            info!("{} {} ERC1155 events were scanned in block range of {} - {}({})", events.len(), client.chain_name, from, to, to - from + 1);
                            for event in events {
                                callback.on_erc1155_event(event);
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
                                        error!("Encountered an error when get ERC1155 events from {}: {:?}, wait for 30 seconds.", client.chain_name, e);
                                        sleep(Duration::from_secs(30)).await;
                                    }
                                },
                                _ => {
                                    error!("Encountered an error when get ERC1155 events from {}: {:?}, wait for 30 seconds.", client.chain_name, err);
                                    sleep(Duration::from_secs(30)).await;
                                }
                            }
                        },
                    }
                } else {
                    debug!("Track {} ERC1155 events too fast, wait for 30 seconds.", client.chain_name);
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

pub async fn get_erc1155_events(client: &EvmClient, from: u64, to: u64) -> Result<Vec<Erc1155Event>> {
    let transfer_single_topic = H256::from_slice(&bytes("0xc3d58168c5ae7397731d063d5bbf3d657854427343f4c083240f7aacaa2d0f62"));
    let transfer_batch_topic = H256::from_slice(&bytes("0x4a39dc06d4c0dbc64b70af90fd698a233a518aa5d07e595d983b8c0526c8f7fb"));
    let topics = vec![
        transfer_single_topic, 
        transfer_batch_topic
    ];
    let logs = client.get_logs(None, topics, from, to).await?;

    let mut events = vec![];

    for log in logs {

        if client.is_erc1155(log.address).await? {

            if log.topics[0] == transfer_single_topic {
                let token_id = U256::from_big_endian(&log.data.0[0..32]);
                let amount = U256::from_big_endian(&log.data.0[32..64]);
                events.push(build_event(&log, token_id, amount));
            } else {
                let (token_ids, amounts) = get_ids_and_amounts(&log.data);
                for i in 0..token_ids.len() {
                    let token_id = token_ids[i];
                    let amount = amounts[i];
                    events.push(build_event(&log, token_id, amount));
                }
            };

        }

    }

    Ok(events)
}

fn get_ids_and_amounts(data: &Bytes) -> (Vec<U256>, Vec<U256>) {
    let mut items = vec![];
    for i in 0..data.0.len() / 32 {
        if i >= 2 {
            let item = &data.0[32*i..32*i+32];
            items.push(item);
        }
    }

    if items.len() > 0 && items.len() % 2 == 0 {
        let ids = &items[0..items.len()/2][1..];
        let ids: Vec<U256> = ids.iter().map(|id| U256::from_big_endian(id)).collect();

        let amounts = &items[items.len()/2..items.len()][1..];
        let amounts: Vec<U256> = amounts.iter().map(|id| U256::from_big_endian(id)).collect();

        (ids, amounts)
    } else {
        (vec![], vec![])
    }
}

fn build_event(log: &Log, token_id: U256, amount: U256) -> Erc1155Event {
    let operator = H160::from(log.topics[1]);
    let from = H160::from(log.topics[2]);
    let to = H160::from(log.topics[3]);
    Erc1155Event {
        block_number: log.block_number.map(|b| b.as_u64()),
        address: log.address, 
        transaction_hash: log.transaction_hash,
        operator,
        from,
        to,
        token_id,
        amount,
    }
}
