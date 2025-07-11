use std::collections::{HashMap, HashSet};

use alloy::primitives::{Address, U256};
use alloy_rpc_types_eth::{Transaction, TransactionReceipt, TransactionTrait};
use rayon::prelude::*;

use crate::{eth_client::types::ParsedData, types::Receipt};

/// Parses a vector of transaction receipts in parallel to extract key information.
/// This functions gets all the Transfer events which is used for get token transactions.
/// It does a summary of the receipt to store it in the DB.
#[tracing::instrument(skip(receipts, transactions))]
pub async fn parse_receipts(
    receipts: &Vec<TransactionReceipt>,
    transactions: &Vec<Transaction>,
) -> (ParsedData, Vec<Receipt>) {
    let block_id = receipts
        .first()
        .and_then(|r| r.block_number)
        .unwrap_or_default();
    let (receipts_summary, interactions) = transactions
        .into_par_iter()
        .zip(receipts)
        .fold(
            || (Vec::new(), HashMap::<_, HashSet<_>>::new()),
            |mut acc, (tx, receipt)| {
                let (ref mut receipts_summary, ref mut interactions) = acc;

                // Just native transfer will be tracked here
                if tx.value() == U256::ZERO {
                    return acc;
                }

                if let Some(to) = tx.to() {
                    interactions.entry(to).or_default().insert(Address::ZERO);
                }
                if tx.inner.hash() == &receipt.transaction_hash {
                    interactions
                        .entry(receipt.from)
                        .or_default()
                        .insert(Address::ZERO);
                }

                receipts_summary.push(Receipt {
                    transaction_hash: (*tx.inner.hash()).into(),
                    gas_used: receipt.gas_used,
                });

                acc
            },
        )
        .reduce(
            || (Vec::new(), HashMap::new()),
            |mut total, partial| {
                let (ref mut t_receipts_summary, ref mut t_interactions) = total;
                let (p_receipts_summary, p_interactions) = partial;

                for (account, contracts) in p_interactions {
                    t_interactions.entry(account).or_default().extend(contracts);
                }
                t_receipts_summary.extend(p_receipts_summary);

                total
            },
        );

    (
        ParsedData {
            block_id,
            interactions,
        },
        receipts_summary,
    )
}
