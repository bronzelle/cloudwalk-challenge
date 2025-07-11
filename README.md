# Blockchain Indexer

This project is a simple blockchain indexer for Ethereum-like chains. It connects to an Ethereum node, listens for new blocks, processes them, and stores the data in a local SQLite database.

## Prerequisites

- Rust toolchain (via `rustup`)
- `diesel_cli` for database migrations. Install it with:

  ```shell
  cargo install diesel_cli --no-default-features --features sqlite
  ```

## Environment Configuration

The application requires environment variables for configuration. An example is provided in `.env.example`.

Copy the example file to `.env` and update it with your configuration:

```shell
cp .env.example .env
```

## Local development

For Local development, use a local database.
Set DATABASE_URL enviroment variable with the address of the SQLite database then run:

```shell
diesel setup
```

## Profiling

Run the application with profiling feature enabled. The output reports will be under `report/`. A new one is generated every 60 seconds.

```shell
cargo run --release --features profiling
```

## Tests

For running all the tests:

```shell
cargo test
```

## Database Visualization

Use an external tool to view the database. Like [sqlite-viewer](https://inloop.github.io/sqlite-viewer/).
