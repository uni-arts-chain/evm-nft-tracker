use crate::{EvmClient, Result, Error};
use array_bytes::hex2bytes_unchecked as bytes;
use jsonrpc_core::error;
use web3::types::{Bytes, Log, H160, H256, U256};

/// The Erc721 Transfer Event Wrapper
#[derive(Debug, Clone)]
pub struct Erc721Event {
    /// The block to which this event belongs
    pub block_number: Option<u64>,
    /// The ERC721 contract address
    pub address: H160,
    /// The transaction that issued this event
    pub transaction_hash: Option<H256>,
    /// Transfer from
    pub from: H160,
    /// Transfer to
    pub to: H160,
    /// Transferred ERC721 token
    pub token_id: U256,
}

/// The Erc1155 Transfer Event Wrapper
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
    /// Transfer to
    pub to: H160,
    /// The token type being transferred
    pub token_id: U256,
    /// Number of the token transferred
    pub amount: U256,
}

/// Event
#[derive(Debug, Clone)]
pub enum Event {
    /// Erc721Event
    Erc721(Erc721Event),
    /// Erc1155Event
    Erc1155(Erc1155Event)
}

/// Get all events between `from` and `to`.
/// the `from` and `to` blocks are included.
pub async fn get_events(client: &EvmClient, from: u64, to: u64) -> Result<Vec<Event>> {
    let erc721_transfer_topic = H256::from_slice(&bytes(
        "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
    ));
    let erc1155_transfer_single_topic = H256::from_slice(&bytes(
        "0xc3d58168c5ae7397731d063d5bbf3d657854427343f4c083240f7aacaa2d0f62",
    ));
    let erc1155_transfer_batch_topic = H256::from_slice(&bytes(
        "0x4a39dc06d4c0dbc64b70af90fd698a233a518aa5d07e595d983b8c0526c8f7fb",
    ));
    let topics = vec![
        erc721_transfer_topic, 
        erc1155_transfer_single_topic, 
        erc1155_transfer_batch_topic
    ];
    let logs = client.get_logs(None, topics, from, to).await?;

    let mut result = vec![];
    for log in logs {

        if let Err(err) = process_log(client, &log, erc721_transfer_topic, erc1155_transfer_single_topic, &mut result).await {

            if let Some(e) = process_err(&log, err) {
                return Err(e);
            }

        }

    }

    Ok(result)
}

fn process_err(log: &Log, err: Error) -> Option<Error> {
    match err {
        Error::Web3ContractError(ref e1) => {
            match e1 {

                web3::contract::Error::Abi(web3::ethabi::Error::InvalidName(msg)) => {
                    error!("{:?} >>> {}", log, msg);
                    None
                },

                web3::contract::Error::Api(web3::Error::Rpc(jsonrpc_core::types::Error { code, message, data } )) => {
                    if message == "execution reverted" {
                        error!("{:?} >>> {:?}", log, err);
                        None
                    } else {
                        Some(err)
                    }
                },

                _ => Some(err),
            }
        },
        _ => Some(err),
    }
}

async fn process_log(client: &EvmClient, log: &Log, erc721_transfer_topic: H256, erc1155_transfer_single_topic: H256, result: &mut Vec<Event>) -> Result<()> {
    Ok(if log.topics[0] == erc721_transfer_topic {

        // ERC721
        if log.topics.len() == 4 &&
            client.is_erc721(log.address).await? &&
                client.supports_erc721_metadata(log.address).await? {
            result.push(build_erc721_event(&log));
        }

    } else {

        // ERC1155
        if client.is_erc1155(log.address).await? && 
            client.supports_erc1155_metadata(log.address).await? {
            if log.topics[0] == erc1155_transfer_single_topic {
                let event = build_erc1155_event(&log);
                result.push(event);
            } else {
                let mut events = build_erc1155_events(&log);
                result.append(&mut events);
            };
        }

    })
}

fn build_erc721_event(log: &Log) -> Event {
    let from = H160::from(log.topics[1]);
    let to = H160::from(log.topics[2]);
    let token_id = U256::from(log.topics[3].0);
    Event::Erc721(
        Erc721Event {
            block_number: log.block_number.map(|b| b.as_u64()),
            address: log.address,
            transaction_hash: log.transaction_hash,
            from,
            to,
            token_id,
        }
    )
}

fn build_erc1155_event(log: &Log) -> Event {
    let token_id = U256::from_big_endian(&log.data.0[0..32]);
    let amount = U256::from_big_endian(&log.data.0[32..64]);
    let block_number = log.block_number.map(|b| b.as_u64());
    let address = log.address;
    let transaction_hash = log.transaction_hash;
    let operator = H160::from(log.topics[1]);
    let from = H160::from(log.topics[2]);
    let to = H160::from(log.topics[3]);

    Event::Erc1155(
        Erc1155Event {
            block_number,
            address,
            transaction_hash,
            operator,
            from,
            to,
            token_id, 
            amount,
        }
    )
}

fn build_erc1155_events(log: &Log) -> Vec<Event> {
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

    //
    let mut events = vec![];
    for i in 0..token_ids.len() {
        let token_id = token_ids[i];
        let amount = amounts[i];
        let event = Event::Erc1155(
            Erc1155Event {
                block_number,
                address,
                transaction_hash,
                operator,
                from,
                to,
                token_id, 
                amount,
            }
        );
        events.push(event);
    }

    events
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
