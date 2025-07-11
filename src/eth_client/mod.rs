mod contracts;
mod parser_log;
mod parser_receipt;
mod types;
pub mod update_balances;

use std::sync::Arc;

use alloy_provider::{DynProvider, Provider, ProviderBuilder, WsConnect};
use alloy_rpc_types_eth::{BlockId, BlockNumberOrTag, BlockTransactions, Filter, Header};
use futures_util::StreamExt;
use tokio::sync::mpsc::{self, Receiver};
use tracing::Instrument;

use crate::{eth_client::update_balances::get_balances, types::BlockSummary};

#[tracing::instrument(skip(rpc))]
pub async fn connect(
    rpc: impl Into<String>,
) -> anyhow::Result<Receiver<anyhow::Result<BlockSummary>>> {
    let provider = ProviderBuilder::new()
        .connect_ws(WsConnect::new(rpc))
        .await?
        .erased();

    let (sender, receiver) = mpsc::channel(100);

    let provider = Arc::new(provider);

    tokio::spawn(
        async move {
            let sub = match provider.subscribe_blocks().await {
                Ok(sub) => sub,
                Err(e) => {
                    tracing::error!("Failed to subscribe to blocks: {}", e);
                    return;
                }
            };
            let mut stream = sub.into_stream();
            while let Some(header) = stream.next().await {
                process_block(Arc::clone(&provider), header, sender.clone());
            }
        }
        .instrument(tracing::info_span!("block_subscription_listener")),
    );
    Ok(receiver)
}

#[tracing::instrument(skip(provider, sender))]
fn process_block(
    provider: Arc<DynProvider>,
    header: Header,
    sender: mpsc::Sender<anyhow::Result<BlockSummary>>,
) {
    tokio::spawn(async move {
        let info = get_block_info(provider, header).await;
        if sender.send(info).await.is_err() {
            eprintln!("Receiver dropped. Stopping block listener.");
            return;
        }
    });
}

#[tracing::instrument(skip(provider))]
async fn get_block_info(
    provider: Arc<DynProvider>,
    header: Header,
) -> anyhow::Result<BlockSummary> {
    let filter = Filter::new().at_block_hash(header.hash);
    let block_id = BlockId::Number(BlockNumberOrTag::Number(header.number));
    let (block_result, logs_result, receipts_result) = tokio::join!(
        provider.get_block_by_hash(header.hash).full(),
        provider.get_logs(&filter),
        provider.get_block_receipts(block_id),
    );

    let (block, logs, receipts) = (
        block_result?.ok_or_else(|| anyhow::anyhow!("Block not found"))?,
        logs_result?,
        receipts_result?,
    );

    let transactions = match block.transactions {
        BlockTransactions::Full(transactions) => Ok(transactions),
        _ => Err(anyhow::anyhow!("Block transactions are not full")),
    }?;

    let receipts = receipts.ok_or_else(|| anyhow::anyhow!("Receipts not found"))?;

    let (mut logs_accounts, (transactions_accounts, receipts)) = tokio::join!(
        parser_log::parse_logs(&logs),
        parser_receipt::parse_receipts(&receipts, &transactions),
    );

    for (account, contracts) in transactions_accounts.interactions {
        logs_accounts
            .interactions
            .entry(account)
            .or_default()
            .extend(contracts);
    }

    let balances = get_balances(provider, logs_accounts).await;

    Ok(BlockSummary {
        block: header.into(),
        logs: logs.iter().map(|log| log.clone().into()).collect(),
        transactions: transactions
            .iter()
            .map(|tx| {
                // tx.
                tx.inner.hash().clone().into()
            })
            .collect(),
        balances: balances.into_iter().map(|b| b.into()).collect(),
        receipts,
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
