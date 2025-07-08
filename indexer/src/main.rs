use std::env;

use eth_client::connect;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let rpc_url = env::var("JSON_RPC_API_KEY").expect("JSON_RPC_API_KEY must be set in .env file");

    let (tx, mut rx) = mpsc::channel(100);

    println!("Connecting to Ethereum node at {}...", rpc_url);
    connect(rpc_url, tx).await?;
    println!("Connection established. Background task is listening for new blocks...");

    while let Some(header) = rx.recv().await {
        println!(
            "New block received from channel: Block #{} with hash {}",
            header.number, header.hash
        );
    }
    Ok(())
}
