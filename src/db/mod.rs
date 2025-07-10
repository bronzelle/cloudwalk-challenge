pub mod models;
pub mod schema;

use self::models::{DbBlock, DbTransaction, NewBlock, NewLog, NewLogTopic, NewTransaction};
use crate::types;
use crate::types::{Block, Info, Log, Transaction};
use diesel::define_sql_function;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

define_sql_function!(fn last_insert_rowid() -> BigInt);

pub struct Database {
    pub conn: SqliteConnection,
}

impl Database {
    pub fn connect(database_url: &str) -> anyhow::Result<Self> {
        let conn = SqliteConnection::establish(database_url)?;
        Ok(Self { conn })
    }

    pub fn query_block_by_hash(&mut self, hash: &[u8]) -> anyhow::Result<Info> {
        let conn = &mut self.conn;
        let db_block: DbBlock = schema::blocks::table
            .filter(schema::blocks::hash.eq(hash))
            .select(DbBlock::as_select())
            .first(conn)?;

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

        let log_ids: Vec<i32> = db_logs
            .iter()
            .map(|log| log.id)
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| anyhow::anyhow!("Missing log ids"))?;

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

    pub fn insert_block(&mut self, info: &Info) -> anyhow::Result<()> {
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
                            topic: topic,
                        })
                        .collect();

                    new_log_topics.extend(topics);
                }

                diesel::insert_into(schema::log_topics::table)
                    .values(&new_log_topics)
                    .execute(conn)?;
            }

            Ok(())
        })?;

        Ok(())
    }

    fn get_conn(&mut self) -> &mut SqliteConnection {
        &mut self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Block, Log, Transaction};
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

    #[test]
    fn test_insert_and_query_block() {
        let database_url = ":memory:";
        let mut db = Database::connect(&database_url).expect("Failed to connect to database");
        db.get_conn()
            .run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");

        let info = data_setup();
        db.insert_block(&info).expect("Insertion failed.");
        let mut queried_info = db
            .query_block_by_hash(&info.block.hash)
            .expect("Query failed.");
        queried_info
            .logs
            .sort_by_key(|log| (log.transaction_hash.clone(), log.log_index));

        assert_eq!(info, queried_info);
    }

    fn data_setup() -> Info {
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
        };
        let log2 = Log {
            transaction_hash: Some([2; 32]),
            log_index: Some(1),
            address: [8; 20],
            topics: vec![[9; 32]],
            data: vec![10; 32],
        };
        let log3 = Log {
            transaction_hash: Some([2; 32]),
            log_index: Some(2),
            address: [11; 20],
            topics: vec![],
            data: vec![],
        };

        let log4 = Log {
            transaction_hash: Some([3; 32]),
            log_index: Some(0),
            address: [12; 20],
            topics: vec![[13; 32]],
            data: vec![14; 32],
        };
        let log5 = Log {
            transaction_hash: Some([3; 32]),
            log_index: Some(1),
            address: [15; 20],
            topics: vec![],
            data: vec![],
        };
        let log6 = Log {
            transaction_hash: Some([3; 32]),
            log_index: Some(2),
            address: [16; 20],
            topics: vec![[17; 32], [18; 32], [19; 32]],
            data: vec![20; 32],
        };
        let log7 = Log {
            transaction_hash: Some([3; 32]),
            log_index: Some(3),
            address: [21; 20],
            topics: vec![],
            data: vec![],
        };

        let mut info = Info {
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
        };
        info.logs
            .sort_by_key(|log| (log.transaction_hash.clone(), log.log_index));

        info
    }
}
