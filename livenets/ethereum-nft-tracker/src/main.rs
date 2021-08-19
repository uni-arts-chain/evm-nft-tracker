use web3::{
    Web3,
    transports::Http,
};
use web3::types::{H256, H160, Log, U256};
use nft_events::{
    EvmClient,
    erc721, erc721_db, Erc721Event, Erc721EventCallback,
    erc1155, erc1155_db, Erc1155Event, Erc1155EventCallback,
};
use std::env;
use directories_next::ProjectDirs;
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use rusqlite::Connection;

#[macro_use]
extern crate log;

#[macro_use]
extern crate async_trait;

struct EthereumErc721EventCallback {
    evm_client: EvmClient,
}

impl EthereumErc721EventCallback {
    fn new(client: EvmClient) -> Self {
        Self {
            evm_client: client,
        }
    }

}

#[async_trait]
impl Erc721EventCallback for EthereumErc721EventCallback {

    async fn on_erc721_event(&mut self, event: Erc721Event, name: Option<String>, symbol: Option<String>, token_uri: Option<String>) -> nft_events::Result<()> {
        println!("------------------------------------------------------------------------------------------");
        println!("event: {:?}", event);
        println!("name: {:?}, symbol: {:?}, token_uri: {:?}", name, symbol, token_uri);

        Ok(())
    }

}

struct EthereumErc1155EventCallback {

}

#[async_trait]
impl Erc1155EventCallback for EthereumErc1155EventCallback {
    async fn on_erc1155_event(&mut self, event: Erc1155Event, token_uri: String) -> nft_events::Result<()> {
        println!("++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++");
        println!("event: {:?}", event);
        println!("token_uri: {:?}", token_uri);
        
        Ok(())
    }
}

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct EthereumNftTrackerConfig {
    rpc: String,
    step: u64,
}

impl Default for EthereumNftTrackerConfig {
    fn default() -> Self {
        EthereumNftTrackerConfig {
            rpc: "https://main-light.eth.linkpool.io".to_owned(),
            step: 6,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var(
        "RUST_LOG",
        r#"
        ethereum_nft_tracker=info,
		nft_events=debug,
        "#,
    );
    env_logger::init();

    let blockchain_name = "ethereum";

    // Data dir
    let app_name = format!("{}-nft-tracker", blockchain_name);
    let project = ProjectDirs::from("pro", "uniscan", app_name.as_str()).unwrap();
    let data_dir = project.data_dir().to_str().unwrap();
    info!("DATA DIR : {}", data_dir);

    // Read config from config file
    let config_path: PathBuf = [data_dir, "config.toml"].iter().collect();
    let cfg: EthereumNftTrackerConfig = confy::load_path(config_path)?;
    let ethereum_rpc = &cfg.rpc;
    let step = cfg.step;
    info!("  {} rpc : {}", blockchain_name, ethereum_rpc);
    info!("  scan step : {} blocks", step);
    

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: ethereum-nft-tracker <ETHEREUM_BLOCK_NUMBER>")
    } else {
        if let Ok(start_from) = args[1].parse::<u64>() {
            let web3 = Web3::new(
                Http::new(ethereum_rpc).unwrap(),
            );
            let client = EvmClient::new("Ethereum", web3);

            // ERC721
            // ******************************************************************
            // Prepare database to store erc721 metadata
            let database_path: PathBuf = [data_dir, "erc721.db"].iter().collect();
            let db_conn1 = Connection::open(database_path.clone()).unwrap();
            erc721_db::create_tables_if_not_exist(&db_conn1).unwrap();
                
            let mut callback = EthereumErc721EventCallback::new(client.clone());
            let t1 = erc721::track_erc721_events(&client, &db_conn1, start_from, step, None, &mut callback);

            // ERC1155
            // ******************************************************************
            // Prepare database to store erc721 metadata
            let database_path: PathBuf = [data_dir, "erc1155.db"].iter().collect();
            let db_conn2 = Connection::open(database_path.clone()).unwrap();
            erc1155_db::create_tables_if_not_exist(&db_conn2).unwrap();

            let mut callback = EthereumErc1155EventCallback {};
            let t2 = erc1155::track_erc1155_events(&client, &db_conn2, start_from, step, None, &mut callback);

            tokio::join!(t1, t2);
        } else {
            println!("Usage: ethereum-nft-tracker <ETHEREUM_BLOCK_NUMBER>")
        }
    }

    Ok(())
}

