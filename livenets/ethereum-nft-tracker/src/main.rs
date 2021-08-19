use nft_events::{
    Erc721Event, Erc721EventCallback,
    Erc1155Event, Erc1155EventCallback,
};
use directories_next::ProjectDirs;
use std::path::PathBuf;
use std::env;

#[macro_use]
extern crate log;

#[macro_use]
extern crate async_trait;

struct EthereumErc721EventCallback {
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
    env::set_var(
        "RUST_LOG",
        r#"
        ethereum_nft_tracker=info,
		nft_events=debug,
        "#,
    );
    env_logger::init();

    let chain_name = "Ethereum";

    // Data dir
    let app_name = format!("{}-nft-tracker", chain_name.to_lowercase());
    let project = ProjectDirs::from("pro", "uniscan", app_name.as_str()).unwrap();
    let data_dir = project.data_dir().to_str().unwrap();
    info!("DATA & CONFIG DIR : {}", data_dir);

    // Read config from config file
    let config_path: PathBuf = [data_dir, "config.toml"].iter().collect();
    let cfg: EthereumNftTrackerConfig = confy::load_path(config_path)?;
    let rpc = &cfg.rpc;
    let step = cfg.step;
    info!("  {} rpc : {}", chain_name, rpc);
    info!("  Track step : {} blocks", step);
    
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: ethereum-nft-tracker <ETHEREUM_BLOCK_NUMBER>")
    } else {
        if let Ok(start_from) = args[1].parse::<u64>() {
            let mut erc721_cb = EthereumErc721EventCallback {};
            let mut erc1155_cb = EthereumErc1155EventCallback {};
            nft_events::start_tracking(chain_name, rpc, data_dir, start_from, step, &mut erc721_cb, &mut erc1155_cb).await?;
        } else {
            println!("Usage: ethereum-nft-tracker <ETHEREUM_BLOCK_NUMBER>")
        }
    }

    Ok(())
}



