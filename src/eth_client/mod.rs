use alloy::primitives::B256;
use alloy_provider::{DynProvider, Provider, ProviderBuilder, WsConnect};
use alloy_rpc_types_eth::{BlockTransactions, Filter, Header, Log};
use futures_util::StreamExt;
use tokio::sync::mpsc::{self, Receiver};

pub struct Info {
    pub header: Header,
    pub logs: anyhow::Result<Vec<Log>>,
    pub transactions: anyhow::Result<Vec<B256>>,
}

pub async fn connect(rpc: impl Into<String>) -> anyhow::Result<Receiver<Info>> {
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

async fn get_block_info(provider: &DynProvider, header: Header) -> Info {
    let filter = Filter::new().at_block_hash(header.hash);
    let (block_result, logs_result) = tokio::join!(
        provider.get_block_by_hash(header.hash).hashes(),
        provider.get_logs(&filter)
    );

    if let Err(e) = &block_result {
        eprintln!("Failed to get block by hash {}: {e}", header.hash);
    }
    if let Err(e) = &logs_result {
        eprintln!("Failed to get logs for block {}: {e}", header.hash);
    }

    let block = block_result
        .map_err(anyhow::Error::from)
        .and_then(|block| block.ok_or_else(|| anyhow::anyhow!("Block not found")));

    let logs = logs_result.map_err(anyhow::Error::from);

    let transactions = block.and_then(|block| match block.transactions {
        BlockTransactions::Hashes(transactions) => Ok(transactions),
        _ => Err(anyhow::anyhow!("Block transactions are not full")),
    });

    Info {
        header,
        logs,
        transactions,
    }
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
        let info = get_block_info(&provider, header).await;
        assert_eq!(info.header.hash, hash);
        assert!(info.logs.is_ok());
        assert!(info.transactions.is_ok());
    }
}
