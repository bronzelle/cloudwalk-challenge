pub mod models;
pub mod schema;

use self::models::{
    DbBlock, DbTransaction, NewBalance, NewBlock, NewLog, NewLogTopic, NewReceipt, NewTransaction,
};
use crate::types::{self, BlockSummary};
use crate::types::{Block, Info, Log, Transaction};
use diesel::define_sql_function;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
#[cfg(test)]
use diesel_migrations::{EmbeddedMigrations, embed_migrations};

#[cfg(test)]
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

define_sql_function!(fn last_insert_rowid() -> BigInt);

pub struct Database {
    pub conn: SqliteConnection,
}

impl Database {
    #[tracing::instrument(skip(database_url))]
    pub fn connect(database_url: &str) -> anyhow::Result<Self> {
        let conn = SqliteConnection::establish(database_url)?;
        Ok(Self { conn })
    }

    #[tracing::instrument(skip(conn))]
    fn get_block_info(
        conn: &mut SqliteConnection,
        db_block: DbBlock,
    ) -> Result<Info, anyhow::Error> {
        let db_transactions: Vec<DbTransaction> = schema::transactions::table
            .filter(
                schema::transactions::block_number.eq(db_block
                    .number
                    .ok_or_else(|| anyhow::anyhow!("Missing block number"))?),
            )
            .select(DbTransaction::as_select())
            .load::<DbTransaction>(conn)?;

        let db_logs: Vec<models::Log> = schema::logs::table
            .filter(
                schema::logs::block_number.eq(db_block
                    .number
                    .ok_or_else(|| anyhow::anyhow!("Missing block number"))?),
            )
            .load::<models::Log>(conn)?;

        let log_ids: Vec<i32> = db_logs.iter().filter_map(|log| log.id).collect();

        let db_log_topics: Vec<models::LogTopic> = schema::log_topics::table
            .filter(schema::log_topics::log_id.eq_any(log_ids))
            .load::<models::LogTopic>(conn)?;

        let logs = db_logs
            .into_iter()
            .map(|db_log| {
                let topics: Vec<[u8; 32]> = db_log_topics
                    .iter()
                    .filter(|topic| {
                        db_log
                            .id
                            .and_then(|i| (i == topic.log_id).then_some(i))
                            .is_some()
                    })
                    .map(|topic| topic.topic.clone().try_into().unwrap())
                    .collect();
                let mut log: Result<types::Log, _> = db_log.try_into();
                if let Ok(log) = &mut log {
                    log.topics = topics;
                }
                log
            })
            .collect::<Result<Vec<Log>, _>>()?;

        let block = Block::try_from(db_block)?;

        let transactions = db_transactions
            .into_iter()
            .map(|v| v.try_into())
            .collect::<Result<Vec<Transaction>, _>>()?;

        Ok(Info {
            block,
            transactions,
            logs,
        })
    }

    #[tracing::instrument(skip(self))]
    pub fn query_block_by_number(&mut self, number: u64) -> anyhow::Result<Info> {
        let conn = &mut self.conn;
        let db_block: DbBlock = schema::blocks::table
            .filter(schema::blocks::number.eq(number as i64))
            .select(DbBlock::as_select())
            .first(conn)?;

        Self::get_block_info(conn, db_block)
    }

    #[tracing::instrument(skip(self))]
    pub fn query_block_by_hash(&mut self, hash: &[u8]) -> anyhow::Result<Info> {
        let conn = &mut self.conn;
        let db_block: DbBlock = schema::blocks::table
            .filter(schema::blocks::hash.eq(hash))
            .select(DbBlock::as_select())
            .first(conn)?;

        Self::get_block_info(conn, db_block)
    }

    #[tracing::instrument(skip(self))]
    pub fn query_transaction_by_hash(&mut self, hash: &[u8]) -> anyhow::Result<types::Transaction> {
        let conn = &mut self.conn;
        let db_tx: DbTransaction = schema::transactions::table
            .filter(schema::transactions::hash.eq(hash))
            .select(DbTransaction::as_select())
            .first(conn)?;
        db_tx.try_into()
    }

    // pub fn get_logs_filtered(
    //     &mut self,
    //     block_hash: Option<&[u8]>,
    //     transaction_hash: Option<&[u8]>,
    //     topics: Vec<&[u8]>,
    // ) -> anyhow::Result<Vec<types::Log>> {
    //     let conn = &mut self.conn;
    //     let mut query = schema::logs::table.into_boxed::<diesel::sqlite::Sqlite>();

