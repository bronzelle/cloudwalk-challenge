use alloy_provider::{DynProvider, Provider, ProviderBuilder, WsConnect};
use alloy_rpc_types_eth::{BlockTransactions, Filter, Header};
use futures_util::StreamExt;
use tokio::sync::mpsc::{self, Receiver};

use crate::types::Info;

pub async fn connect(rpc: impl Into<String>) -> anyhow::Result<Receiver<anyhow::Result<Info>>> {
    let provider = ProviderBuilder::new()
        .connect_ws(WsConnect::new(rpc))
        .await?
        .erased();

    let (sender, receiver) = mpsc::channel(100);

    tokio::spawn(async move {
        let sub = match provider.subscribe_blocks().await {
            Ok(sub) => sub,
            Err(e) => {
                eprintln!("Failed to subscribe to blocks: {}", e);
                return;
            }
        };
        let mut stream = sub.into_stream();
        while let Some(header) = stream.next().await {
            let info = get_block_info(&provider, header).await;
            if sender.send(info).await.is_err() {
                eprintln!("Receiver dropped. Stopping block listener.");
                break;
            }
        }
    });
    Ok(receiver)
}

async fn get_block_info(provider: &DynProvider, header: Header) -> anyhow::Result<Info> {
    let filter = Filter::new().at_block_hash(header.hash);
    let (block_result, logs_result) = tokio::join!(
        provider.get_block_by_hash(header.hash).hashes(),
        provider.get_logs(&filter)
    );

    let (block, logs) = (
        block_result?.ok_or_else(|| anyhow::anyhow!("Block not found"))?,
        logs_result?,
    );

    let transactions = match block.transactions {
        BlockTransactions::Hashes(transactions) => Ok(transactions),
        _ => Err(anyhow::anyhow!("Block transactions are not full")),
    }?;

    Ok(Info {
        block: header.into(),
        logs: logs.iter().map(|log| log.clone().into()).collect(),
        transactions: transactions
            .iter()
            .map(|hash| hash.clone().into())
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use std::env;

    use alloy::primitives::b256;

    use super::*;

    async fn provider() -> DynProvider {
        dotenvy::dotenv().ok();
        let rpc = env::var("JSON_RPC_API_KEY").expect("JSON_RPC_API_KEY must be set in .env file");

        ProviderBuilder::new()
            .connect_ws(WsConnect::new(rpc))
            .await
            .expect("Connection to the provider failed")
            .erased()
    }

    #[tokio::test]
    async fn test_get_block_data() {
        let provider = provider().await;
        let hash = b256!("0xe3d57e27e5300f22504990aec927d0cd055313adca707092ea34a557a0c501c7");
        let header = Header::<alloy::consensus::Header> {
            hash,
            ..Default::default()
        };
        let info = get_block_info(&provider, header)
            .await
            .expect("Block info retrieval failed");
        assert_eq!(info.block.hash, hash);
        assert_eq!(info.logs.len(), 816);
        assert_eq!(info.transactions.len(), 311);
    }
}
