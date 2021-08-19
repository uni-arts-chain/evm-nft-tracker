use crate::Result;

use rusqlite::{
    Connection, params
};

pub fn create_tables_if_not_exist(conn: &Connection) -> Result<()> {
    conn.execute(
        "create table if not exists erc1155_collections (
             id integer primary key,
             address text not null unique
         )",
        [],
    )?;
    conn.execute(
        "create table if not exists erc1155_tokens (
             id integer primary key,
             token_id text not null,
             collection_id integer not null references erc721_collections(id),
             token_uri text
         )",
        [],
    )?;

    Ok(()) 
}

// id, address
pub fn get_collection_from_db(conn: &Connection, address: &str) -> Result<Option<(usize, String)>> {
    let sql = format!("SELECT * from erc1155_collections where address='{}'", address);
    let mut stmt = conn.prepare(
        sql.as_str()
    )?;

    match stmt.query_row([], |row| {
        Ok(
            (
                row.get(0)?,
                row.get(1)?,
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

pub fn add_collection_to_db(conn: &Connection, address: String) -> Result<usize> {
    conn.execute(
        "INSERT INTO erc1155_collections (address) values (?1)",
        params![&address],
    )?;
    let id = conn.last_insert_rowid() as usize;
    Ok(id)
}

// token_id here is the token_id in contract
pub fn get_token_from_db(conn: &Connection, collection_id: usize, token_id: &str) -> Result<Option<(usize, String, usize, Option<String>)>> {
    let sql = format!("SELECT * from erc1155_tokens where collection_id={} and token_id='{}'", collection_id, token_id);
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

pub fn add_token_to_db(conn: &Connection, token_id: String, collection_id: usize, token_uri: Option<String>) -> Result<usize> {
    if token_uri.is_some() {
        conn.execute(
            "INSERT INTO erc1155_tokens (token_id, collection_id, token_uri) values (?1, ?2, ?3)",
            params![&token_id, &collection_id, &token_uri],
        )?;
    } else {
        conn.execute(
            "INSERT INTO erc1155_tokens (token_id, collection_id) values (?1, ?2)",
            params![&token_id, &collection_id],
        )?;
    }

    let id = conn.last_insert_rowid() as usize;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use web3::types::{H256, H160, Log, U256};

    #[tokio::test]
    async fn test_get_collection_from_db() {
        std::fs::remove_file("./erc1155_test1.db");
        
        let conn = Connection::open("./erc1155_test1.db").unwrap();
        create_tables_if_not_exist(&conn).unwrap();

        let address = format!("{:?}", H160::zero());

        let result = get_collection_from_db(&conn, &address).unwrap();
        assert_eq!(None, result);

        add_collection_to_db(&conn, address.clone()).unwrap();
        let result = get_collection_from_db(&conn, &address).unwrap();
        assert_eq!(Some((1, "0x0000000000000000000000000000000000000000".to_string())), result);

        std::fs::remove_file("./erc1155_test1.db");
    }

    #[tokio::test]
    async fn test_get_token_from_db() {
        std::fs::remove_file("./erc1155_test2.db");

        let conn = Connection::open("./erc1155_test2.db").unwrap();
        create_tables_if_not_exist(&conn).unwrap();

        let token_id = "10276";
        
        // there is no token with the token_id
        let token = get_token_from_db(&conn, 1usize, token_id).unwrap();
        assert_eq!(None, token);

        // add a collection first
        let address = "0x76be3b62873462d2142405439777e971754e8e77";
        let collection_id = add_collection_to_db(&conn, address.to_string()).unwrap();

        // add the token
        let token_uri = Some("ipfs://ipfs/QmXTSZ7ag9yzQFkvUrZ5qr7KuCUtW2jeDiYJrJ8s7TpSNb".to_owned());
        let id = add_token_to_db(&conn, token_id.to_owned(), collection_id, token_uri.clone()).unwrap();

        // the token is already in the database
        let token = get_token_from_db(&conn, collection_id, token_id).unwrap();
        assert_eq!(Some((id, token_id.to_owned(), collection_id, token_uri)), token);

        std::fs::remove_file("./erc1155_test2.db");
    }
}
