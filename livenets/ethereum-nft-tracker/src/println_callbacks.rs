use nft_events::{Erc1155Event, Erc1155EventCallback, Erc721Event, Erc721EventCallback};

pub struct EthereumErc721EventCallback {}

#[async_trait]
impl Erc721EventCallback for EthereumErc721EventCallback {
    async fn on_erc721_event(
        &mut self,
        event: Erc721Event,
        name: String,
        symbol: String,
        total_supply: Option<u128>,
        token_uri: String,
    ) -> nft_events::Result<()> {
        println!("------------------------------------------------------------------------------------------");
        println!("event: {:?}", event);
        println!(
            "name: {:?}, symbol: {:?}, token_uri: {:?}",
            name, symbol, token_uri
        );
        println!("total_supply: {:?}", total_supply);

        Ok(())
    }
}

pub struct EthereumErc1155EventCallback {}

#[async_trait]
impl Erc1155EventCallback for EthereumErc1155EventCallback {
    async fn on_erc1155_event(
        &mut self,
        event: Erc1155Event,
        token_uri: String,
    ) -> nft_events::Result<()> {
        println!("++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++");
        println!("event: {:?}", event);
        println!("token_uri: {:?}", token_uri);

        Ok(())
    }
}

