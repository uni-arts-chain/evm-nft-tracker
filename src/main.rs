use web3::{
    Web3,
    transports::Http,
};
use web3::types::{H160, H256, U256};

mod error;
mod evm_client;
pub mod erc721;
pub mod erc1155;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;
pub use evm_client::EvmClient;
pub use crate::erc721::{
    Erc721Event, Erc721EventCallback
};
pub use crate::erc1155::{
    Erc1155Event, Erc1155EventCallback
};

#[macro_use]
extern crate log;

struct DefaultErc721EventCallback {

}

impl Erc721EventCallback for DefaultErc721EventCallback {
    fn on_erc721_event(&self, event: Erc721Event) {
        println!("{:?}", event);
    }
}

struct DefaultErc1155EventCallback {

}

impl Erc1155EventCallback for DefaultErc1155EventCallback {
    fn on_erc1155_event(&self, event: Erc1155Event) {
        println!("{:?}", event);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var(
        "RUST_LOG",
        r#"
		evm_nft=debug,
        "#,
    );
    env_logger::init();


    // // Ethereum
    // let web3 = Web3::new(
    //     Http::new("https://mainnet.infura.io/v3/60703fcc6b4e48079cfc5e385ee7af80").unwrap(),
    // );
    // let client = EvmClient::new("Ethereum", web3);
    // erc721::track_erc721_events(&client, 12989117, 5, Box::new(DefaultErc721Callback {})).await;
    // erc1155::track_erc1155_events(&client, 12989117, 5, Box::new(DefaultErc1155EventCallback {})).await;

    // Pangolin 
    let web3 = Web3::new(
        Http::new("http://pangolin-rpc.darwinia.network").unwrap(),
    );
    let client = EvmClient::new("Pangolin", web3);
    let client_clone = client.clone();
    tokio::spawn(async move {
        erc721::track_erc721_events(&client_clone, 185852, 5, Box::new(DefaultErc721EventCallback {})).await;
    });
    erc1155::track_erc1155_events(&client, 185852, 5, Box::new(DefaultErc1155EventCallback {})).await;


    // let from = 12967549;
    // let   to = 12968550;
    // // let from = 176250;
    // // let   to = 176478;
    // match erc721::get_events(&client, from, to).await {
    //     Ok(events) => {
    //         for event in events {
    //             println!("{:?}", event);
    //             // let metadata = client.get_erc721_metadata(event.address, event.token_id).await?;
    //             // println!("{:?}", metadata);
    //         }
    //     },
    //     Err(err) => {
    //         println!("{:?}", err);
    //     },
    // }

    // let events = erc1155::get_events(&client, 176250, 176258).await?;
    // println!("{:?}", events);

    Ok(())
}
