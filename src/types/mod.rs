use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockSummary {
    pub block: Block,
    pub transactions: Vec<Transaction>,
    pub logs: Vec<Log>,
    pub balances: Vec<Balance>,
    pub receipts: Vec<Receipt>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Info {
    pub block: Block,
    pub transactions: Vec<Transaction>,
    pub logs: Vec<Log>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Balance {
    pub account: [u8; 20],
    pub token: [u8; 20],
    pub balance: [u8; 32],
    pub block_id: u64,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Receipt {
    pub transaction_hash: [u8; 32],
    pub gas_used: u64,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub number: u64,
    pub hash: [u8; 32],
    pub parent_hash: [u8; 32],
    pub timestamp: u64,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub base_fee_per_gas: Option<u64>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: [u8; 32],
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Log {
    pub block_number: u64,
    pub transaction_hash: Option<[u8; 32]>,
    pub log_index: Option<u64>,
    pub address: [u8; 20],
    pub topics: Vec<[u8; 32]>,
    pub data: Vec<u8>,
}

impl From<alloy_rpc_types_eth::Header> for Block {
    fn from(header: alloy_rpc_types_eth::Header) -> Self {
        Block {
            number: header.number,
            hash: header.hash.into(),
            parent_hash: header.parent_hash.into(),
            timestamp: header.timestamp,
            gas_limit: header.gas_limit,
            gas_used: header.gas_used,
            base_fee_per_gas: header.base_fee_per_gas,
        }
    }
}

impl From<alloy::primitives::B256> for Transaction {
    fn from(hash: alloy::primitives::B256) -> Self {
        Transaction { hash: hash.into() }
    }
}

impl From<alloy_rpc_types_eth::Log> for Log {
    fn from(log: alloy_rpc_types_eth::Log) -> Self {
        let topics = log
            .topics()
            .iter()
            .map(|t| t.to_owned().into())
            .collect();
        Log {
            transaction_hash: log.transaction_hash.map(|h| h.into()),
            log_index: log.log_index,
            address: log.inner.address.into(),
            topics,
            data: log.inner.data.data.clone().into(),
            block_number: log.block_number.unwrap_or_default(),
        }
    }
}
