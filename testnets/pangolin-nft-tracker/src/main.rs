use web3::{
    Web3,
    transports::Http,
};
use nft_events::{
    EvmClient,
    erc721, Erc721Event, Erc721EventCallback,
    erc1155, Erc1155Event, Erc1155EventCallback,
};
use std::env;

#[macro_use]
extern crate async_trait;

struct PangolinErc721EventCallback {

}

#[async_trait]
impl Erc721EventCallback for PangolinErc721EventCallback {
    async fn on_erc721_event(&mut self, event: Erc721Event, name: Option<String>, symbol: Option<String>, token_uri: Option<String>) -> nft_events::Result<()> {
        println!("{:?}", event);
        Ok(())
    }
}

struct PangolinErc1155EventCallback {

}

#[async_trait]
impl Erc1155EventCallback for PangolinErc1155EventCallback {
    async fn on_erc1155_event(&mut self, event: Erc1155Event, token_uri: String) -> nft_events::Result<()> {
        println!("{:?}", event);
        Ok(())
    }
}

use serde::{Serialize, Deserialize};

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
    std::env::set_var(
        "RUST_LOG",
        r#"
        nft_events=debug,
        "#,
        );
    env_logger::init();

    // |Platform | Value                                                                                 |
    // | ------- | ------------------------------------------------------------------------------------- |
    // | Linux   | `$XDG_CONFIG_HOME`/rs.pangolin-nft-tracker or `$HOME`/.config/rs.pangolin-nft-tracker |
    // | macOS   | `$HOME`/Library/Preferences/rs.pangolin-nft-tracker                                   |
    // | Windows | `{FOLDERID_RoamingAppData}`\\rs.pangolin-nft-tracker\\config                          |
    let cfg: PangolinNftTrackerConfig = confy::load("pangolin-nft-tracker")?;
    let rpc = &cfg.rpc;
    let step = cfg.step;

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: pangolin-nft-tracker <PANGOLIN_BLOCK_NUMBER>")
    } else {
        if let Ok(start_from) = args[1].parse::<u64>() {
            let web3 = Web3::new(
                Http::new(rpc).unwrap(),
                );
            let client = EvmClient::new("Pangolin", web3);
            let client_clone = client.clone();

            // tokio::spawn(async move {
            //     let mut callback = PangolinErc721EventCallback {};
            //     erc721::track_erc721_events(&client_clone, start_from, step, None, &mut callback).await;
            // });

            // let mut callback = PangolinErc1155EventCallback {};
            // erc1155::track_erc1155_events(&client, start_from, step, None, &mut callback).await;
        } else {
            println!("Usage: pangolin-nft-tracker <PANGOLIN_BLOCK_NUMBER>")
        }
    }

    Ok(())
}
