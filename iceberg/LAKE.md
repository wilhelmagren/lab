## Iceberg catalogs

Iceberg supports several catalog types.
- **Hadoop catalog** - stores table/namespace metadata on a filesystem (HDFS/S3/etc.), no separate service
- **Hive Metastore (HMS) catalog** - uses the Hive metastore
- **REST catalog** - a network service that implements Iceberg's REST API
- **JDBC catalog** - stores metadata in a relational DB via JDBC (requires transactional DB semantics)
- **Nessie catalog** - a versioned ("Git-like") catalog
- **AWS Glue catalog** - official Iceberg integration with the AWS Glue Data Catalog

Hadoop catalog expects a filesystem with **atomic rename**; S3/MinIO object storage does not provide
that, so concurrent writers aren't safe.


## Target architecture

- **Catalog**: run **Nessie** and expose its **Iceberg REST** endpoint. This gives us Git-like
branches/tags and cross-table semantics, and it's compatible with clients that "speak" Iceberg REST
(including Doris).
- **Storage**: MinIO (S3-compatible) for data + metadata files, accessed via **Iceberg S3FileIO**.
- **Compute**:
    - **Spark** for ingestion/transform/writes (either use Nessie's custom catalog or the REST endpoint).
    - **Doris** for BI/serving, connected through Iceberg REST (and MinIO creds). Doris reads *and*
      can write Iceberg.
