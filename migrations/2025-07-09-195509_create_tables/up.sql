-- Your SQL goes here
-- Add migration script here
CREATE TABLE IF NOT EXISTS blocks (
    number BIGINT PRIMARY KEY,
    hash BLOB UNIQUE NOT NULL,
    parent_hash BLOB NOT NULL,
    timestamp BIGINT NOT NULL,
    gas_limit BIGINT NOT NULL,
    gas_used BIGINT NOT NULL,
    base_fee_per_gas BIGINT
);

CREATE TABLE IF NOT EXISTS transactions (
    hash BLOB PRIMARY KEY,
    block_number BIGINT NOT NULL,
    FOREIGN KEY (block_number) REFERENCES blocks (number) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transaction_hash BLOB,
    log_index BIGINT,
    address BLOB NOT NULL,
    data BLOB NOT NULL,
    block_number BIGINT NOT NULL,
    FOREIGN KEY (transaction_hash) REFERENCES transactions (hash) ON DELETE SET NULL,
    FOREIGN KEY (block_number) REFERENCES blocks (number) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS log_topics (
    log_id INTEGER NOT NULL,
    topic_index INTEGER NOT NULL,
    topic BLOB NOT NULL,
    PRIMARY KEY (log_id, topic_index),
    FOREIGN KEY (log_id) REFERENCES logs (id) ON DELETE CASCADE
);

