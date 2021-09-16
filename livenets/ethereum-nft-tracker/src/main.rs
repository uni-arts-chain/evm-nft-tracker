use directories_next::ProjectDirs;
use std::env;
use std::path::PathBuf;

pub mod sidekiq_helper;
mod println_callbacks;
mod sidekiq_callbacks;

#[macro_use]
extern crate log;

#[macro_use]
extern crate async_trait;


use serde::{Deserialize, Serialize};

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
    if args.len() == 1 {
        println!("Usage: ethereum-nft-tracker <ETHEREUM_BLOCK_NUMBER>")
    } else {
        if args.len() == 2 {
            if let Ok(start_from) = args[1].parse::<u64>() {
                let mut erc721_cb = println_callbacks::EthereumErc721EventCallback {};
                let mut erc1155_cb = println_callbacks::EthereumErc1155EventCallback {};
                nft_events::start_tracking(
                    chain_name,
                    rpc,
                    data_dir,
                    start_from,
                    step,
                    &mut erc721_cb,
                    &mut erc1155_cb,
                )
                .await?;
            } else {
                println!("Usage: ethereum-nft-tracker <ETHEREUM_BLOCK_NUMBER>")
            }
        } else {
            if let Ok(start_from) = args[1].parse::<u64>() {
                let mut erc721_cb = sidekiq_callbacks::EthereumErc721EventCallback {};
                let mut erc1155_cb = sidekiq_callbacks::EthereumErc1155EventCallback {};
                nft_events::start_tracking(
                    chain_name,
                    rpc,
                    data_dir,
                    start_from,
                    step,
                    &mut erc721_cb,
                    &mut erc1155_cb,
                )
                .await?;
            } else {
                println!("Usage: ethereum-nft-tracker <ETHEREUM_BLOCK_NUMBER>")
            }
            
        }
    }
    Ok(())
}
