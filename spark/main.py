from pyspark.sql import SparkSession
from pyspark.sql.functions import lit


def main():
    print("Hello from pyspark!")

    """
    from pyspark import SparkConf, SparkContext
    conf = SparkConf().setAppName("app").setMaster("spark://<MASTER_IP>:7077")
    sc = SparkContext(conf=conf)

    ...

    """

    spark = SparkSession.builder.appName("SparkApp").getOrCreate()
    df = spark.read.csv("./data/big_mart_sales.csv", header=True, inferSchema=True)
    df.createOrReplaceTempView("big_mart_sales")

    print("How many rows?")
    df_n_rows = spark.sql("SELECT COUNT(*) FROM big_mart_sales")
    print(df_n_rows.show())

    print("Query with SQL")
    df_sql = spark.sql("""
    SELECT
        Item_weight,
        AVG(Item_Weight),
        AVG(Item_Visibility),
        AVG(Item_MRP),
        AVG(Outlet_Establishment_Year),
        AVG(Item_Outlet_Sales)
    FROM big_mart_sales
    GROUP BY Item_Weight
    LIMIT 5
    """)
    print(df_sql.show())

    print("Query with dataframe api")
    df_df = df.groupBy("Item_Weight").avg()
    print(df_df.show(5))

    print("Reading a json file")
    df_json = spark.read.json("./data/drivers.json")
    df_json.createOrReplaceTempView("drivers")
    print(df_json.show(5))

    print("Adding lit column to df_sql")
    df_sql = df_sql.withColumn("nationality", lit("British"))
    df_sql.createOrReplaceTempView("big_mart_sales")

    print("Joining tables with SQL")
    df_sql_join = spark.sql("""
        SELECT a.nationality, a.Item_Weight, b.name FROM big_mart_sales AS a
        JOIN drivers AS b ON a.nationality = b.nationality
    """)

    print(df_sql_join.show(5))


if __name__ == "__main__":
    main()
