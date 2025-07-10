use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub(crate) type ApiResponse<T> = Result<Json<T>, InternalErrors>;

#[derive(Deserialize, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl From<InternalErrors> for ErrorResponse {
    fn from(error: InternalErrors) -> Self {
        ErrorResponse {
            error: error.to_string(),
        }
    }
}

#[derive(Debug, Error)]
pub enum InternalErrors {
    #[error("Block not found {0}")]
    BlockNotFound(String),
    #[error("Invalid hash {0}")]
    InvalidHash(String),
    #[error("Transaction not found {0}")]
    TransactionNotFound(String),
}

impl IntoResponse for InternalErrors {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            InternalErrors::BlockNotFound(_) => StatusCode::NOT_FOUND,
            InternalErrors::InvalidHash(_) => StatusCode::BAD_REQUEST,
            InternalErrors::TransactionNotFound(_) => StatusCode::NOT_FOUND,
        };
        (status_code, Json(ErrorResponse::from(self))).into_response()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub number: u64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: u64,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub base_fee_per_gas: Option<u64>,
}

impl From<crate::types::Block> for Block {
    fn from(block: crate::types::Block) -> Self {
        Block {
            number: block.number,
            hash: hex::encode(block.hash),
            parent_hash: hex::encode(block.parent_hash),
            timestamp: block.timestamp,
            gas_limit: block.gas_limit,
            gas_used: block.gas_used,
            base_fee_per_gas: block.base_fee_per_gas,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub hash: String,
}

impl From<crate::types::Transaction> for Transaction {
    fn from(tx: crate::types::Transaction) -> Self {
        Transaction {
            hash: hex::encode(tx.hash),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Log {
    pub transaction_hash: Option<String>,
    pub log_index: Option<u64>,
    pub address: String,
    pub data: String,
    pub block_number: u64,
}

impl From<crate::types::Log> for Log {
    fn from(log: crate::types::Log) -> Self {
        Log {
            transaction_hash: log.transaction_hash.map(hex::encode),
            log_index: log.log_index,
            address: hex::encode(log.address),
            data: hex::encode(log.data),
            block_number: log.block_number,
        }
    }
}
