### Spark session config for Iceberg + Nessie + Spark + MinIO stack

```python
conf = (
    SparkConf()
        .setAppName('SparkIntegrTest')
        .set('spark.jars.packages',
             'org.apache.iceberg:iceberg-spark-runtime-3.5_2.12:1.7.1,org.apache.iceberg:iceberg-aws-bundle:1.7.1,org.projectnessie.nessie-integrations:nessie-spark-extensions-3.5_2.12:0.95.0')
        .set('spark.sql.extensions', 'org.apache.iceberg.spark.extensions.IcebergSparkSessionExtensions,org.projectnessie.spark.extensions.NessieSparkSessionExtensions')

        # nessie catalog params
        .set('spark.sql.catalog.nessie', 'org.apache.iceberg.spark.SparkCatalog')
        .set('spark.sql.catalog.nessie.catalog-impl', 'org.apache.iceberg.nessie.NessieCatalog')
        .set('spark.sql.catalog.nessie.uri', os.getenv('NESSIE_ENDPOINT'))
        .set('spark.sql.catalog.nessie.warehouse', 's3a://landing')
        .set('spark.sql.catalog.nessie.ref', 'main')
        .set("spark.sql.catalog.nessie.authentication.type", "NONE")

        # hadoop configs
        .set('spark.hadoop.fs.s3a.impl', 'org.apache.hadoop.fs.s3a.S3AFileSystem')
        .set('spark.hadoop.fs.s3a.path.style.access', 'true')
        .set('spark.hadoop.fs.s3a.connection.ssl.enabled', 'false')
        .set('spark.hadoop.fs.s3a.endpoint', os.getenv('MINIO_ENDPOINT'))
        .set('spark.hadoop.fs.s3a.access.key', os.getenv('MINIO_SERVER_ACCESS_KEY'))
        .set('spark.hadoop.fs.s3a.secret.key', os.getenv('MINIO_SERVER_SECRET_KEY'))
        .set('spark.hadoop.fs.s3a.endpoint.region', os.getenv('AWS_REGION'))
        .set('aws.region', os.getenv('AWS_REGION'))
)
```
