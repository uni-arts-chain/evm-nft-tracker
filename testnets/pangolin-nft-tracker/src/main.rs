use directories_next::ProjectDirs;
use nft_events::{Erc1155Event, Erc1155EventCallback, Erc721Event, Erc721EventCallback};
use std::env;
use std::path::PathBuf;

#[macro_use]
extern crate log;

#[macro_use]
extern crate async_trait;

struct PangolinErc721EventCallback {}

#[async_trait]
impl Erc721EventCallback for PangolinErc721EventCallback {
    async fn on_erc721_event(
        &mut self,
        event: Erc721Event,
        name: String,
        symbol: String,
        token_uri: String,
    ) {
        println!("------------------------------------------------------------------------------------------");
        println!("event: {:?}", event);
        println!(
            "name: {:?}, symbol: {:?}, token_uri: {:?}",
            name, symbol, token_uri
        );
    }
}

struct PangolinErc1155EventCallback {}

#[async_trait]
impl Erc1155EventCallback for PangolinErc1155EventCallback {
    async fn on_erc1155_event(
        &mut self,
        event: Erc1155Event,
        token_uri: String,
    ) {
        println!("++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++");
        println!("event: {:?}", event);
        println!("token_uri: {:?}", token_uri);
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct PangolinNftTrackerConfig {
    rpc: String,
    step: u64,
}

impl Default for PangolinNftTrackerConfig {
    fn default() -> Self {
        PangolinNftTrackerConfig {
            rpc: "http://pangolin-rpc.darwinia.network".to_owned(),
            step: 6,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env::set_var(
        "RUST_LOG",
        r#"
        pangolin_nft_tracker=info,
		nft_events=debug,
        "#,
    );
    env_logger::init();

    let chain_name = "Pangolin";

    // Data dir
    let app_name = format!("{}-nft-tracker", chain_name.to_lowercase());
    let project = ProjectDirs::from("pro", "uniscan", app_name.as_str()).unwrap();
    let data_dir = project.data_dir().to_str().unwrap();
    info!("DATA & CONFIG DIR : {}", data_dir);

    // Read config from config file
    let config_path: PathBuf = [data_dir, "config.toml"].iter().collect();
    let cfg: PangolinNftTrackerConfig = confy::load_path(config_path)?;
    let rpc = &cfg.rpc;
    let step = cfg.step;
    info!("  {} rpc : {}", chain_name, rpc);
    info!("  Track step : {} blocks", step);

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: pangolin-nft-tracker <ETHEREUM_BLOCK_NUMBER>")
    } else {
        if let Ok(start_from) = args[1].parse::<u64>() {
            let mut erc721_cb = PangolinErc721EventCallback {};
            let mut erc1155_cb = PangolinErc1155EventCallback {};
            nft_events::start_tracking(
                chain_name,
                rpc,
                start_from,
                step,
                &mut erc721_cb,
                &mut erc1155_cb,
            )
            .await?;
        } else {
            println!("Usage: pangolin-nft-tracker <ETHEREUM_BLOCK_NUMBER>")
        }
    }

    Ok(())
}
