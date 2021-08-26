//! This module defines several functions to access ERC721 metadata in the database.
use crate::Result;

use rusqlite::{
    Connection, params
};

/// This function is used to create the tables used to store the ERC721 metadatas
pub fn create_tables_if_not_exist(conn: &Connection) -> Result<()> {
    conn.execute(
        "create table if not exists erc721_collections (
             id integer primary key,
             address text not null unique,
             name text,
             symbol text
         )",
        [],
    )?;
    conn.execute(
        "create table if not exists erc721_tokens (
             id integer primary key,
             token_id text not null,
             collection_id integer not null references erc721_collections(id),
             token_uri text
         )",
        [],
    )?;

    Ok(()) 
}

/// Get the name and symbol of a ERC721 contract.
/// The returned tuple is (_, contract_address, name, symbol), the name and symbol may be None
/// if the contract has no name and symbol.
pub fn get_collection_from_db(conn: &Connection, address: &str) -> Result<Option<(usize, String, Option<String>, Option<String>)>> {
    let sql = format!("SELECT * from erc721_collections where address='{}'", address);
    let mut stmt = conn.prepare(
        sql.as_str()
    )?;

    match stmt.query_row([], |row| {
        Ok(
            (
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            )
        )
    }) {
        Ok(collection) => Ok(Some(collection)),
        Err(_err@rusqlite::Error::QueryReturnedNoRows) => {
            Ok(None)
        },
        Err(err) => {
            Err(err)?
        }
    }
}

/// Save the name and symbol of a ERC721 contract to database.
/// It returns the database id.
pub fn add_collection_to_db(conn: &Connection, address: String, name: Option<String>, symbol: Option<String>) -> Result<usize> {
    if name.is_some() && symbol.is_some() {
        conn.execute(
            "INSERT INTO erc721_collections (address, name, symbol) values (?1, ?2, ?3)",
            params![&address, &name, &symbol],
        )?;
    } else {
        conn.execute(
            "INSERT INTO erc721_collections (address) values (?1)",
            params![&address],
        )?;
    }
    let id = conn.last_insert_rowid() as usize;
    Ok(id)
}

// pub fn save_collection_if_not_exists(conn: &Connection, event: &Erc721Event, metadata: Option<(String, String, String)>) -> Result<(usize, String, Option<String>, Option<String>)> {
//     let collection_result = erc721::get_collection_from_db(conn, event.address.clone())?;
//     let collection = collection_result.unwrap_or_else(|| {
//         if let Some(metadata) = metadata {
//             add_collection_to_db(&self.db_conn, event.address.clone(), metadata.0, metadata.1)?
//         } else {
//             add_collection_to_db(&self.db_conn, event.address.clone(), None, None)?
//         }
//     });
//     Ok(collection)
// }

// pub fn save_token_if_not_exists(conn: &Connection, event: &Erc721Event, metadata: Option<(String, String, String)>) -> Result<(usize, String, Option<String>)> {
// }

/// Get the token_uri of a ERC721 token from database.
/// token_id here is the `token_id` in contract.
/// The returned tuple is (_, contract_address, _, token_uri)
pub fn get_token_from_db(conn: &Connection, collection_id: usize, token_id: &str) -> Result<Option<(usize, String, usize, Option<String>)>> {
    let sql = format!("SELECT * from erc721_tokens where collection_id={} and token_id='{}'", collection_id, token_id);
    let mut stmt = conn.prepare(
        sql.as_str()
    )?;

    match stmt.query_row([], |row| {
        Ok(
            (
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            )
        )
    }) {
        Ok(token) => Ok(Some(token)),
        Err(_err@rusqlite::Error::QueryReturnedNoRows) => {
            Ok(None)
        },
        Err(err) => {
            Err(err)?
        }
    }
}

