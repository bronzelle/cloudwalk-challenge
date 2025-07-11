# Blockchain Indexer - Technical Design

## 🎯 Objective

The Blockchain Indexer aims to create a minimal Ethereum node. It provides a web interface to retrieve the stored data.

- It retrieves every generated block header since the application started.
- It retrieves complete log and transaction data for each block.
- It retrieves block receipts.
- It retrieves the balance of all users performing native transactions or transferring tokens for specific contracts.

## 🚧 Project Structure

This project is structured into modules: indexer, eth_client, db, api, and types.

This modular design allows for easier exchange of components, such as the database module. For example, if the database module, which currently uses SQLite, needs to be replaced, minimal effort is required. Each module works with its own types. For instance, to switch to PostgreSQL, you would create a new database module and implement the necessary type conversions from the shared `types` module.

```shell
.
├── .github         # Contains the CI process of this project
├── Dockerfile      # Builds the image to run this project on the server.
├── GEMINI.md       # Technical Assessment
├── README.md       # Instructions on how to build and run this project.
├── TECHNICAL.md    # This file.
├── TODO.md         # List of tasks done or planned.
├── migrations      # Database migration files. They should be run using Diesel.
└── src             #
    ├── api
    │   # Implements the API.
    │   # Started by the indexer module.
    ├── db
    │   # Handles all database access.
    │   # Uses SQLite as the database.
    ├── eth_client
    │   ├── contracts
    │   │   # Implements contract interfaces.
    │   │   # Used to interact with on-chain contract implementations.
    │   # Handles all blockchain interactions via JSON-RPC.
    ├── indexer
    │   # Links all other modules.
    │   # Starts the API server, the `eth_client`, and sends parsed
    │   # block information to the database module.
    ├── types
    │   # Contains types shared across all modules.
    └── main.rs
        # Application entry point and indexer starter.
```

## 🧠 Architectural Decisions and Stack

- **alloy**: A modern library for interacting with EVM-compatible chains.
  It provides various functions and data types for this purpose.

- **anyhow**: Chosen for its simplicity in internal error handling.

- **axum**:

- **diesel**: A Prisma-like ORM, designed for simplicity and ease of use.

- **pprof**: A profiling tool that can be integrated into the application to generate flamegraphs. It also exports raw data.

- **rayon**: Used for parallel processing of logs, receipts, and transactions to improve parsing speed.

- **thiserror**: Used to simplify error responses in the API. It is also suitable for defining specific errors for modules or crates, though this has not been fully implemented yet.

- **tracing**: Provides easy integration with tracing backends, including self-hosted options.

## ⚖️ Trade-offs

While simpler, SQLite introduces performance considerations. Profiling reports indicate that a significant amount of time is spent on database operations.

Only data from specific tokens is stored to reduce the amount of data requested from the RPC provider.

## 👤 Author

Rodrigo Bronzelle - <bronzelle@gmail.com>
