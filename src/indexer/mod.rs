use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{api, db::Database, eth_client};

#[tracing::instrument(skip(rpc, database_url))]
pub async fn start(rpc: impl Into<String>, database_url: &str) -> anyhow::Result<()> {
    let database = Arc::new(Mutex::new(Database::connect(database_url)?));

    let db = Arc::clone(&database);
    tokio::spawn(async move {
        api::run_api(db).await;
    });

    let mut rx = eth_client::connect(rpc).await?;
    println!("Connection established. Background task is listening for new blocks...");

    while let Some(block) = rx.recv().await {
        match block {
            Ok(block) => {
                println!(
                    "Block received: 0x{}",
                    block
                        .block
                        .hash
                        .iter()
                        .map(|x| format!("{:02x}", x))
                        .collect::<String>()
                );
                let result = database.lock().await.insert_block(&block);
                if let Err(e) = result {
                    eprintln!("Error inserting block into database: {}", e);
                }
            }
            Err(e) => println!("Error receiving block: {}", e),
        }
    }

    Ok(())
}
