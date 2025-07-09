use crate::eth_client;

pub async fn start(rpc: impl Into<String>) -> anyhow::Result<()> {
    let mut rx = eth_client::connect(rpc).await?;
    println!("Connection established. Background task is listening for new blocks...");

    while let Some(block) = rx.recv().await {
        match block {
            Ok(block) => println!(
                "New block received from channel: Block #{:?} with hash {:?}",
                block.block.number, block.block.hash
            ),
            Err(e) => println!("Error receiving block: {}", e),
        }
    }

    Ok(())
}
