use nft_events::{Erc1155Event, Erc1155EventCallback, Erc721Event, Erc721EventCallback};
use crate::sidekiq_helper;

pub struct EthereumErc721EventCallback {}

#[async_trait]
impl Erc721EventCallback for EthereumErc721EventCallback {
    async fn on_erc721_event(
        &mut self,
        event: Erc721Event,
        name: String,
        symbol: String,
        token_uri: String,
    ) {
        sidekiq_helper::send_erc721(
            "Ethereum".to_string(),
            event,
            name,
            symbol,
            token_uri,
        );
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
        sidekiq_helper::send_erc1155(
            "Ethereum".to_string(),
            event,
            token_uri,
        );

        Ok(())
    }
}

