CREATE TABLE IF NOT EXISTS balances (
    account BLOB NOT NULL,
    token BLOB NOT NULL,
    balance BLOB NOT NULL,
    block_id BIGINT NOT NULL,
    PRIMARY KEY (account, token, block_id),
    FOREIGN KEY (block_id) REFERENCES blocks (number) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS receipts (
    transaction_hash BLOB PRIMARY KEY,
    gas_used BIGINT NOT NULL,
    FOREIGN KEY (transaction_hash) REFERENCES transactions (hash) ON DELETE CASCADE
);