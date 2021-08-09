use crate::{
    Result, EvmClient
};
use web3::types::{H256, H160, Log, U256, Bytes};
use array_bytes::hex2bytes_unchecked as bytes;

#[derive(Debug)]
pub struct Erc1155Event {
    pub address: H160,
    pub transaction_hash: Option<H256>,
    pub operator: H160,
    pub from: H160,
    pub to: H160,
    pub token_id: U256,
    pub amount: U256,
}

pub async fn get_events(client: &EvmClient, from: u64, to: u64) -> Result<Vec<Erc1155Event>> {
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
        address: log.address, 
        transaction_hash: log.transaction_hash,
        operator,
        from,
        to,
        token_id,
        amount,
    }
}