/// Save the token uri to database.
/// It returns the database id.
pub fn add_token_to_db(conn: &Connection, token_id: String, collection_id: usize, token_uri: Option<String>) -> Result<usize> {
    if token_uri.is_some() {
        conn.execute(
            "INSERT INTO erc721_tokens (token_id, collection_id, token_uri) values (?1, ?2, ?3)",
            params![&token_id, &collection_id, &token_uri],
            // &[&token_id, &collection_id.to_string(), &the_token_uri],
        )?;
    } else {
        conn.execute(
            "INSERT INTO erc721_tokens (token_id, collection_id) values (?1, ?2)",
            params![&token_id, &collection_id],
        )?;
    }

    let id = conn.last_insert_rowid() as usize;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use web3::types::{H160, U256};

    #[tokio::test]
    async fn test_get_collection_from_db() {
        let conn = Connection::open("./test1.db").unwrap();
        create_tables_if_not_exist(&conn).unwrap();

        let address = format!("{:?}", H160::zero());

        let result = get_collection_from_db(&conn, &address).unwrap();
        assert_eq!(None, result);

        add_collection_to_db(&conn, address.clone(), None, None).unwrap();
        let result = get_collection_from_db(&conn, &address).unwrap();
        assert_eq!(Some((1, "0x0000000000000000000000000000000000000000".to_string(), None, None)), result);

        std::fs::remove_file("./test1.db").unwrap();
    }

    #[tokio::test]
    async fn test_add_collection_to_db() {
        let conn = Connection::open("./test2.db").unwrap();
        create_tables_if_not_exist(&conn).unwrap();

        // 1
        let address = "0xC5c1C9c3cEA2f4A68E540b18e63310310FD8af57";

        let result = get_collection_from_db(&conn, address).unwrap();
        assert_eq!(None, result);

        add_collection_to_db(&conn, address.to_string(), None, None).unwrap();
        let result = get_collection_from_db(&conn, address).unwrap();
        assert_eq!(Some((1usize, address.to_string(), None, None)), result);

        // 2
        let address = "0xa7d8d9ef8d8ce8992df33d8b8cf4aebabd5bd270";

        let result = get_collection_from_db(&conn, address).unwrap();
        assert_eq!(None, result);

        add_collection_to_db(&conn, address.to_string(), Some("Art Blocks".to_owned()), Some("BLOCKS".to_owned())).unwrap();
        let result = get_collection_from_db(&conn, address).unwrap();
        assert_eq!(Some((2usize, address.to_string(), Some("Art Blocks".to_owned()), Some("BLOCKS".to_owned()))), result);

        std::fs::remove_file("./test2.db").unwrap();
    }

    #[tokio::test]
    async fn test_add_token_to_db() {
        let conn = Connection::open("./test3.db").unwrap();
        create_tables_if_not_exist(&conn).unwrap();

        let address = "0xa7d8d9ef8d8ce8992df33d8b8cf4aebabd5bd270";
        let collection_id = add_collection_to_db(&conn, address.to_string(), Some("Art Blocks".to_owned()), Some("BLOCKS".to_owned())).unwrap();

        let token_id = U256::from_dec_str("129000030").unwrap();
        let id = add_token_to_db(&conn, token_id.to_string(), collection_id, Some("https://api.artblocks.io/token/129000030".to_owned())).unwrap();
        assert_eq!(1usize, id);

        // test u256 can be save as string correctly
        let token = get_token_from_db(&conn, collection_id, &token_id.to_string()).unwrap().unwrap();

        assert_eq!("129000030".to_string(), token.1);

        std::fs::remove_file("./test3.db").unwrap();
    }

    #[tokio::test]
    async fn test_get_token_from_db() {
        let conn = Connection::open("./test4.db").unwrap();
        create_tables_if_not_exist(&conn).unwrap();

        let token_id = "129000030";
        let token = get_token_from_db(&conn, 1usize, token_id).unwrap();
        assert_eq!(None, token);

        let address = "0xa7d8d9ef8d8ce8992df33d8b8cf4aebabd5bd270";
        let collection_id = add_collection_to_db(&conn, address.to_string(), Some("Art Blocks".to_owned()), Some("BLOCKS".to_owned())).unwrap();
        let token_uri = Some("https://api.artblocks.io/token/129000030".to_owned());
        let id = add_token_to_db(&conn, token_id.to_owned(), collection_id, token_uri.clone()).unwrap();
        let token = get_token_from_db(&conn, collection_id, token_id).unwrap();
        assert_eq!(Some((id, token_id.to_owned(), collection_id, token_uri)), token);

        std::fs::remove_file("./test4.db").unwrap();
    }
}
