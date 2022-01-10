// #![feature(backtrace)]
#![warn(missing_docs)]
//! This library was used to discover EVM-based NFTs, including ERC-721 and ERC-1155 NFTs.
//! It discovers NFTs by listening to the transfer events of ERC-721 and ERC-1155 contracts.
//! It consider only visual NFTs. If a NFT contract has no metadata, it will be ignored.
mod error;
mod evm_client;

/// helper to get evm nft events
pub mod events_helper;
pub use events_helper::Event;
pub use events_helper::Erc721Event;
pub use events_helper::Erc1155Event;

/// the events tracker
pub mod tracker;
pub use tracker::Erc721EventCallback;
pub use tracker::Erc1155EventCallback;

pub use error::Error;
/// The lib's result
pub type Result<T> = std::result::Result<T, Error>;

pub use evm_client::EvmClient;


#[macro_use]
extern crate log;

#[macro_use]
extern crate async_trait;

use web3::{transports::Http, Web3};

/// This is the entry function for this library.
/// This function wraps the logic for tracking erc721 and erc1155 transfers.
pub async fn start_tracking(
    chain_name: &str,
    rpc: &str,
    start_from: u64,
    step: u64,
    erc721_cb: &mut dyn Erc721EventCallback,
    erc1155_cb: &mut dyn Erc1155EventCallback,
) -> Result<()> {
    let web3 = Web3::new(Http::new(rpc)?);
    let client = EvmClient::new(chain_name.to_owned(), web3);

    tracker::track_events(&client, start_from, step, None, erc721_cb, erc1155_cb).await;

    Ok(())
}


