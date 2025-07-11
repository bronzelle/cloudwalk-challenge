use std::collections::{HashMap, HashSet};

use alloy_rpc_types_eth::Log;
use alloy_sol_types::SolEvent;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::eth_client::{
    contracts::erc20::IERC20,
    types::{KNOWN_TOKENS, ParsedData},
};

#[tracing::instrument(skip(logs))]
pub async fn parse_logs(logs: &Vec<Log>) -> ParsedData {
    let block_id = logs
        .first()
        .and_then(|l| l.block_number)
        .unwrap_or_default();
    let interactions = logs
        .into_par_iter()
        .fold(
            HashMap::<_, HashSet<_>>::new,
            |mut acc, log| {
                let interactions = &mut acc;

                let Ok(event) = IERC20::Transfer::decode_log(&log.inner) else {
                    return acc;
                };

                // If the transfer is made with an unknown token, it's not tracked.
                if !KNOWN_TOKENS.contains(&event.address) {
                    return acc;
                }

                interactions
                    .entry(event.from)
                    .or_default()
                    .insert(event.address);
                interactions
                    .entry(event.to)
                    .or_default()
                    .insert(event.address);

                acc
            },
        )
        .reduce(
            HashMap::new,
            |mut total, partial| {
                let t_interactions = &mut total;
                let p_interactions = partial;

                for (account, contracts) in p_interactions {
                    t_interactions.entry(account).or_default().extend(contracts);
                }

                total
            },
        );

    ParsedData {
        block_id,
        interactions,
    }
}
