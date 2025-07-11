// @generated automatically by Diesel CLI.

diesel::table! {
    balances (account, token, block_id) {
        account -> Binary,
        token -> Binary,
        balance -> Binary,
        block_id -> BigInt,
    }
}

diesel::table! {
    blocks (number) {
        number -> Nullable<BigInt>,
        hash -> Binary,
        parent_hash -> Binary,
        timestamp -> BigInt,
        gas_limit -> BigInt,
        gas_used -> BigInt,
        base_fee_per_gas -> Nullable<BigInt>,
    }
}

diesel::table! {
    log_topics (log_id, topic_index) {
        log_id -> Integer,
        topic_index -> Integer,
        topic -> Binary,
    }
}

diesel::table! {
    logs (id) {
        id -> Nullable<Integer>,
        transaction_hash -> Nullable<Binary>,
        log_index -> Nullable<BigInt>,
        address -> Binary,
        data -> Binary,
        block_number -> BigInt,
    }
}

diesel::table! {
    receipts (transaction_hash) {
        transaction_hash -> Nullable<Binary>,
        gas_used -> BigInt,
    }
}

diesel::table! {
    transactions (hash) {
        hash -> Nullable<Binary>,
        block_number -> BigInt,
    }
}

diesel::joinable!(balances -> blocks (block_id));
diesel::joinable!(log_topics -> logs (log_id));
diesel::joinable!(logs -> blocks (block_number));
diesel::joinable!(logs -> transactions (transaction_hash));
diesel::joinable!(receipts -> transactions (transaction_hash));
diesel::joinable!(transactions -> blocks (block_number));

diesel::allow_tables_to_appear_in_same_query!(
    balances,
    blocks,
    log_topics,
    logs,
    receipts,
    transactions,
);
