use crate::Result;
use web3::{
    transports::http::Http,
    types::{BlockNumber, FilterBuilder, Log, SyncState, H160, H256, U64, U256},
    Web3,
    contract::{Contract, Options},
};
use array_bytes::hex2array;

#[derive(Clone)]
pub struct EvmClient {
    pub chain_name: &'static str,
    web3: Web3<Http>,
}

impl EvmClient {
    pub fn new(chain_name: &'static str, web3: Web3<Http>) -> EvmClient {
        EvmClient { 
            chain_name,
            web3
        }
    }
}

impl EvmClient {
    pub async fn get_logs(
        &self,
        contract_address: Option<H160>,
        topics: Vec<H256>,
        from: u64,
        to: u64,
    ) -> Result<Vec<Log>> {
        // build filter
        let filter_builder = if let Some(contract) = contract_address {
            FilterBuilder::default()
                .address(vec![contract])
                .topics(Some(topics.clone()), None, None, None)
        } else {
            FilterBuilder::default()
                .topics(Some(topics.clone()), None, None, None)
        };

        let filter = filter_builder
            .clone()
            .from_block(BlockNumber::Number(U64::from(from)))
            .to_block(BlockNumber::Number(U64::from(to)))
            .build();

        Ok(self.web3.eth().logs(filter).await?)
    }

    pub async fn get_latest_block_number(&self) -> Result<u64> {
        let eth = self.web3.eth();
        let sync_state = eth.syncing().await?;

        let latest_block_number = match sync_state {
            // TOOD: what the difference between eth_blockNumber and eth_getBlockByNumber("latest", false)
            SyncState::NotSyncing => eth.block_number().await?.as_u64(),
            SyncState::Syncing(info) => info.current_block.as_u64(),
        };
        Ok(latest_block_number)
    }

    pub async fn is_erc721(&self, contract_address: H160) -> Result<bool> {
        let contract = Contract::from_json(
            self.web3.eth(),
            contract_address,
            include_bytes!("./contracts/erc721.json"),
        )?;
        let interface_id: [u8; 4] = hex2array::<_, 4>("0x80ac58cd").unwrap();
        Ok(
            contract.query("supportsInterface", (interface_id,), None, Options::default(), None).await?
        )
    }

    /// (name, symbol, token_uri)
    pub async fn get_erc721_metadata(&self, contract_address: H160, token_id: U256) -> Result<Option<(String, String, String)>> {
        let contract = Contract::from_json(
            self.web3.eth(),
            contract_address,
            include_bytes!("./contracts/erc721.json"),
        )?;
        let interface_id: [u8; 4] = hex2array::<_, 4>("0x5b5e139f").unwrap();
        let supports_metadata: bool = contract.query("supportsInterface", (interface_id,), None, Options::default(), None).await?;
        if supports_metadata {
            let name: String = contract.query("name", (), None, Options::default(), None).await?;
            let symbol: String = contract.query("symbol", (), None, Options::default(), None).await?;
            let token_uri: String = contract.query("tokenURI", (token_id,), None, Options::default(), None).await?;
            Ok(Some((name, symbol, token_uri)))
        } else {
            Ok(None)
        }
    }

    pub async fn is_erc1155(&self, contract_address: H160) -> Result<bool> {
        let contract = Contract::from_json(
            self.web3.eth(),
            contract_address,
            include_bytes!("./contracts/erc1155.json"),
        )?;
        let interface_id: [u8; 4] = hex2array::<_, 4>("0xd9b67a26").unwrap();
        Ok(
            contract.query("supportsInterface", (interface_id,), None, Options::default(), None).await?
        )
    }
}
