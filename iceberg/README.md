# What is Apache Iceberg?

**Apache Iceberg** is a high performance open table format designed for large-scale analytical datasets.
Iceberg enables the use of SQL tables for big data while making it possible for, e.g.,
Spark engines to safely work with the same tables at the same time.

Iceberg addresses the performance and usability challenges of [Apache Hive](https://en.wikipedia.org/wiki/Apache_Hive)
(which is a data warehouse software project built on top of Apache Hadoop for providing
data query and analysis) in large and demanding data lake environments.

Trivia: Iceberg was startd at Netflix by Ryan Blue and Dan Weeks. Hive was never able to
guarantee correctness and did not provide stable atomic transactions. Iceberg thus:

1. Ensures the correctness of the data and support ACID transactions.
2. Improve performance by enabling finer-grained operations to be done at the file
   granularity for optimal writes.
3. Simplify and abstract general operation and maintenance of tables.

In simple terms, Apache Iceberg bridges the scalability of data lakes and the query
capabilities of data warehouses. It offers the best of both worlds, allowing you to query
large datasets in a structured format using SQL while maintaining the scalability of a
data lake.


## Apache Iceberg Core Features


### Schema Evolution

This feature allows you to update a table's schema without rewriting the entire dataset,
which is practical in dynamic environments where data requirements frequently change.
Iceberg also ensures that you can seamlessly add or rename new columns, giving you
confidence that your data is always up to date and correct.


## Hidden Partitioning

Traditional table formats require explicit partitioning columns, which can lead to
suboptimal query performance. Iceberg's hidden partitioning automatically tracks and
optimizes partitions without exposing partition columns to the end user.


## ACID Transactions

Iceberg provides full ACID compliance, ensuring your data is consistent and reliable. This
means that you can perform concurrent read and write operations without risking data
corruption.


## Time Travel

With Iceberg's time travel feature, you can query historical snapshots of your data. If a
dataset was accidentally overwritten, you can roll back to a previous version and recover
the lost data.


## Table Versioning

Iceberg maintains a complete history of table changes, enabling you to audit or debug
issues. Each snapshot contains metadata about when changes occured etc.


## Compression and Serialization

Iceberg allows various compression techniques to reduce data size during storage and
transmission for optimal performance.


# Apache Iceberg Architecture

![Apache Iceberg Architecture](https://estuary.dev/static/0c05c453aad1d27b1fe9c2eaad18dc08/6c394/architecture_a4f0cb3c14.webp)

It comprises three main components: metadata, data files, and catalogs.


### Metadata

Iceberg uses metadata to store table schemas, snapshots, and partitioning information.
Metadata is organized into **manifest files** and **manifest lists**.

- **Manifest files:** contains details about data files, including their paths, record
  counts, and partition values.
- **Manifest lists:** acts as a directory for all manifest files, enabling quick table
  scans.


### Data files

Iceberg stores data in immutable file formats like Parquet, Avro, or ORC. When you update
an a table, Iceberg creates a new data file instead of modifying existing ones.


### Catalogs

Catalogs are responsible for table discovery and managements. Iceberg supports multiple
catalog implementations, including Hive Metastore, AWS Glue, and REST-based catalogs. A
catalog lets you interact with Iceberg tables using your preferred query engine.


## Using Apache Iceberg

If you are using Apache Spark as a query engine, you can create a new catalog with:

```
CREATE CATALOG cool_catalog USING 'org.apache.iceberg.spark.SparkCatalog'
OPTIONS ('type', 'hive', 'uri', 'thrift://localhost:9083');
```

Create a new table with the following command:

```
CREATE TABLE IF NOT EXISTS cool_catalog.db.cool_table (
    id BIGINT,
    data STRING
) USING iceberg;
```


Retrieve the data from the table using SQL:

```
SELECT * FROM cool_catalog.db.cool_table;
```

Adding a column (with schema evolution under the hood) in PySpark:

```python
from pathlib import Path
from pyspark.sql import SparkSession
from pyspark.sql.functions import lit


# Local warehouse for files.
pwd_warehouse = Path.pwd() / "warehouse"

spark = SparkSession.builder \
    .appName("Iceberg Schema Evolution") \
    .config("spark.sql.catalog.spark_catalog", "org.apache.iceberg.spark.SparkCatalog") \
    .config("spark.sql.catalog.spark_catalog.type", "hadoop") \
    .config("spark.sql.catalog.spark_catalog.warehouse", str(pwd_warehouse)) \
    .getOrCreate()

# Load an existing Iceberg table.
table = spark.read.format("iceberg").load("cool_catalog.db.cool_table")

# Add a new column.
updated_table = table.withColumn("new_column", lit("default_value"))

# Write back to the Iceberg table.
updated_table.write.format("iceberg").mode("overwrite").save("cool_catalog.db.cool_table")

```

Iceberg manages the metadata, so any readers continue to query the table seamlessly even
as the schema evolves.


Querying a snapshot by timestamp:

```python
timestamp = "2025-03-03T12:00:00.000Z"
historical_data = spark.read.format("iceberg") \
    .option("as-of-timestamp", timestamp) \
    .load("cool_catalog.db.cool_table")

historical_data.show()

```

Querying optimized partitions:

```python
partitioned_data = spark.read.format("iceberg").load("cool_catalog.db.cool_table")
filtered_data = partitioned_data.filter("event_date = '2025-01-01'")
filtered_data.show()

```

Writing data with ACID guarantees:

```python
new_data = spark.createDataFrame(
    [("user_1", "2025-01-10", 100.0)],
    ["user_id", "event_date", "amount"],
)

new_data.write.format("iceberg") \
    .mode("append") \
    .save("cool_catalog.db.cool_table")

```

## Apache Iceberg vs. Delta Lake

Delta Lake emerged as a competitor to Apache Iceberg, offering similar modern table format
features. However, Iceberg distinguishes itself in areas like multi-cloud compatibility
and advanced metadata management:

- **Time travel:** both Iceberg and Delta Lake support time travel.
- **Multi-Cloud support:** Delta Lake is tightly integrated with the Databricks ecosystem,
  whereas Iceberg is highly compatible across major cloud platforms.
- **Performance:** Iceberg uses a highly optimized metadat laeyr, which reduces query
  latency and improves scalability. Iceberg's metadata tree is designed for large-scale
  deployments, wheras Delta Lake's metadata management can becomes a bottleneck as data
  volumes grow.

