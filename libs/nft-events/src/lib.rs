mod error;
mod evm_client;

// erc721
pub mod erc721_db;
pub mod erc721_evm;
pub mod erc721;

// erc1155
pub mod erc1155_db;
pub mod erc1155_evm;
pub mod erc1155;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

pub use evm_client::EvmClient;

pub use erc721_evm::Erc721Event;
pub use erc721::Erc721EventCallback;

pub use erc1155_evm::Erc1155Event;
pub use erc1155::Erc1155EventCallback;

#[macro_use]
extern crate log;

#[macro_use]
extern crate async_trait;

use rusqlite::Connection;
use std::path::PathBuf;
use web3::{
    Web3,
    transports::Http,
};

pub async fn start_tracking(chain_name: &str, rpc: &str, data_dir: &str, start_from: u64, step: u64, erc721_cb: &mut dyn Erc721EventCallback, erc1155_cb: &mut dyn Erc1155EventCallback) -> Result<()> {
    let web3 = Web3::new(
        Http::new(rpc)?,
    );
    let client = EvmClient::new(chain_name.to_owned(), web3);

    // ERC721
    // ******************************************************************
    // Prepare database to store erc721 metadata
    let database_path: PathBuf = [data_dir, "erc721.db"].iter().collect();
    let db_conn1 = Connection::open(database_path.clone())?;
    erc721_db::create_tables_if_not_exist(&db_conn1)?;
        
    let t1 = erc721::track_erc721_events(&client, &db_conn1, start_from, step, None, erc721_cb);

    // ERC1155
    // ******************************************************************
    // Prepare database to store erc721 metadata
    let database_path: PathBuf = [data_dir, "erc1155.db"].iter().collect();
    let db_conn2 = Connection::open(database_path.clone())?;
    erc1155_db::create_tables_if_not_exist(&db_conn2)?;

    let t2 = erc1155::track_erc1155_events(&client, &db_conn2, start_from, step, None, erc1155_cb);

    tokio::join!(t1, t2);

    Ok(())
}

