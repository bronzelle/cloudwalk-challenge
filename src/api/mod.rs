pub mod handlers;
pub mod models;

use crate::db::Database;
use axum::{routing::get, Router};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn run_api(db: Arc<Mutex<Database>>) {
    let app = Router::new()
        .route("/blocks/{number}", get(handlers::get_block_by_number))
        .route("/blocks/hash/{hash}", get(handlers::get_block_by_hash))
        .route(
            "/transactions/{hash}",
            get(handlers::get_transaction_by_hash),
        )
        // .route("/logs/filter", get(handlers::get_logs_filtered))
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8383")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
