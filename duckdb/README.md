# DuckDB

DuckDB is a relational (table-oriented) database management system (DBMS) that supports
the Structured Query Language (SQL). SQLite is the world's most widely deployed DBMS,
DuckDB adopts the ideas of SQLite regarding simplicity and embedded operation.

DuckDB has **no external dependencies**, neither for compilation nor during run-time. For
releases, the entire source tree of DuckDB is compiled into two files, a header and an
implementation file. For building, all that is required to build DuckDB is a working C++11
compiler.

For DuckDB, there is no DBMS server software to install, update and maintain. DuckDB does
not run as a separate process, but completely embedded within a host process. For the
analytical use cases that DuckDB targets, this has the additional advantage of
**high-speed data transfer** to and from the database.

DuckDB provides **transactional guarantees** (ACID properties) through their custom, bulk
optimized **Multi-Version Concurrency Control (MVCC)**. Data can be stored in persistent,
**single-file databases**. DuckDB supports secondary indexes to speed up queries trying to
find a single table entry.

**DuckDB is deeply integrated into Python and R for efficient interactive data analysis.**

DuckDB is designed to support **analytical query workloads**, online analytical processing
(OLAP). To efficiently support this workload, it is critical to reduce the amount of CPU
cycles that are expended per individual value. The state of the art in data management to
achieve this are either **vectorized or just-in-time query execution engines**. DuckDB
uses a **columnar-vectorized query execution engine**. This greatly reduces overhead
present in traditional systems such as PostgreSQL, MySQL, or SQLite which process each row
sequentially. Vectorized query execution leads to far better performance in OLAP queries.


## DuckDB vs PostgreSQL

### Designed for Analytics vs Transactional Workloads

- DuckDB is optimized for analytical workloads—think OLAP-style queries, data science, and fast aggregations on local files.

- PostgreSQL is a traditional relational database system built for transactional workloads—think web applications, CRUD operations, and ACID compliance for multi-user environments.

### Embedded vs Client-Server Architecture

- DuckDB runs in-process (embedded directly in applications) without a separate database server, making it lightweight and easy to integrate.

- PostgreSQL follows a client-server model, requiring a running database instance, authentication mechanisms, and network connections.

### Columnar vs Row-Based Storage

- DuckDB stores data in a columnar format, which is highly efficient for analytical queries that scan large datasets.
- PostgreSQL uses a row-based format, better suited for transactional workloads where individual records are frequently updated.

### Scalability & Distributed Capabilities

- DuckDB is not designed for distributed computing or clustering—it excels in local, single-node processing.
- PostgreSQL supports replication, sharding, and extensions like CitusDB for horizontal scaling.


## Installation

Installation CLI:

```
curl https://install.duckdb.org | sh
```

Installation Python library:

```
python3 -m pip install duckdb
```


Install `ducklake` and `postgres` extensions with:

```python
import duckdb

with duckdb.connection() as conn:
    conn.install_extension("postgres")
    conn.install_extension("ducklake")

```


## Connection options

When using DuckDB through `duckdb.sql()` it operates on an **in-memory** database, i.e.,
no tables are persisted on disk. Invoking the `duckdb.connect()` method without arguments
returns a connection, which also uses an in-memory database:

```python
import duckdb

conn = duckdb.connect()
conn.sql("SELECT 42 AS x").show()
```

## Persistent storage

The `duckdb.connect(dbname)` creates a connection to a **persistent** database. Any data
written to that connection will be persisted, and can be reloaded by reconnecting to the
same file, both from Python and from other DuckDB clients.

```python
import duckdb

# create a connection to a file called 'file.db'
con = duckdb.connect("file.db")
# create a table and load data into it
con.sql("CREATE TABLE test (i INTEGER)")
con.sql("INSERT INTO test VALUES (42)")
# query the table
con.table("test").show()
# explicitly close the connection
con.close()
# Note: connections also closed implicitly when they go out of scope
```

```python
import duckdb

with duckdb.connect("file.db") as con:
    con.sql("CREATE TABLE test (i INTEGER)")
    con.sql("INSERT INTO test VALUES (42)")
    con.table("test").show()
    # the context manager closes the connection automatically
```

### Configuration

```python
import duckdb

con = duckdb.connect(config = {'threads': 1})
```


## Usage Python


```python
import duckdb

duckdb.sql("SELECT 42").show()

┌───────┐
│  42   │
│ int32 │
├───────┤
│    42 │
└───────┘
```


### Data ingress

```python
import duckdb

duckdb.read_csv("example.csv")                # read a CSV file into a Relation
duckdb.read_parquet("example.parquet")        # read a Parquet file into a Relation
duckdb.read_json("example.json")              # read a JSON file into a Relation

duckdb.sql("SELECT * FROM 'example.csv'")     # directly query a CSV file
duckdb.sql("SELECT * FROM 'example.parquet'") # directly query a Parquet file
duckdb.sql("SELECT * FROM 'example.json'")    # directly query a JSON file
```


### Pandas query

```python
import duckdb
import pandas as pd

pandas_df = pd.DataFrame({"a": [42]})
duckdb.sql("SELECT * FROM pandas_df")
```


### Polars query


```python
import duckdb
import polars as pl

polars_df = pl.DataFrame({"a": [42]})
duckdb.sql("SELECT * FROM polars_df")

┌───────┐
│   a   │
│ int64 │
├───────┤
│    42 │
└───────┘
```


### Result conversion

