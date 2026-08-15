from pyspark.sql import SparkSession
import time

spark = SparkSession.builder.appName("WithoutComet").master("local[*]").getOrCreate()

# Sample workload
data = [(i, i * 2) for i in range(10_000_000)]
df = spark.createDataFrame(data, ["x", "y"])

start = time.time()
count = df.filter("y % 10 = 0").select("x").count()
end = time.time()

print()
print("=" * 80)
print(f"Count without Comet: {count}")
print(f"Time without Comet:  {end - start:.3f} seconds")
print("=" * 80)
print()

spark.stop()
