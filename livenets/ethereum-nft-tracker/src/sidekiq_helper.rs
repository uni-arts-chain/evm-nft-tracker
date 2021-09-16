use nft_events::{Erc1155Event, Erc721Event};
use sidekiq::{create_redis_pool, Client, ClientOpts, Job, JobOpts};
use serde_json;

use serde_json::value::Value;

pub fn send_erc721(
    blockchain: String,
    event: Erc721Event,
    name: String,
    symbol: String,
    token_uri: String,
    total_supply: Option<u128>,
) {
    if event.block_number.is_some() && event.transaction_hash.is_some() {
        let block_number = event.block_number.unwrap();
        let address = format!("{:?}", event.address);
        let transaction_hash = format!("{:?}", event.transaction_hash.unwrap());
        let from = format!("{:?}", event.from);
        let to = format!("{:?}", event.to);
        let token_id = event.token_id.to_string();

        let job = build_erc721_job(
            blockchain, 
            block_number, 
            address, 
            transaction_hash, 
            from, 
            to, 
            token_id, 
            token_uri, 
            name, 
            symbol, 
            total_supply
        );

        push(job);
    }
}

fn build_erc721_job(
    blockchain: String,
    block_number: u64,
    address: String,
    transaction_hash: String,
    from: String,
    to: String,
    token_id: String,
    token_uri: String,
    name: String,
    symbol: String,
    total_supply: Option<u128>,
) -> Job {
    let class = "ProcessErc721EventWorker".to_string();

    let ts = if let Some(total_supply) = total_supply {
        Value::from(total_supply as u64)
    } else {
        Value::Null
    };
    let value = serde_json::json!({
        "blockchain": blockchain,
        "block_number": block_number,
        "address": address,
        "transaction_hash": transaction_hash,
        "from": from,
        "to": to,
        "token_id": token_id,
        "token_uri": token_uri,
        "name": name,
        "symbol": symbol,
        "total_supply": ts, 
    });
    let args: Vec<Value> = vec![value];

    let job_opts = JobOpts {
        queue: "erc721_events".to_string(),
        ..Default::default()
    };
    Job::new(class, args, job_opts)
}

pub fn send_erc1155(
    blockchain: String,
    event: Erc1155Event,
    token_uri: String,
) {
    if event.block_number.is_some() && event.transaction_hash.is_some() {
        let block_number = event.block_number.unwrap();
        let address = format!("{:?}", event.address);
        let transaction_hash = format!("{:?}", event.transaction_hash.unwrap());
        let from = format!("{:?}", event.from);
        let to = format!("{:?}", event.to);
        let token_id = event.token_id.to_string();
        let amount = event.amount.as_u128();

        let job = build_erc1155_job(
            blockchain, 
            block_number, 
            address, 
            transaction_hash, 
            from, 
            to, 
            token_id, 
            amount, 
            token_uri, 
        );

        push(job);
    }
}

fn build_erc1155_job(
    blockchain: String,
    block_number: u64,
    address: String,
    transaction_hash: String,
    from: String,
    to: String,
    token_id: String,
    amount: u128,
    token_uri: String,
) -> Job {
    let class = "ProcessErc1155EventWorker".to_string();

    let value = serde_json::json!({
        "blockchain": blockchain,
        "block_number": block_number,
        "address": address,
        "transaction_hash": transaction_hash,
        "from": from,
        "to": to,
        "token_id": token_id,
        "token_uri": token_uri,
        "amount": amount as u64,
    });
    let args: Vec<Value> = vec![value];

    let job_opts = JobOpts {
        queue: "erc1155_events".to_string(),
        ..Default::default()
    };
    Job::new(class, args, job_opts)
}

fn get_client() -> Client {
    let client_opts = ClientOpts {
        namespace: None,
    };
    let pool = create_redis_pool().unwrap();
    Client::new(pool, client_opts)
}

fn push(job: Job) {
    let client = get_client();
    match client.push(job) {
        Ok(_) => {}
        Err(err) => {
            println!("Sidekiq push failed: {}", err);
        }
    }
}

#[test]
fn test_build_erc721_job() {
    let job = build_erc721_job(
        "Ethereum".to_string(),
        123456,
        "0xca0d36c67a0c1bf6b28e76fb8c2188c31b87d152".to_string(),
        "0x42ac0589bf82ccff3358859728890c520f616ac5e184cc685113732880df0825".to_string(),
        "0x8628ff3ac814ee8937c10860b85d55e6aa67cfa2".to_string(),
        "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2".to_string(),
        "1111".to_string(),
        "https://token_uri".to_string(),
        "Hello".to_string(),
        "HL".to_string(),
        Some(1234),
    );

    push(job)
}

#[test]
fn test_build_erc1155_job() {
    let job = build_erc1155_job(
        "Ethereum".to_string(),
        123456,
        "0xca0d36c67a0c1bf6b28e76fb8c2188c31b87d152".to_string(),
        "0x42ac0589bf82ccff3358859728890c520f616ac5e184cc685113732880df0825".to_string(),
        "0x8628ff3ac814ee8937c10860b85d55e6aa67cfa2".to_string(),
        "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2".to_string(),
        "1111".to_string(),
        1234,
        "https://token_uri".to_string(),
    );

    push(job)
}
