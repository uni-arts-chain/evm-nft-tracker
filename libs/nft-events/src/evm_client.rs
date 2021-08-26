//! This module contains an EVM client.
//! This EVM client provides several methods for accessing the EVM of the host blockchain.
use crate::Result;
use array_bytes::hex2array;
use web3::{
    contract::{Contract, Options},
    transports::http::Http,
    types::{BlockNumber, FilterBuilder, Log, SyncState, H160, H256, U256, U64},
    Web3,
};

/// The EVM client struct
#[derive(Clone)]
pub struct EvmClient {
    /// The blockchain name used for display
    pub chain_name: String,
    web3: Web3<Http>,
}

impl EvmClient {
    /// Initialize a new EvmClient instance
    pub fn new(chain_name: String, web3: Web3<Http>) -> EvmClient {
        EvmClient { chain_name, web3 }
    }
}

impl EvmClient {
    /// Get EVM `Log` from the blockchain according to the conditions
    /// If the distance between `from` and `to` is large, it may take a
    /// long time to return. In some cases it may end up in error.
    pub async fn get_logs(
        &self,
        contract_address: Option<H160>,
        topics: Vec<H256>,
        from: u64,
        to: u64,
    ) -> Result<Vec<Log>> {
        // build filter
        let filter_builder = if let Some(contract) = contract_address {
            FilterBuilder::default().address(vec![contract]).topics(
                Some(topics.clone()),
                None,
                None,
                None,
            )
        } else {
            FilterBuilder::default().topics(Some(topics.clone()), None, None, None)
        };

        let filter = filter_builder
            .clone()
            .from_block(BlockNumber::Number(U64::from(from)))
            .to_block(BlockNumber::Number(U64::from(to)))
            .build();

        Ok(self.web3.eth().logs(filter).await?)
    }

    /// Get the latest block number
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

    /// Check if a contract address is a visual ERC721 contract
    pub async fn is_visual_erc721(&self, contract_address: H160) -> Result<bool> {

        let contract = Contract::from_json(
            self.web3.eth(),
            contract_address,
            include_bytes!("./contracts/erc721.json"),
        )?;

        let interface_id: [u8; 4] = hex2array::<_, 4>("0x80ac58cd").unwrap();
        let is_erc721: web3::contract::Result<bool> = contract
            .query(
                "supportsInterface",
                (interface_id,),
                None,
                Options::default(),
                None,
            )
            .await;

        match is_erc721 {
            Ok(erc721) => {
                if erc721 {
                    let interface_id: [u8; 4] = hex2array::<_, 4>("0x5b5e139f").unwrap();
                    let supports_metadata: web3::contract::Result<bool> = contract
                        .query(
                            "supportsInterface",
                            (interface_id,),
                            None,
                            Options::default(),
                            None,
                        )
                        .await;
                    match supports_metadata {
                        Ok(supports) => Ok(supports),
                        Err(_) => Ok(false),
                    }
                } else {
                    Ok(false)
                }
            },
            Err(_) => Ok(false),
        }
    }

    /// Get the metadata of an ERC721 token
    /// If the ERC721 contract not support metadata, this function will return Ok(None).
    /// The returned tuple is (name, symbol, token_uri).
    pub async fn get_erc721_metadata(
        &self,
        contract_address: &H160,
        token_id: &U256,
    ) -> Result<Option<(String, String, String)>> {
        let contract = Contract::from_json(
            self.web3.eth(),
            contract_address.clone(),
            include_bytes!("./contracts/erc721.json"),
        )?;
        let interface_id: [u8; 4] = hex2array::<_, 4>("0x5b5e139f").unwrap();
        let supports_metadata: bool = contract
            .query(
                "supportsInterface",
                (interface_id,),
                None,
                Options::default(),
                None,
            )
            .await?;
        if supports_metadata {
            let name: String = contract
                .query("name", (), None, Options::default(), None)
                .await?;
            let symbol: String = contract
                .query("symbol", (), None, Options::default(), None)
                .await?;
            let token_uri: String = contract
                .query(
                    "tokenURI",
                    (token_id.clone(),),
                    None,
                    Options::default(),
                    None,
                )
                .await?;
            Ok(Some((name, symbol, token_uri)))
        } else {
            Ok(None)
        }
    }

