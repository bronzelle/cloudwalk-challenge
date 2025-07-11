pub mod handlers;
pub mod models;

use crate::db::Database;
use axum::{Router, routing::get};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tracing::instrument(skip(db))]
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use axum::http::StatusCode;
    use tokio::sync::OnceCell;

    static ONCE: OnceCell<Arc<Mutex<Database>>> = OnceCell::const_new();

    // This helper function will spawn the server in the background, only once.
    async fn setup_app() -> Arc<Mutex<Database>> {
        let database = ONCE
            .get_or_init(|| async {
                let database = Arc::new(Mutex::new(Database::connect_test()));
                let db = Arc::clone(&database);
                tokio::spawn(async {
                    run_api(db).await;
                });
                // Give the server a moment to start up.
                tokio::time::sleep(Duration::from_millis(100)).await;
                database
            })
            .await;
        
        database.clone()
    }

    #[tokio::test]
    async fn test_get_block_by_number() {
        setup_app().await;

        let response = reqwest::get("http://127.0.0.1:8383/blocks/1")
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_block_by_hash_not_found() {
        setup_app().await;

        let response = reqwest::get("http://127.0.0.1:8383/blocks/hash/1234567890123456789012345678901234567890123456789012345678901234")
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_transaction_by_wrong_hash() {
        setup_app().await;

        let response = reqwest::get("http://127.0.0.1:8383/transactions/0x1234567890123456789012345678901234567890123456789012345678901234")
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_get_transaction_by_hash() {
        let db = setup_app().await;

        let info = Database::data_setup();
        db.lock()
            .await
            .insert_block(&info)
            .expect("Insertion failed.");

        let response = reqwest::get("http://127.0.0.1:8383/transactions/0202020202020202020202020202020202020202020202020202020202020202")
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
