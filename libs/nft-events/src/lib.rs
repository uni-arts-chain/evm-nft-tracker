#![warn(missing_docs)]
//! This library was used to discover EVM-based NFTs, including ERC-721 and ERC-1155 NFTs.
//! It discovers NFTs by listening to the transfer events of ERC-721 and ERC-1155 contracts.
//! It consider only visual NFTs. If a NFT contract has no metadata, it will be ignored.
mod error;
mod evm_client;

// erc721
pub mod erc721;
pub mod erc721_evm;

// erc1155
pub mod erc1155;
pub mod erc1155_evm;

pub use error::Error;
/// The lib's result
pub type Result<T> = std::result::Result<T, Error>;

pub use evm_client::EvmClient;

pub use erc721::Erc721EventCallback;
pub use erc721_evm::Erc721Event;

pub use erc1155::Erc1155EventCallback;
pub use erc1155_evm::Erc1155Event;

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

    // ERC721
    // ******************************************************************
    let t1 = erc721::track_erc721_events(&client, start_from, step, None, erc721_cb);

    // ERC1155
    // ******************************************************************
    let t2 = erc1155::track_erc1155_events(&client, start_from, step, None, erc1155_cb);

    tokio::join!(t1, t2);

    Ok(())
}