    /// Get the name and symbol of an ERC721 contract
    pub async fn get_erc721_name_symbol(
        &self,
        contract_address: &H160,
    ) -> Result<Option<(String, String)>> {
        let contract = Contract::from_json(
            self.web3.eth(),
            contract_address.clone(),
            include_bytes!("./contracts/erc721.json"),
        )?;
        let interface_id: [u8; 4] = hex2array::<_, 4>("0x5b5e139f").unwrap();
        let supports_metadata: bool = contract
            .query(
                "supportsInterface",
                (interface_id,),
                None,
                Options::default(),
                None,
            )
            .await?;
        if supports_metadata {
            let name: String = contract
                .query("name", (), None, Options::default(), None)
                .await?;
            let symbol: String = contract
                .query("symbol", (), None, Options::default(), None)
                .await?;
            Ok(Some((name, symbol)))
        } else {
            Ok(None)
        }
    }

    /// Get the token_uri of an ERC721 token
    pub async fn get_erc721_token_uri(
        &self,
        contract_address: &H160,
        token_id: &U256,
    ) -> Result<Option<String>> {
        let contract = Contract::from_json(
            self.web3.eth(),
            contract_address.clone(),
            include_bytes!("./contracts/erc721.json"),
        )?;

        let interface_id: [u8; 4] = hex2array::<_, 4>("0x5b5e139f").unwrap();
        let supports_metadata: bool = contract
            .query(
                "supportsInterface",
                (interface_id,),
                None,
                Options::default(),
                None,
            )
            .await?;
        if supports_metadata {
            let token_uri: String = contract
                .query(
                    "tokenURI",
                    (token_id.clone(),),
                    None,
                    Options::default(),
                    None,
                )
                .await?;
            Ok(Some(token_uri))
        } else {
            Ok(None)
        }
    }

    /// Check if a contract address is a visual ERC1155 contract
    pub async fn is_visual_erc1155(&self, contract_address: H160) -> Result<bool> {
        let contract = Contract::from_json(
            self.web3.eth(),
            contract_address,
            include_bytes!("./contracts/erc1155.json"),
        )?;

        let interface_id: [u8; 4] = hex2array::<_, 4>("0xd9b67a26").unwrap();
        let is_erc1155: web3::contract::Result<bool> = contract
            .query(
                "supportsInterface",
                (interface_id,),
                None,
                Options::default(),
                None,
            )
            .await;

        match is_erc1155 {
            Ok(erc1155) => {
                if erc1155 {
                    let interface_id: [u8; 4] = hex2array::<_, 4>("0x0e89341c").unwrap();
                    let supports_metadata: web3::contract::Result<bool> = contract
                        .query(
                            "supportsInterface",
                            (interface_id,),
                            None,
                            Options::default(),
                            None,
                        )
                        .await;
                    match supports_metadata {
                        Ok(supports) => Ok(supports),
                        Err(_) => Ok(false),
                    }
                } else {
                    Ok(false)
                }
            },
            Err(_) => Ok(false),
        }
    }

