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

struct PolygonErc721EventCallback {

}

impl Erc721EventCallback for PolygonErc721EventCallback {
    fn on_erc721_event(&self, event: Erc721Event) {
        println!("{:?}", event);
    }
}

struct PolygonErc1155EventCallback {

}

impl Erc1155EventCallback for PolygonErc1155EventCallback {
    fn on_erc1155_event(&self, event: Erc1155Event) {
        println!("{:?}", event);
    }
}

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct PolygonNftTrackerConfig {
    rpc: String,
    step: u64,
}

impl Default for PolygonNftTrackerConfig {
    fn default() -> Self {
        PolygonNftTrackerConfig {
            rpc: "https://rpc-mainnet.matic.network".to_owned(),
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
    // | Linux   | `$XDG_CONFIG_HOME`/rs.polygon-nft-tracker or `$HOME`/.config/rs.polygon-nft-tracker |
    // | macOS   | `$HOME`/Library/Preferences/rs.polygon-nft-tracker                                   |
    // | Windows | `{FOLDERID_RoamingAppData}`\\rs.polygon-nft-tracker\\config                          |
    let cfg: PolygonNftTrackerConfig = confy::load("polygon-nft-tracker")?;
    let polygon_rpc = &cfg.rpc;
    let step = cfg.step;

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: polygon-nft-tracker <POLYGON_BLOCK_NUMBER>")
    } else {
        if let Ok(start_from) = args[1].parse::<u64>() {
            let web3 = Web3::new(
                Http::new(polygon_rpc).unwrap(),
            );
            let client = EvmClient::new("Polygon", web3);
            let client_clone = client.clone();

            tokio::spawn(async move {
                erc721::track_erc721_events(&client_clone, start_from, step, Box::new(PolygonErc721EventCallback {})).await;
            });
            erc1155::track_erc1155_events(&client, start_from, step, Box::new(PolygonErc1155EventCallback {})).await;
        } else {
            println!("Usage: polygon-nft-tracker <POLYGON_BLOCK_NUMBER>")
        }
    }

    Ok(())
}
