use std::{pin::Pin, sync::Arc};

use alloy::primitives::Address;
use alloy_provider::{DynProvider, Provider};
use futures::future::join_all;

use crate::eth_client::{
    contracts::erc20::IERC20,
    types::{Balance, ParsedData},
};

pub async fn get_balances(provider: Arc<DynProvider>, interaction: ParsedData) -> Vec<Balance> {
    let mut balance_futures: Vec<Pin<Box<dyn Future<Output = Option<Balance>> + Send>>> =
        Vec::new();
    let block_id = interaction.block_id;

    for (account, contracts) in interaction.interactions {
        for token_address in contracts {
            if token_address != Address::ZERO {
                let contract = IERC20::new(token_address, Arc::clone(&provider));
                let future = async move {
                    if let Ok(balance) = contract.balanceOf(account).call().await {
                        Some(Balance {
                            account,
                            balance,
                            token: token_address,
                            block_id,
                        })
                    } else {
                        None
                    }
                };
                balance_futures.push(Box::pin(future));
            } else {
                let provider = Arc::clone(&provider);
                let future = async move {
                    if let Ok(balance) = provider.get_balance(account).await {
                        Some(Balance {
                            account,
                            balance,
                            token: Address::ZERO,
                            block_id,
                        })
                    } else {
                        None
                    }
                };
                balance_futures.push(Box::pin(future));
            }
        }
    }

    join_all(balance_futures)
        .await
        .into_iter()
        .filter_map(|b| b)
        .collect()
}