    /// Get the uri of an ERC1155 token
    /// If the ERC1155 contract not support metadata, this function will return an Err
    pub async fn get_erc1155_token_uri(
        &self,
        contract_address: &H160,
        token_id: &U256,
    ) -> Result<String> {
        let contract = Contract::from_json(
            self.web3.eth(),
            contract_address.clone(),
            include_bytes!("./contracts/erc1155.json"),
        )?;

        let token_uri: String = contract
            .query("uri", (token_id.clone(),), None, Options::default(), None)
            .await?;
        Ok(token_uri)

        // match contract.query("uri", (token_id.clone(),), None, Options::default(), None).await {
        //     Ok(token_uri@String) => Ok(Some(token_uri)),
        //     Err(err) => {

        //     }
        // }

        // let interface_id: [u8; 4] = hex2array::<_, 4>("0x0e89341c").unwrap();
        // let supports_metadata: bool = contract.query("supportsInterface", (interface_id,), None, Options::default(), None).await?;
        // if supports_metadata {
        //     let token_uri: String = contract.query("uri", (token_id.clone(),), None, Options::default(), None).await?;
        //     Ok(Some(token_uri))
        // } else {
        //     Ok(None)
        // }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    // use std::io::{stdin,stdout,Write};

    use super::*;

    #[tokio::test]
    async fn test_is_visual_erc721() {
        let web3 = Web3::new(Http::new("https://main-light.eth.linkpool.io").unwrap());
        let client = EvmClient::new("Ethereum".to_owned(), web3);

        // A visual ERC721
        let address = H160::from_str("0xa56a4f2b9807311ac401c6afba695d3b0c31079d").unwrap();
        assert_eq!(true, client.is_visual_erc721(address).await.unwrap());

        // Not ERC721
        let address = H160::from_str("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap();
        assert_eq!(false, client.is_visual_erc721(address).await.unwrap());

        // Not contract address
        let address = H160::from_str("0x0000000000000000000000000000000000000000").unwrap();
        assert_eq!(false, client.is_visual_erc721(address).await.unwrap());
    }

    #[tokio::test]
    async fn test_non_visual_erc721() {
        let web3 = Web3::new(Http::new("https://pangolin-rpc.darwinia.network").unwrap());
        let client = EvmClient::new("Pangolin".to_owned(), web3);
        // A non-visual ERC721
        let address = H160::from_str("0x2b75d135E605D9aBABb9a6F7bFad31F7d003F44e").unwrap();
        assert_eq!(false, client.is_visual_erc721(address).await.unwrap());
    }

    #[tokio::test]
    async fn test_is_visual_erc1155() {
        let web3 = Web3::new(Http::new("https://main-light.eth.linkpool.io").unwrap());
        let client = EvmClient::new("Ethereum".to_owned(), web3);

        // ERC1155
        let address = H160::from_str("0x797a48c46be32aafcedcfd3d8992493d8a1f256b").unwrap();
        assert_eq!(true, client.is_visual_erc1155(address).await.unwrap());

        // Not ERC155, support ERC165
        let address = H160::from_str("0xa56a4f2b9807311ac401c6afba695d3b0c31079d").unwrap();
        assert_eq!(false, client.is_visual_erc1155(address).await.unwrap());

        // Not ERC1155, not support ERC165
        let address = H160::from_str("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap();
        assert_eq!(false, client.is_visual_erc1155(address).await.unwrap());

        // Not contract address
        let address = H160::from_str("0x0000000000000000000000000000000000000000").unwrap();
        assert_eq!(false, client.is_visual_erc1155(address).await.unwrap());
    }

    #[tokio::test]
    async fn test_non_visual_erc1155() {
        let web3 = Web3::new(Http::new("https://pangolin-rpc.darwinia.network").unwrap());
        let client = EvmClient::new("Pangolin".to_owned(), web3);
        // A non-visual ERC1155
        let address = H160::from_str("0x1Cc1D7F55D5540041f869cF94c1294A0D95992C0").unwrap();
        assert_eq!(false, client.is_visual_erc1155(address).await.unwrap());
    }

    #[tokio::test]
    async fn test_get_erc721_metadata() {
        let web3 = Web3::new(Http::new("https://main-light.eth.linkpool.io").unwrap());
        let client = EvmClient::new("Ethereum".to_owned(), web3);

        let address = H160::from_str("0xa56a4f2b9807311ac401c6afba695d3b0c31079d").unwrap();
        let token_id = U256::from_dec_str("10279").unwrap();
        let metadata = client
            .get_erc721_metadata(&address, &token_id)
            .await
            .unwrap()
            .unwrap();
        let name = metadata.0;
        let symbol = metadata.1;
        let token_uri = metadata.2;
        assert_eq!("MonsterBlocks", name);
        assert_eq!("MONSTERBLOCK", symbol);
        assert_eq!("https://api.monsterblocks.io/metadata/10279", token_uri);
    }

    #[tokio::test]
    async fn test_get_logs() {
        let transfer_topic =
            H256::from_str("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef")
                .unwrap();

        // let mut infura_project_id=String::new();
        // print!("Please enter your infura project id: ");
        // let _ = stdout().flush();
        // stdin().read_line(&mut infura_project_id).expect("Did not enter a correct infura project id?");
        // if let Some('\n') = infura_project_id.chars().next_back() {
        //     infura_project_id.pop();
        // }
        // if let Some('\r') = infura_project_id.chars().next_back() {
        //     infura_project_id.pop();
        // }

        let infura_project_id = "60703fcc6b4e48079cfc5e385ee7af80";
        let web3 = Web3::new(
            Http::new(format!("https://mainnet.infura.io/v3/{}", infura_project_id).as_str())
                .unwrap(),
        );
        let client_infura = EvmClient::new("Ethereum".to_owned(), web3);
        let logs_from_infura = client_infura
            .get_logs(None, vec![transfer_topic], 13000000, 13000010)
            .await
            .unwrap();

        let web3 = Web3::new(Http::new("https://main-light.eth.linkpool.io").unwrap());
        let client_linkpool = EvmClient::new("Ethereum".to_owned(), web3);
        let logs_from_linkpool = client_linkpool
            .get_logs(None, vec![transfer_topic], 13000000, 13000010)
            .await
            .unwrap();

        assert_eq!(logs_from_linkpool.len(), logs_from_infura.len());
    }

    #[tokio::test]
    async fn test_get_logs_fail_cased_by_too_big_range() {
        let transfer_topic =
            H256::from_str("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef")
                .unwrap();

        let infura_project_id = "60703fcc6b4e48079cfc5e385ee7af80";
        let web3 = Web3::new(
            Http::new(format!("https://mainnet.infura.io/v3/{}", infura_project_id).as_str())
                .unwrap(),
        );
        let client_infura = EvmClient::new("Ethereum".to_owned(), web3);
        let result = client_infura
            .get_logs(None, vec![transfer_topic], 13000000, 13001000)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_erc721_token_uri() {
        let web3 = Web3::new(Http::new("https://main-light.eth.linkpool.io").unwrap());
        let client = EvmClient::new("Ethereum".to_owned(), web3);

        let address = H160::from_str("0xa56a4f2b9807311ac401c6afba695d3b0c31079d").unwrap();
        let token_id = U256::from_dec_str("10279").unwrap();
        let token_uri = client
            .get_erc721_token_uri(&address, &token_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!("https://api.monsterblocks.io/metadata/10279", token_uri);
    }

    #[tokio::test]
    async fn test_get_erc721_token_uri_fail() {
        let web3 = Web3::new(Http::new("https://main-light.eth.linkpool.io").unwrap());
        let client = EvmClient::new("Ethereum".to_owned(), web3);

        let address = H160::from_str("0x57f1887a8bf19b14fc0df6fd9b2acc9af147ea85").unwrap();
        let token_id = U256::from_dec_str(
            "38845564502965131371508063114826058623537470318810020350714825917421388823764",
        )
        .unwrap();
        let token_uri = client
            .get_erc721_token_uri(&address, &token_id)
            .await
            .unwrap();
        assert_eq!(None, token_uri);
    }

    #[tokio::test]
    async fn test_get_erc1155_token_uri() {
        let web3 = Web3::new(Http::new("https://main-light.eth.linkpool.io").unwrap());
        let client = EvmClient::new("Ethereum".to_owned(), web3);

        // This erc1155 contract is not support erc1155 metadata extension, beacause supportsInterface(0x0e89341c) retruns false,
        // but it has the uri(token_id) method
        let address = H160::from_str("0x76be3b62873462d2142405439777e971754e8e77").unwrap();
        let token_id = U256::from_dec_str("10276").unwrap();
        let token_uri = client
            .get_erc1155_token_uri(&address, &token_id)
            .await
            .unwrap();
        assert_eq!(
            "ipfs://ipfs/QmXTSZ7ag9yzQFkvUrZ5qr7KuCUtW2jeDiYJrJ8s7TpSNb",
            token_uri
        );
    }
}
