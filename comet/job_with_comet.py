from pyspark.sql import SparkSession
import time

spark = SparkSession.builder.appName("WithComet").master("local[*]").getOrCreate()

data = [(i, i * 2) for i in range(10_000_000)]
df = spark.createDataFrame(data, ["x", "y"])

start = time.time()
count = df.filter("y % 10 = 0").select("x").count()
end = time.time()

print()
print("=" * 80)
print(f"Count with comet: {count}")
print(f"Time with comet:  {end - start:.3f}s")
print("=" * 80)
print()

spark.stop()
