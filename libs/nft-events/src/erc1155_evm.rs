//! This module is a library to get ERC1155 transfer events.
use crate::{EvmClient, Result};
use array_bytes::hex2bytes_unchecked as bytes;
use web3::types::{Bytes, Log, H160, H256, U256};

/// The Erc721 Transfer Event Wrapper
#[derive(Debug, Clone)]
pub struct Erc1155Event {
    /// The block to which this event belongs
    pub block_number: Option<u64>,
    /// The ERC721 contract address
    pub address: H160,
    /// The transaction that issued this event
    pub transaction_hash: Option<H256>,
    /// The address of an account/contract that is approved to make the transfer
    pub operator: H160,
    /// Transfer from
    pub from: H160,
    /// Balance of from
    pub balance_of_from: U256,
    /// Transfer to
    pub to: H160,
    /// Balance of to
    pub balance_of_to: U256,
    /// The token type being transferred
    pub token_id: U256,
    /// Number of the token transferred
    pub amount: U256,
}

/// Get all erc1155 events between `from` and `to`.
/// the `from` and `to` blocks are included.
pub async fn get_erc1155_events(
    client: &EvmClient,
    from: u64,
    to: u64,
) -> Result<Vec<Erc1155Event>> {
    let transfer_single_topic = H256::from_slice(&bytes(
        "0xc3d58168c5ae7397731d063d5bbf3d657854427343f4c083240f7aacaa2d0f62",
    ));
    let transfer_batch_topic = H256::from_slice(&bytes(
        "0x4a39dc06d4c0dbc64b70af90fd698a233a518aa5d07e595d983b8c0526c8f7fb",
    ));
    let topics = vec![transfer_single_topic, transfer_batch_topic];
    let logs = client.get_logs(None, topics, from, to).await?;

    let mut result = vec![];

    for log in logs {
        if client.is_visual_erc1155(log.address).await? {
            if log.topics[0] == transfer_single_topic {
                let event = build_event(client, &log).await?;
                result.push(event);
            } else {
                let mut events = build_events(client, &log).await?;
                result.append(&mut events);
            };
        }
    }

    Ok(result)
}

async fn build_event(client: &EvmClient, log: &Log) -> Result<Erc1155Event> {
    let token_id = U256::from_big_endian(&log.data.0[0..32]);
    let amount = U256::from_big_endian(&log.data.0[32..64]);
    let block_number = log.block_number.map(|b| b.as_u64());
    let address = log.address;
    let transaction_hash = log.transaction_hash;
    let operator = H160::from(log.topics[1]);
    let from = H160::from(log.topics[2]);
    let to = H160::from(log.topics[3]);
    let balances = client.get_erc1155_balances(&address, &vec![from, to], &vec![token_id, token_id], block_number).await?;
    // let balance_of_from = U256::zero();// balances[0];
    // let balance_of_to = U256::zero();// balances[1];
    let balance_of_from = balances[0];
    let balance_of_to = balances[1];

    Ok(
        Erc1155Event {
            block_number,
            address,
            transaction_hash,
            operator,
            from,
            balance_of_from,
            to,
            balance_of_to,
            token_id, 
            amount,
        }
    )
}

async fn build_events(client: &EvmClient, log: &Log) -> Result<Vec<Erc1155Event>> {
    let block_number = log.block_number.map(|b| b.as_u64());
    let address = log.address;
    let transaction_hash = log.transaction_hash;
    let operator = H160::from(log.topics[1]);
    let from = H160::from(log.topics[2]);
    let to = H160::from(log.topics[3]);

    // owner_list: token_id_list
    // from: 1
    // from: 2
    // from: 3
    // to: 1
    // to: 2
    // to: 3
    let (token_ids, amounts) = get_ids_and_amounts(&log.data);
    let mut owner_list = vec![];
    let mut token_id_list = vec![];
    for i in 0..token_ids.len() {
        let token_id = token_ids[i];
        owner_list.push(from);
        token_id_list.push(token_id);
    }
    for i in 0..token_ids.len() {
        let token_id = token_ids[i];
        owner_list.push(to);
        token_id_list.push(token_id);
    }
    let balances = client.get_erc1155_balances(&address, &owner_list, &token_id_list, block_number).await?;

    //
    let mut events = vec![];
    for i in 0..token_ids.len() {
        let token_id = token_ids[i];
        let amount = amounts[i];
        // let balance_of_from = U256::zero();// balances[i];
        // let balance_of_to = U256::zero();// balances[i+3];
        let balance_of_from = balances[i];
        let balance_of_to = balances[i+3];
        events.push(Erc1155Event {
            block_number,
            address,
            transaction_hash,
            operator,
            from,
            balance_of_from,
            to,
            balance_of_to,
            token_id, 
            amount,
        });
    }

    Ok(events)
}

fn get_ids_and_amounts(data: &Bytes) -> (Vec<U256>, Vec<U256>) {
    let mut items = vec![];
    for i in 0..data.0.len() / 32 {
        if i >= 2 {
            let item = &data.0[32 * i..32 * i + 32];
            items.push(item);
        }
    }

    if items.len() > 0 && items.len() % 2 == 0 {
        let ids = &items[0..items.len() / 2][1..];
        let ids: Vec<U256> = ids.iter().map(|id| U256::from_big_endian(id)).collect();

        let amounts = &items[items.len() / 2..items.len()][1..];
        let amounts: Vec<U256> = amounts.iter().map(|id| U256::from_big_endian(id)).collect();

        (ids, amounts)
    } else {
        (vec![], vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use web3::{transports::http::Http, Web3};

    #[tokio::test]
    async fn test_get_erc1155_events() {
        let web3 = Web3::new(Http::new("https://main-light.eth.linkpool.io").unwrap());
        let client = EvmClient::new("Ethereum".to_owned(), web3);

        let events = get_erc1155_events(&client, 13015344, 13015344)
            .await
            .unwrap();
        assert_eq!(4, events.len());
    }
}
