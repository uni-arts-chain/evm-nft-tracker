mod error;
mod evm_client;
pub mod erc721;
pub mod erc1155;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

pub use evm_client::EvmClient;

pub use erc721::{
    Erc721Event, Erc721EventCallback,
};

pub use erc1155::{
    Erc1155Event, Erc1155EventCallback,
};

#[macro_use]
extern crate log;
