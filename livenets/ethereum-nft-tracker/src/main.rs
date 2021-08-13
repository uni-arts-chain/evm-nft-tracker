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

struct EthereumErc721EventCallback {

}

impl Erc721EventCallback for EthereumErc721EventCallback {
    fn on_erc721_event(&mut self, event: Erc721Event) {
        println!("{:?}", event);
    }
}

struct EthereumErc1155EventCallback {

}

impl Erc1155EventCallback for EthereumErc1155EventCallback {
    fn on_erc1155_event(&mut self, event: Erc1155Event) {
        println!("{:?}", event);
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
		nft_events=debug,
        "#,
    );
    env_logger::init();

    // |Platform | Value                                                                                 |
    // | ------- | ------------------------------------------------------------------------------------- |
    // | Linux   | `$XDG_CONFIG_HOME`/rs.ethereum-nft-tracker or `$HOME`/.config/rs.ethereum-nft-tracker |
    // | macOS   | `$HOME`/Library/Preferences/rs.ethereum-nft-tracker                                   |
    // | Windows | `{FOLDERID_RoamingAppData}`\\rs.ethereum-nft-tracker\\config                          |
    let cfg: EthereumNftTrackerConfig = confy::load("ethereum-nft-tracker")?;
    let ethereum_rpc = &cfg.rpc;
    let step = cfg.step;

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: ethereum-nft-tracker <ETHEREUM_BLOCK_NUMBER>")
    } else {
        if let Ok(start_from) = args[1].parse::<u64>() {
            let web3 = Web3::new(
                Http::new(ethereum_rpc).unwrap(),
            );
            let client = EvmClient::new("Ethereum", web3);
            let client_clone = client.clone();

            tokio::spawn(async move {
                let mut callback = EthereumErc721EventCallback {};
                erc721::track_erc721_events(&client_clone, start_from, step, None, &mut callback).await;
            });

            let mut callback = EthereumErc1155EventCallback {};
            erc1155::track_erc1155_events(&client, start_from, step, None, &mut callback).await;
        } else {
            println!("Usage: ethereum-nft-tracker <ETHEREUM_BLOCK_NUMBER>")
        }
    }

    Ok(())
}
