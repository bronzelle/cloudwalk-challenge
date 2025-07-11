use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{api::models::InternalErrors, db::Database};
use crate::{
    api::models::{ApiResponse, Transaction},
    types::Info,
};

#[tracing::instrument(skip(db))]
pub async fn get_block_by_number(
    Path(number): Path<u64>,
    State(db): State<Arc<Mutex<Database>>>,
) -> ApiResponse<Info> {
    let mut db = db.lock().await;
    match db.query_block_by_number(number) {
        Ok(block) => Ok(Json(block)),
        Err(_) => Err(InternalErrors::BlockNotFound(number.to_string())),
    }
}

#[tracing::instrument(skip(db))]
pub async fn get_block_by_hash(
    Path(hash): Path<String>,
    State(db): State<Arc<Mutex<Database>>>,
) -> ApiResponse<Info> {
    let Ok(hash_parsed) = hex::decode(hash.clone()) else {
        return Err(InternalErrors::InvalidHash(hash));
    };
    let mut db = db.lock().await;
    match db.query_block_by_hash(hash_parsed.as_slice()) {
        Ok(block) => Ok(Json(block)),
        Err(_) => Err(InternalErrors::BlockNotFound(hash)),
    }
}

#[tracing::instrument(skip(db))]
pub async fn get_transaction_by_hash(
    Path(hash): Path<String>,
    State(db): State<Arc<Mutex<Database>>>,
) -> ApiResponse<Transaction> {
    let Ok(hash_parsed) = hex::decode(hash.clone()) else {
        return Err(InternalErrors::InvalidHash(hash));
    };
    let mut db = db.lock().await;
    match db.query_transaction_by_hash(hash_parsed.as_slice()) {
        Ok(transaction) => Ok(Json(transaction.into())),
        Err(_) => Err(InternalErrors::TransactionNotFound(hash)),
    }
}

// pub async fn get_logs_filtered(
//     Query(params): Query<HashMap<String, String>>,
//     State(db): State<Arc<Database>>,
// ) -> impl IntoResponse {
//     let block_hash = params.get("block_hash").map(|s| decode(s).ok()).flatten();
//     let transaction_hash = params
//         .get("transaction_hash")
//         .map(|s| decode(s).ok())
//         .flatten();
//     let topics = params.get("topics").map(|s| {
//         s.split(',')
//             .map(|t| decode(t).ok())
//             .collect::<Vec<Option<Vec<u8>>>>()
//             .into_iter()
//             .filter_map(|x| x)
//             .collect::<Vec<Vec<u8>>>()
//     });

//     let topics_refs: Vec<&[u8]> = topics
//         .as_ref()
//         .map_or(vec![], |v| v.iter().map(|x| x.as_slice()).collect());

//     let mut db = db.as_ref().clone();
//     match db.get_logs_filtered(
//         block_hash.as_deref(),
//         transaction_hash.as_deref(),
//         topics_refs,
//     ) {
//         Ok(logs) => Json(json!(logs.into_iter().map(Log::from).collect::<Vec<Log>>())),
//         Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
//     }
// }
