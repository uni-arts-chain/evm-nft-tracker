use crate::{
    Result, EvmClient
};
use web3::types::{H256, H160, Log, U256};
use array_bytes::hex2bytes_unchecked as bytes;

#[derive(Debug, Clone)]
pub struct Erc721Event {
    pub block_number: Option<u64>,
    pub address: H160,
    pub transaction_hash: Option<H256>,
    pub from: H160,
    pub to: H160,
    pub token_id: U256,
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
        Web3,
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
}
