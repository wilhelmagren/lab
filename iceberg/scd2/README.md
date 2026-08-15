# Slowly Changing Dimensions

**Slowly Changing Dimensions** (SCD) is a dimension that stores data which maye change
over time. Common examples of SCDs include geographical locations, customer details, or
product attributes.

## Type 0: retain original

The **Type 0** dimension attributes never change and are assigned to attributes that have
durable values or are described as 'Original'. Examples: *date of birth*, *original credit
score*. Type 0 applies to most date dimension attributes.


## Type 1: overwrite

This method overwrites old data with new data, and therefore does not track historical
data. Example of a supplier table:

|Supplier_Key|Supplier_Code|Supplier_Name|Supplier_State|
|--|--|--|--|
|123|ABC|Acme Supply Co|CA|

In the above example, Supplier_Code is the **natural key** and Supplier_Key is a
**surrogate key**. Technically, the surrogate key is not necessary, since the row will be
unique by the natural key (Supplier_Code).

If the supplier relocates the headquarters to Illinois the record would be overwritten
with:

|Supplier_Key|Supplier_Code|Supplier_Name|Supplier_State|
|--|--|--|--|
|123|ABC|Acme Supply Co|IL|

The disadvantage of the Type 1 method is that there is no history in the data warehouse.
It has the advantage however that it is easy to maintain and saves on storage.

Note that if one has calculated an aggregate table summarizing facts by supplier state, it
will need to be recalculated when the Supplier_State is changed.


## Type 2: add new row

This method tracks historical data by creating multiple records for a given **natural
key** in the dimensional tables with separate **surrogate keys** and/or different version
numbers. Unlimited history is preserved for each insert. The natural key in these examples
is the 'Supplier_Code' of 'ABC'.

For example, if the supplier relocates to Illinois the version numbers will be incremented
sequentially:

|Supplier_Key|Supplier_Code|Supplier_Name|Supplier_State|Version|
|--|--|--|--|--|
|123|ABC|Acme Supply Co|CA|0|
|123|ABC|Acme Supply Co|IL|1|
|123|ABC|Acme Supply Co|NY|2|

Another method is to add the 'effective dates' columns:

|Supplier_Key|Supplier_Code|Supplier_Name|Supplier_State|Start_Date|End_Date|
|--|--|--|--|--|--|
|123|ABC|Acme Supply Co|CA|2000-01-01T00:00:00|2004-12-22T00:00:00|
|123|ABC|Acme Supply Co|IL|2004-12-22T00:00:00|NULL|

The 'Start date/time' of the second row is equal to the 'End date/time' of the previous
row. The null 'End_Date' in row two indicates the current tuple version. A standardized
surrogate high date (e.g. '9999-12-31') may instead be used as an end date so that
null-value substitution is not required when querying. In some database software, using an
artificial high date value could cause performance issues, that using a null value would
prevent.

And a third method uses an effective date and a current flag:

|Supplier_Key|Supplier_Code|Supplier_Name|Supplier_State|Effective_Date|Is_Current|
|--|--|--|--|--|--|
|123|ABC|Acme Supply Co|CA|2000-01-01T00:00:00|False|
|123|ABC|Acme Supply Co|IL|2004-12-22T00:00:00|True|

The 'Is_Current' flag value of 'True' indicates the current tuple version. Transactions
that reference a particular **surrogate key** (Supplier_Key) are then permanently bound to
the time slices defined by that row of the slowly changing dimension table. An aggregate
table summarizing facts by supplier state continues to reflect the historical state, i.e.,
the state the supplier was in at the time of the transaction; no update is needed!

However, to reference the entity via the natural key, it is necessary to remove the unique
constraint making **referential integrity** impossible.

If there are retroactive changes made to the contents of the dimension, or if new
attributes are added to the dimension (for example a 'Sales_Rep' column) which have
different effective dates from those already defined, then this can result in the existing
transactions needing to be updated to reflect the new situation. **This can be an expensive
database operation, so Type 2 SCDs are not a good choice if the dimensional model is
subject to frequent change.**


## Remaining types:

- **Type 3:** add new attribute
- **Type 4:** add history table
- **Type 5:** is a combination of types 1 and 4
- **Type 6:** is a combination of types 1, 2, and 3
- **Type 7:** hybrid, both surrogate and natural key
 

## Apache Iceberg and SCD Type 2

