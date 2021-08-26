#![allow(missing_docs)]
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Web3Error(#[from] web3::Error),
    #[error(transparent)]
    Web3EthabiError(#[from] web3::ethabi::Error),
    #[error(transparent)]
    Web3ContractError(#[from] web3::contract::Error),
    #[error(transparent)]
    RusqliteError(#[from] rusqlite::Error),
    #[error("Other error: {0}")]
    Other(String),
}
