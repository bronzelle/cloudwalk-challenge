# Project: Blockchain Indexer

This file tracks the progress of the blockchain indexer project.

## Core Features

- [x] Connect to an Ethereum node via WebSocket.
- [x] Subscribe to new block headers.
- [x] For each new block, fetch the full block data.
- [x] For each new block, fetch all associated logs.
- [x] Extract transactions from each block.
- [ ] Extract receipts from each transaction.
- [ ] Store block information in a database.
- [ ] Store transaction information in a database.
- [ ] Store log information in a database.

## Database

- [ ] Design the database schema (tables for blocks, transactions, logs).
- [ ] Choose and integrate Diesel using SQLite as the database.
- [ ] Implement functions to insert data into the database.

## API (Bonus)

- [ ] Retrieve `accounts` information (native asset balance).
- [ ] Create an `accounts` table to track native asset (Ether) balances.
- [ ] Retrieve `token_transfers`.
- [ ] Create a `token_transfers` table for ERC-20/721 events.
- [ ] Retrieve `token_balances` for ERC-20/721 holdings.
- [ ] Create a `token_balances` table for ERC-20/721 holdings.
- [ ] Design and implement a simple HTTP API to query the indexed data.
- [ ] Add endpoints to get a block by number or hash.
- [ ] Add endpoints to get a transaction by hash.
- [ ] Add endpoints to get logs with filtering options.

## Testing

- Add unit tests for individual functions.
  - [x] `eth_client`: test retrieving information from a block
- [ ] Add integration tests for the end-to-end flow.

## Code Quality & Refinements

- [x] Use `tokio` for asynchronous operations.
- [x] Use `anyhow` for error handling.
- [ ] Implement CI process.
- [ ] Implement more robust error handling and retries for network operations.
- [ ] Add comprehensive logging throughout the application.
- [ ] Structure the project into logical modules.

## Performance & Instrumentation

- [ ] Add instrumentation to measure performance of key operations.
- [ ] Set up benchmarks for performance testing.
