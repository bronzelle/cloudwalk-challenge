use alloy_provider::{Provider, ProviderBuilder, WsConnect};
use alloy_rpc_types_eth::Header;
use futures_util::StreamExt;
use tokio::sync::mpsc;

pub async fn connect(rpc: impl Into<String>, sender: mpsc::Sender<Header>) -> anyhow::Result<()> {
    let provider = ProviderBuilder::new()
        .connect_ws(WsConnect::new(rpc.into()))
        .await?
        .erased();

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
            if sender.send(header).await.is_err() {
                eprintln!("Receiver dropped. Stopping block listener.");
                break;
            }
        }
    });
    Ok(())
}
