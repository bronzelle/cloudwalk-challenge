use std::env;

pub mod db;
pub mod eth_client;
pub mod indexer;
pub mod types;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let rpc_url = env::var("JSON_RPC_API_KEY")
        .expect("JSON_RPC_API_KEY must be set. You can set it in .env file");
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set. You can set it in .env file");
    indexer::start(rpc_url, &database_url).await?;
    Ok(())
}
