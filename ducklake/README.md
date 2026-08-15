## DuckLake

an integrated data lake and catalog format.

DuckLake delivers advanced data lake features without traditional lakehouse
complexity by using Parquet files and your SQL database. It's an open,
standalone format from the DuckDB team ([link](https://ducklake.select/)).

DuckLake uses a database system to manage your metadata for the catalog.
All you need to run your own data warehouse is a database system and
storage for Parquet files.


### Features

- **Data lake operations:** DuckLake supports snapshots, time travel queries, schema evolution and partitioning.
- **Lightweight snapshots:** You can have as many snapshots as you want without frequent compacting steps!
- **ACID transactions:** DuckLake allows concurrent access with ACID transactional guarantees over multi-table operations.
- **Performance-oriented:** DuckLake uses statistics for filter pushdown, enabling fast queries even on large datasets.


#### Catalog database

DuckLake can use any SQL system as its catalog database, provided that it
supports ACID transactions and primary key constraints.


#### Clients

Users can run multiple DuckLake clients and connect concurrently to the catalog
database – PostgreSQL, MySQL or SQLite – to work over the same DuckLake dataset.


#### Storage

DuckLake can store your data on any object storage such as AWS S3.


### Your first DuckLake with DuckDB

```sql
INSTALL ducklake;
INSTALL sqlite;

ATTACH 'ducklake:sqlite:metadata.sqlite' AS my_ducklake
    (DATA_PATH 'data_files/');
USE my_ducklake;
```


### DuckDB Installation

```bash
curl https://install.duckdb.org | sh
```

