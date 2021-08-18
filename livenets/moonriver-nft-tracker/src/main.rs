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

struct MoonriverErc721EventCallback {

}

#[async_trait]
impl Erc721EventCallback for MoonriverErc721EventCallback {
    async fn on_erc721_event(&mut self, event: Erc721Event, name: Option<String>, symbol: Option<String>, token_uri: Option<String>) -> nft_events::Result<()> {
        println!("{:?}", event);
        Ok(())
    }
}

struct MoonriverErc1155EventCallback {

}

impl Erc1155EventCallback for MoonriverErc1155EventCallback {
    fn on_erc1155_event(&mut self, event: Erc1155Event) {
        println!("{:?}", event);
    }
}

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct MoonriverNftTrackerConfig {
    rpc: String,
    step: u64,
}

impl Default for MoonriverNftTrackerConfig {
    fn default() -> Self {
        MoonriverNftTrackerConfig {
            rpc: "https://rpc.moonriver.moonbeam.network".to_owned(),
            step: 6,
        }
    }
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var(
        "RUST_LOG",
        r#"
		nft_events=info,
        "#,
    );
    env_logger::init();

    // |Platform | Value                                                                                 |
    // | ------- | ------------------------------------------------------------------------------------- |
    // | Linux   | `$XDG_CONFIG_HOME`/rs.moonriver-nft-tracker or `$HOME`/.config/rs.moonriver-nft-tracker |
    // | macOS   | `$HOME`/Library/Preferences/rs.moonriver-nft-tracker                                   |
    // | Windows | `{FOLDERID_RoamingAppData}`\\rs.moonriver-nft-tracker\\config                          |
    let cfg: MoonriverNftTrackerConfig = confy::load("moonriver-nft-tracker")?;
    let moonriver_rpc = &cfg.rpc;
    let step = cfg.step;

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: moonriver-nft-tracker <MOONRIVER_BLOCK_NUMBER>")
    } else {
        if let Ok(start_from) = args[1].parse::<u64>() {
            let web3 = Web3::new(
                Http::new(moonriver_rpc).unwrap(),
            );
            let client = EvmClient::new("Moonriver", web3);
            let client_clone = client.clone();

            // tokio::spawn(async move {
            //     let mut callback = MoonriverErc721EventCallback {};
            //     erc721::track_erc721_events(&client_clone, start_from, step, None, &mut callback).await;
            // });

            // let mut callback = MoonriverErc1155EventCallback {};
            // erc1155::track_erc1155_events(&client, start_from, step, None, &mut callback).await;
        } else {
            println!("Usage: moonriver-nft-tracker <MOONRIVER_BLOCK_NUMBER>")
        }
    }

    Ok(())
}
