# Blockchain Indexer

## Requirements

You need to have Diesel-cli installed.

## Test

For tests, use a local database.
Set DATABASE_URL enviroment variable with the address of the SQLite database then run:

```shell
diesel setup
diesel migrations run
```

https://inloop.github.io/sqlite-viewer/
