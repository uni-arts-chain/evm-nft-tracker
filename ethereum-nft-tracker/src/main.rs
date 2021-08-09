use web3::{
    Web3,
    transports::Http,
};

use nft_events::{
    EvmClient,
    erc721, Erc721Event, Erc721EventCallback,
    erc1155, Erc1155Event, Erc1155EventCallback,
};

struct EthereumErc721EventCallback {

}

impl Erc721EventCallback for EthereumErc721EventCallback {
    fn on_erc721_event(&self, event: Erc721Event) {
        println!("{:?}", event);
    }
}

struct EthereumErc1155EventCallback {

}

impl Erc1155EventCallback for EthereumErc1155EventCallback {
    fn on_erc1155_event(&self, event: Erc1155Event) {
        println!("{:?}", event);
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

    let web3 = Web3::new(
        Http::new("https://mainnet.infura.io/v3/60703fcc6b4e48079cfc5e385ee7af80").unwrap(),
    );
    let client = EvmClient::new("Ethereum", web3);
    let client_clone = client.clone();

    tokio::spawn(async move {
        erc721::track_erc721_events(&client_clone, 12989117, 5, Box::new(EthereumErc721EventCallback {})).await;
    });
    erc1155::track_erc1155_events(&client, 12989117, 5, Box::new(EthereumErc1155EventCallback {})).await;

    Ok(())
}