    //     if let Some(b_hash) = block_hash {
    //         let db_block: DbBlock = schema::blocks::table
    //             .filter(schema::blocks::hash.eq(b_hash))
    //             .select(DbBlock::as_select())
    //             .first(conn)?;
    //         query = query
    //             .filter(schema::logs::block_number.eq(db_block.number.unwrap_or_default() as i64));
    //     }

    //     if let Some(tx_hash) = transaction_hash {
    //         query = query.filter(schema::logs::transaction_hash.eq(tx_hash));
    //     }

    //     let db_logs: Vec<models::Log> = query.load::<models::Log>(conn)?;

    //     let log_ids: Vec<i32> = db_logs.iter().filter_map(|log| log.id).collect();

    //     let db_log_topics: Vec<models::LogTopic> = schema::log_topics::table
    //         .filter(schema::log_topics::log_id.eq_any(log_ids))
    //         .load::<models::LogTopic>(conn)?;

    //     let mut logs: Vec<types::Log> = db_logs
    //         .into_iter()
    //         .map(|db_log| {
    //             let topics: Vec<[u8; 32]> = db_log_topics
    //                 .iter()
    //                 .filter(|topic| {
    //                     db_log
    //                         .id
    //                         .and_then(|i| (i == topic.log_id).then_some(i))
    //                         .is_some()
    //                 })
    //                 .map(|topic| topic.topic.clone().try_into().unwrap())
    //                 .collect();
    //             let mut log: Result<types::Log, _> = db_log.try_into();
    //             if let Ok(log) = &mut log {
    //                 log.topics = topics;
    //             }
    //             log
    //         })
    //         .collect::<Result<Vec<types::Log>, _>>()?;

    //     if !topics.is_empty() {
    //         logs.retain(|log| {
    //             topics.iter().all(|filter_topic| {
    //                 log.topics
    //                     .iter()
    //                     .any(|log_topic| log_topic.as_slice() == *filter_topic)
    //             })
    //         });
    //     }

    //     Ok(logs)
    // }

    #[tracing::instrument(skip(self, info))]
    pub fn insert_block(&mut self, info: &BlockSummary) -> anyhow::Result<()> {
        let conn = &mut self.conn;
        conn.transaction(|conn| -> diesel::result::QueryResult<()> {
            let new_block = NewBlock::from(&info.block);
            diesel::insert_into(schema::blocks::table)
                .values(&new_block)
                .execute(conn)?;

            if !info.transactions.is_empty() {
                let new_txs: Vec<NewTransaction> = info
                    .transactions
                    .iter()
                    .map(|tx| NewTransaction {
                        hash: &tx.hash,
                        block_number: info.block.number as i64,
                    })
                    .collect();

                diesel::insert_into(schema::transactions::table)
                    .values(&new_txs)
                    .execute(conn)?;
            }

            if !info.logs.is_empty() {
                let mut new_log_topics: Vec<NewLogTopic> = Vec::new();

                for log in &info.logs {
                    let new_log = NewLog {
                        transaction_hash: log.transaction_hash.as_ref().map(|h| h.as_slice()),
                        log_index: log.log_index.map(|i| i as i64),
                        address: &log.address,
                        data: &log.data,
                        block_number: info.block.number as i64,
                    };

                    diesel::insert_into(schema::logs::table)
                        .values(&new_log)
                        .execute(conn)?;

                    let last_id: i64 = diesel::select(last_insert_rowid()).get_result(conn)?;

                    let topics: Vec<NewLogTopic> = log
                        .topics
                        .iter()
                        .enumerate()
                        .map(|(i, topic)| NewLogTopic {
                            log_id: last_id as i32,
                            topic_index: i as i32,
                            topic,
                        })
                        .collect();

                    new_log_topics.extend(topics);
                }

                diesel::insert_into(schema::log_topics::table)
                    .values(&new_log_topics)
                    .execute(conn)?;
            }

            if !info.balances.is_empty() {
                let new_balances: Vec<NewBalance> = info
                    .balances
                    .iter()
                    .map(NewBalance::from)
                    .collect();

                diesel::insert_into(schema::balances::table)
                    .values(&new_balances)
                    .execute(conn)?;
            }

            if !info.receipts.is_empty() {
                let new_receipts: Vec<NewReceipt> = info
                    .receipts
                    .iter()
                    .map(NewReceipt::from)
                    .collect();

                diesel::insert_into(schema::receipts::table)
                    .values(&new_receipts)
                    .execute(conn)?;
            }

            Ok(())
        })?;

        Ok(())
    }