> [!Important] 
> The material below is taken from the blog post at [link](https://aws.amazon.com/blogs/big-data/implement-historical-record-lookup-and-slowly-changing-dimensions-type-2-using-apache-iceberg/)
> written by Tomohiro Tanaka and Noritaka Sekiyama.

This method creates new records for each data change while preserving old ones, thus
maintaining a full history. **How can we retrieve the history of given records?**

Example, product **Heater** in an ecommerce database:

|id|name|price|
|--|--|--|
|00001|Heater|250|

Apache Iceberg offers a feature known as the **change log view**. The change log view
provides a view of all changes made to the table over time, making it straightforward to
query and analyze the history of any records. With change log view, we can easily track
insertions, updates, and deletions, giving us a complete picture of how our data has
evolved.

For our heater example, Iceberg's change log view would allow us to effortlessly retrieve
a timeline of all price changes, complete with timestamps and other relevant metadata, as
shown below:

|id|name|price|_change_type|
|--|--|--|--|
|00001|Heater|250|INSERT|
|00001|Heater|250|UPDATE_BEFORE|
|00001|Heater|500|UPDATE_AFTER|

SCD Type 2 is a key concept in data warehousing and historical data management and is
particularly relevant to Change Data Capture (CDC) scenarios. SCD Type 2 requires
additional fields such as 'effective_start_date', 'effective_end_date' and 'current_flag'
to manage historical records. Below is what SCD Type 2 looks like with our Heater example
assuming that the update operation is peformed on December 11th 2024:

|id|name|price|effective_start_date|effective_end_date|current_flag|
|--|--|--|--|--|--|
|00001|Heater|250|2024-12-10|2024-12-11|FALSE|
|00001|Heater|500|2024-12-11|NULL|TRUE|

In traditional implementations on data warehouses, SCD Type 2 requires its specific
handling in all 'INSERT', 'UPDATE', and 'DELETE' operations that affect those additional
columns. For example, to update the price of the product, you would need to run the
following query:

```sql
UPDATE product SET effective_end_date = '2024-12-11', current_flag = false
WHERE product_id = '00001' AND current_flag = true;

INSERT INTO product (id, name, price, effective_start_date, effective_end_date, current_flag)
VALUES ('00001', 'Heater', 500, '2024-12-11', NULL, true);
```

For modern data lakes, we propose a new approach to implement SCD Type 2. With Iceberg,
you can create a dedicated view of SCD Type 2 on top of the change log view, eliminating
the need to implement specific handling to make changes on SCD Type 2 tables. With this
approach you can keep managing Iceberg tables without complexity considering SDC Type 2
specification. ANytime when you need SCD Type 2 snapshots of your Iceberg table, you can
create the corresponding representation from the change log view.

This streamlined method not only makes the implementation of SCD Type 2 more
straightforward, but also offers improved performance and scalability for handling large
volumes of historical data in CDC scenarios. It represents a significant advancement in
historical data management, merging traditional data warehousing concepts with modern big
data capabilities.


### Create an Iceberg table

Initialize your SparkSession with Iceberg config:

```python
from pathlib import Path
from pyspark.sql import SparkSession
from pyspark.sql.functions import lit


# Your catalog name.
CATALOG_NAME = "scd_catalog"

# Local warehouse for files.
WAREHOUSE_PATH = Path.pwd() / "warehouse"

spark = SparkSession.builder \
    .appName("Iceberg SCD Type 2") \
    .config(f"spark.sql.catalog.{CATALOG_NAME}", "org.apache.iceberg.spark.SparkCatalog") \
    .config(f"spark.sql.catalog.{CATALOG_NAME}.type", "hadoop") \
    .config(f"spark.sql.catalog.{CATALOG_NAME}.warehouse", str(WAREHOUSE_PATH)) \
    .config("spark.sql.extensions", "org.apache.iceberg.spark.extensions.IcebergSparkSessionExtensions") \
    .config("spark.sql.defaultCatalog", CATALOG_NAME) \
    .getOrCreate()

```

Create the initial dataset:

```python
from datetime import datetime
from pyspark.sql import Row

ut = datetime.now().strftime("%Y-%m-%dT%H:%M:%S")

products = [
    {"id": "00001", "name": "Heater", "price": 250, "category": "Electronics", "updated_at": ut},
    {"id": "00002", "name": "Thermostat", "price": 400, "category": "Electronics", "updated_at": ut},
    {"id": "00003", "name": "Television", "price": 600, "category": "Electronics", "updated_at": ut},
]

df_products = spark.createDataFrame(Row(**p) for p in products)
df_products.createOrReplaceTempView("tmp")

spark.sql(f"""
CREATE TABLE IF NOT EXISTS {CATALOG_NAME}.db.products
USING ICEBERG LOCATION '{WAREHOUSE_PATH}'
AS SELECT * FROM tmp;
""")


```

Run the below query to show the product data in the Iceberg table:

```sql
SELECT * FROM scd_catalog.db.products ORDER BY id;
```

Next we will simulate a CDC workflow by manually applying some changes to the Iceberg
table:

```python
from datetime import datetime
from pyspark.sql import Row

ut = datetime.now().strftime("%Y-%m-%dT%H:%M:%S")

products_updates = [
    {"id": "00001", "name": "Heater", "price": 500, "category": "Electronics", "updated_at": ut},
    {"id": "00006", "name": "Chair", "price": 50, "category": "Furniture", "updated_at": ut},
]

df_products_updates = spark.createDataFrame(Row(**p) for p in products_updates)
df_products_updates.createOrReplaceTempView("products_upsert")

spark.sql(f"""
MERGE INTO {CATALOG_NAME}.db.products t USING
(SELECT * FROM products_upsert) u ON
t.id = u.id
WHEN MATCHED THEN UPDATE SET t.name = u.name, t.price = u.price, t.category = u.category,
t.updated_at = u.updated_at
WHEN NOT MATCHED THEN INSERT *;
""")

# Delete television product from the Iceberg table
spark.sql(f"""
DELETE FROM {CATALOG_NAME}.db.products WHERE product_id = '00003';
""")

```

Now you can run the following select statement to show the updated product data in the
Iceberg table:

```sql
SELECT * FROM scd_catalog.db.products ORDER BY id;
```


and now you can execute the change log view to track the record changes:

```sql
CALL scd_catalog.system.create_changelog_view(
    table => 'db.products',
    changelog_view => 'products_clv',
    identifier_columns => array('id')
);
```

and to view the change log for 'Heater' you can run:

```sql
SELECT id, name, price, category, updated_at, _change_type
FROM products_clv WHERE product_id = '00001'
ORDER BY _change_ordinal, _change_type DESC;
```

Using Iceberg's change log view you can obtain the history of a given record directly from
the Iceberg table's history, without needing to create a separate table for managing
record history. Next, lets implement SCD Type 2 using the change log view.

The change log view (products_clv) that you created previously has a schema that's similar
ot the schema defined in the SCD Type 2 specifications. For this change log view, you add
the 'effective_start', 'effective_end', 'is_current' columns.

```sql
WITH clv_snapshots as (
    SELECT
        clv.*,
        s.snapshot_id,
        s.commited_at,
        s.commited_at as effective_start
    FROM products_clv clv
    JOIN scd_catalog.db.products.snapshots s
    ON clv._commit_snapshot_id = s.snapshot_id
)
SELECT
    id,
    name,
    price,
    category,
    updated_at,
    effective_start,
    CASE
        WHEN effective_start != l_part_commited_at
            OR _change_type = 'UPDATED_BEFORE' THEN l_part_commited_at
        ELSE CAST(null as timestamp)
    END as effective_end,
    CASE
        WHEN effective_start != l_part_commited_at
            OR _change_type = 'UPDATED_BEFORE'
            OR _change_type = 'DELETE' THEN CAST(false as boolean)
        ELSE CAST(true as boolean)
    END as is_current
FROM (SELECT *, MAX(committed_at) OVER (PARTITION BY id, updated_at) as l_part_commited_at FROM clv_snapshots)
WHERE _change_type != 'UPDATED_BEFORE'
ORDER BY id, _change_ordinal;
```

Run the following query to retrieve deleted or updated records on a specific period:

```sql
SELECT id, name, price, category, updated_at, effective_start, effective_end, is_current
FROM scdt2 WHERE id IN (SELECT id FROM scd2
WHERE (_change_type = 'DELETE' or _change_type = 'UPDATE_AFTER')
AND effective_start BETWEEN '2025-03-13T14:00:00' and '2025-03-13T15:00:00')
ORDER BY id, effective_start;
```

As another example, run the following query to retrieve the latest records at a specific
point in time from the SCD Type 2 table by filtering with 'is_current = true' for current
data reporting:

```sql
SELECT id, name, price, category, updated_at
FROM scdt2 WHERE is_current = true ORDER BY id;
```

Using Iceberg's change log view (clv) enables you to implement SCD Type 2 wihout making
any changes to the Iceberg table itself. It also eliminates the need for creating or
managing an additional table for SCD Type 2.