```python
import duckdb

duckdb.sql("SELECT 42").fetchall()   # Python objects
duckdb.sql("SELECT 42").df()         # Pandas DataFrame
duckdb.sql("SELECT 42").pl()         # Polars DataFrame
duckdb.sql("SELECT 42").arrow()      # Arrow Table
duckdb.sql("SELECT 42").fetchnumpy() # NumPy Arrays
```


### Writing data to disk

```python
import duckdb

duckdb.sql("SELECT 42").write_parquet("out.parquet") # Write to a Parquet file
duckdb.sql("SELECT 42").write_csv("out.csv")         # Write to a CSV file
duckdb.sql("COPY (SELECT 42) TO 'out.parquet'")      # Copy to a Parquet file
```


## Loading and Installing Extensions

```python
import duckdb

con = duckdb.connect()
con.install_extension("spatial")
con.load_extension("spatial")
```


## DuckLake

DuckLake is an integrated data lake and catalog format. DuckLake delivers advanced
data lake features without traditional lakehouse complexity by using Parquet files
and your SQL database. It's an open, standalone format from the DuckDB team.


### Iceberg catalog architecture

![Iceberg catalog architecture](https://ducklake.select/images/manifesto/dark/iceberg-catalog-architecture.png)

Once a database has entered the Lakehouse stack anyway, it makes an insane amount
of sense to also use it for managing the rest of the table metadata! We can still
take advantage of the “endless” capacity and “infinite” scalability of blob stores
for storing the actual table data in open formats like Parquet, but we can much more
efficiently and effectively manage the metadata needed to support changes in a database!
Coincidentally, this is also what Google BigQuery (with Spanner) and Snowflake
(with FoundationDB) have chosen, just without the open formats at the bottom.

![DuckLake architecture](https://ducklake.select/images/manifesto/dark/ducklake-architecture.png)

To resolve the fundamental problems of the existing Lakehouse architecture, we have created
a new open table format called DuckLake. DuckLake re-imagines what a “Lakehouse” format should look like by acknowledging two simple truths:

- Storing data files in open formats on blob storage is a great idea for scalability and to prevent lock-in.
- Managing metadata is a complex and interconnected data management task best left to a database management system.

The basic design of DuckLake is to move all metadata structures into a SQL database, both for catalog and table data. The format is defined as a set of relational tables and pure-SQL transactions on them that describe data operations like schema creation, modification, and addition, deletion and updating of data.

DuckLake follows the DuckDB design principles of keeping things simple and incremental. In order to run DuckLake on a laptop, it is enough to just install DuckDB with the ducklake extension. This is great for testing purposes, development and prototyping. In this case, the catalog store is just a local DuckDB file.

DuckLake data files are immutable, it never requires modifying files in place or re-using file names. This allows use with almost any storage system. DuckLake supports integration with any storage system like local disk, local NAS, S3, Azure Blob Store, GCS, etc. The storage prefix for data files (e.g., s3://mybucket/mylake/) is specified when the metadata tables are created.

There are no Avro or JSON files. There is no additional catalog server or additional API to integrate with. It’s all just SQL. We all know SQL.

DuckLake actually increases separation of concerns within a data architecture into three parts. Storage, compute and metadata management. Storage remains on purpose-built file storage (e.g., blob storage), DuckLake can scale infinitely in storage.

An arbitrary number of compute nodes are querying and updating the catalog database and then independently reading and writing from storage. DuckLake can scale infinitely regarding compute.

Just like DuckDB itself, DuckLake is very much about speed. One of the biggest pain points of Iceberg and Delta Lake is the involved sequence of file IO that is required to run the smallest query. Following the catalog and file metadata path requires many separate sequential HTTP requests. As a result, there is a lower bound to how fast reads or transactions can run. There is a lot of time spent in the critical path of transaction commits, leading to frequent conflicts and expensive conflict resolution. While caching can be used to alleviate some of these problems, this adds additional complexity and is only effective for “hot” data.

The unified metadata within a SQL database also allows for low-latency query planning. In order to read from a DuckLake table, a single query is sent to the catalog database, which performs the schema-based, partition-based and statistics-based pruning to essentially retrieve a list of files to be read from blob storage. There are no multiple round trips to storage to retrieve and reconstruct metadata state. There is also less that can go wrong, no S3 throttling, no failing requests, no retries, no not-yet consistent views on storage that lead to files being invisible, etc.

DuckLake is also able to improve the two biggest performance problems of data lakes: small changes and many concurrent changes.

For small changes, DuckLake will dramatically reduce the number of small files written to storage. There is no new snapshot file with a tiny change compared to the previous one, there is no new manifest file or manifest list. DuckLake even optionally allows transparent inlining of small changes to tables into actual tables directly in the metadata store! Turns out, a database system can be used to manage data, too. This allows for sub-millisecond writes and for improved overall query performance by reducing the number of files that have to be read. By writing many fewer files, DuckLake also greatly simplifies cleanup and compaction operations.


## Configuration

To use DuckLake, you need to make two decisions: which metadata catalgo database you want
to use and where you want to store those files. In the simplest case, you use a local
DuckDB file for the metadata catalog and a local folder on your computer for file storage.

- If you would like to perform local data warehousing with a single client, use DuckDB as the catalog database.
- If you would like to perform local data warehousing using multiple local clients, use SQLite as the catalog database.
- If you would like to operate a multi-user lakehouse with potentially remote clients, choose a transactional client-server database system as the catalog database: MySQL or PostgreSQL.

**Note that if you are using DuckDB as your catalog database, you're limited to a single client.**

The DuckLake format does not support indexes, primary keys, foreign keys, and `UNIQUE` and
`CHECK` constraints.
