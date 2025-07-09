use std::env;

pub mod eth_client;
pub mod indexer;
pub mod types;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let rpc_url = env::var("JSON_RPC_API_KEY").expect("JSON_RPC_API_KEY must be set in .env file");
    indexer::start(rpc_url).await?;
    Ok(())
}
