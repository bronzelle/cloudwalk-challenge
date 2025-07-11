use crate::db::schema::{balances, blocks, log_topics, logs, receipts, transactions};
use crate::types;

use diesel::prelude::*;

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = blocks)]
pub struct NewBlock<'a> {
    pub number: i64,
    pub hash: &'a [u8],
    pub parent_hash: &'a [u8],
    pub timestamp: i64,
    pub gas_limit: i64,
    pub gas_used: i64,
    pub base_fee_per_gas: Option<i64>,
}

impl<'a> From<&'a types::Block> for NewBlock<'a> {
    fn from(block: &'a types::Block) -> Self {
        NewBlock {
            number: block.number as i64,
            hash: &block.hash,
            parent_hash: &block.parent_hash,
            timestamp: block.timestamp as i64,
            gas_limit: block.gas_limit as i64,
            gas_used: block.gas_used as i64,
            base_fee_per_gas: block.base_fee_per_gas.map(|val| val as i64),
        }
    }
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = transactions)]
pub struct NewTransaction<'a> {
    pub hash: &'a [u8],
    pub block_number: i64,
}

#[derive(Queryable, AsChangeset, Selectable)]
#[diesel(table_name = logs)]
pub struct Log {
    pub id: Option<i32>,
    pub transaction_hash: Option<Vec<u8>>,
    pub log_index: Option<i64>,
    pub address: Vec<u8>,
    pub data: Vec<u8>,
    pub block_number: i64,
}

impl TryFrom<Log> for types::Log {
    type Error = anyhow::Error;

    fn try_from(log: Log) -> Result<Self, Self::Error> {
        Ok(types::Log {
            transaction_hash: log
                .transaction_hash
                .and_then(|hash| hash.try_into().ok()),
            log_index: log.log_index.map(|index| index as u64),
            address: log
                .address
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid address"))?,
            topics: Default::default(),
            data: log.data,
            block_number: log.block_number as u64,
        })
    }
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = logs)]
pub struct NewLog<'a> {
    pub transaction_hash: Option<&'a [u8]>,
    pub log_index: Option<i64>,
    pub address: &'a [u8],
    pub data: &'a [u8],
    pub block_number: i64,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = log_topics)]
pub struct NewLogTopic<'a> {
    pub log_id: i32,
    pub topic_index: i32,
    pub topic: &'a [u8],
}

#[derive(Queryable)]
#[diesel(table_name = log_topics)]
pub struct LogTopic {
    pub log_id: i32,
    pub topic_index: i32,
    pub topic: Vec<u8>,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = blocks)]
pub struct DbBlock {
    pub number: Option<i64>,
    pub hash: Vec<u8>,
    pub parent_hash: Vec<u8>,
    pub timestamp: i64,
    pub gas_limit: i64,
    pub gas_used: i64,
    pub base_fee_per_gas: Option<i64>,
}

impl TryFrom<DbBlock> for types::Block {
    type Error = anyhow::Error;

    fn try_from(block: DbBlock) -> Result<Self, Self::Error> {
        Ok(types::Block {
            number: block.number.unwrap() as u64,
            hash: block.hash.try_into().unwrap(),
            parent_hash: block
                .parent_hash
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid hash"))?,
            timestamp: block.timestamp as u64,
            gas_limit: block.gas_limit as u64,
            gas_used: block.gas_used as u64,
            base_fee_per_gas: block.base_fee_per_gas.map(|val| val as u64),
        })
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = transactions)]
pub struct DbTransaction {
    pub hash: Option<Vec<u8>>,
    pub block_number: i64,
}

impl TryFrom<DbTransaction> for types::Transaction {
    type Error = anyhow::Error;

    fn try_from(tx: DbTransaction) -> Result<Self, Self::Error> {
        Ok(types::Transaction {
            hash: tx
                .hash
                .ok_or_else(|| anyhow::anyhow!("Missing hash"))?
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid hash"))?,
        })
    }
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = balances)]
pub struct NewBalance<'a> {
    pub account: &'a [u8],
    pub token: &'a [u8],
    pub balance: &'a [u8],
    pub block_id: i64,
}

impl<'a> From<&'a types::Balance> for NewBalance<'a> {
    fn from(balance: &'a types::Balance) -> Self {
        NewBalance {
            account: &balance.account,
            token: &balance.token,
            balance: &balance.balance,
            block_id: balance.block_id as i64,
        }
    }
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = receipts)]
pub struct NewReceipt<'a> {
    pub transaction_hash: &'a [u8],
    pub gas_used: i64,
}

impl<'a> From<&'a types::Receipt> for NewReceipt<'a> {
    fn from(receipt: &'a types::Receipt) -> Self {
        NewReceipt {
            transaction_hash: &receipt.transaction_hash,
            gas_used: receipt.gas_used as i64,
        }
    }
}