    #[cfg(test)]
    fn get_conn(&mut self) -> &mut SqliteConnection {
        &mut self.conn
    }

    #[cfg(test)]
    pub fn connect_test() -> Self {
        use diesel_migrations::MigrationHarness;

        let database_url = ":memory:";
        let mut db = Database::connect(database_url).expect("Failed to connect to database");
        db.get_conn()
            .run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
        db
    }

    #[cfg(test)]
    pub fn data_setup() -> BlockSummary {
        use crate::types::{Balance, Receipt};

        let block = Block {
            number: 1,
            hash: [1; 32],
            parent_hash: [0; 32],
            timestamp: 1234567890,
            gas_limit: 1000000,
            gas_used: 500000,
            base_fee_per_gas: Some(100),
        };

        let tx1 = Transaction { hash: [2; 32] };
        let tx2 = Transaction { hash: [3; 32] };

        let log1 = Log {
            transaction_hash: Some([2; 32]),
            log_index: Some(0),
            address: [4; 20],
            topics: vec![[5; 32], [6; 32]],
            data: vec![7; 32],
            block_number: 1,
        };
        let log2 = Log {
            transaction_hash: Some([2; 32]),
            log_index: Some(1),
            address: [8; 20],
            topics: vec![[9; 32]],
            data: vec![10; 32],
            block_number: 1,
        };
        let log3 = Log {
            transaction_hash: Some([2; 32]),
            log_index: Some(2),
            address: [11; 20],
            topics: vec![],
            data: vec![],
            block_number: 1,
        };

        let log4 = Log {
            transaction_hash: Some([3; 32]),
            log_index: Some(0),
            address: [12; 20],
            topics: vec![[13; 32]],
            data: vec![14; 32],
            block_number: 1,
        };
        let log5 = Log {
            transaction_hash: Some([3; 32]),
            log_index: Some(1),
            address: [15; 20],
            topics: vec![],
            data: vec![],
            block_number: 1,
        };
        let log6 = Log {
            transaction_hash: Some([3; 32]),
            log_index: Some(2),
            address: [16; 20],
            topics: vec![[17; 32], [18; 32], [19; 32]],
            data: vec![20; 32],
            block_number: 1,
        };
        let log7 = Log {
            transaction_hash: Some([3; 32]),
            log_index: Some(3),
            address: [21; 20],
            topics: vec![],
            data: vec![],
            block_number: 1,
        };

        let balance1 = Balance {
            account: [1; 20],
            token: [2; 20],
            balance: [3; 32],
            block_id: 1,
        };
        let balance2 = Balance {
            account: [4; 20],
            token: [5; 20],
            balance: [6; 32],
            block_id: 1,
        };

        let receipt1 = Receipt {
            transaction_hash: [2; 32],
            gas_used: 21000,
        };
        let receipt2 = Receipt {
            transaction_hash: [3; 32],
            gas_used: 30000,
        };

        let mut block_summary = BlockSummary {
            block: block.clone(),
            transactions: vec![tx1.clone(), tx2.clone()],
            logs: vec![
                log1.clone(),
                log2.clone(),
                log3.clone(),
                log4.clone(),
                log5.clone(),
                log6.clone(),
                log7.clone(),
            ],
            balances: vec![balance1.clone(), balance2.clone()],
            receipts: vec![receipt1.clone(), receipt2.clone()],
        };
        block_summary
            .logs
            .sort_by_key(|log| (log.transaction_hash, log.log_index));

        block_summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_query_block() {
        let mut db = Database::connect_test();

        let info = Database::data_setup();
        db.insert_block(&info).expect("Insertion failed.");
        let mut queried_info = db
            .query_block_by_hash(&info.block.hash)
            .expect("Query failed.");
        queried_info
            .logs
            .sort_by_key(|log| (log.transaction_hash, log.log_index));

        assert_eq!(info.block, queried_info.block);
        assert_eq!(info.transactions, queried_info.transactions);
        assert_eq!(info.logs, queried_info.logs);
    }
}
