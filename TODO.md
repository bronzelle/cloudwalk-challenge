# Project: Blockchain Indexer

This file tracks the progress of the blockchain indexer project.

## Core Features

- [x] Connect to an Ethereum node via WebSocket.
- [x] Subscribe to new block headers.
- [x] For each new block, fetch the full block data.
- [x] For each new block, fetch all associated logs.
- [x] Extract transactions from each block.
- [x] Extract receipts from each transaction.
- [x] Store block information in a database.
- [x] Store transaction information in a database.
- [x] Store log information in a database.

## Database

- [x] Design the database schema (tables for blocks, transactions, logs).
- [x] Choose and integrate Diesel using SQLite as the database.
- [x] Implement functions to insert data into the database.

## API (Bonus)

- [x] Retrieve `accounts` information (native asset balance).
- [x] Create an `accounts` table to track native asset (Ether) balances.
- [x] Retrieve `token_transfers`.
- [x] Create a `token_transfers` table for ERC-20/721 events.
- [x] Retrieve `token_balances` for ERC-20/721 holdings.
- [x] Create a `token_balances` table for ERC-20/721 holdings.
- [ ] Design and implement a simple HTTP API to query the indexed data.
  - [x] Add endpoints to get a block by number or hash.
  - [x] Add endpoints to get a transaction by hash.
  - [ ] Add endpoints to get logs with filtering options.
  - [ ] Adapt the endpoint to follow the Ethereum JSON-RPC API standard.

## Testing

- [x] Add unit tests for individual functions.
  - [x] `eth_client`: test retrieving information from a block.
  - [x] `db`: test inserting data into the database and querying it.
  - [x] `api`: test the API endpoints.
- [ ] Add integration tests for the end-to-end flow.

## Code Quality & Refinements

- [x] Use `tokio` for asynchronous operations.
- [x] Use `anyhow` for error handling.
- [x] Implement CI process.
- [x] Add comprehensive logging throughout the application using the `tracing` crate.
- [ ] Implement more robust error handling and retries for network operations.
- [ ] Structure the project into logical modules.

## Performance & Instrumentation

- [x] Add instrumentation to key operations using `tracing`.
- [ ] Implement a tracing backend (e.g., Jaeger, OpenTelemetry) to visualize traces.
- [x] Investigate and set up profiling tools (e.g., `pprof`, `flamegraph`) to identify performance bottlenecks.

## Tasks out-of-scope

- Getting internal transactions for the native token.
