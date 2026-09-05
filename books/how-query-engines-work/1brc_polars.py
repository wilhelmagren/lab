import time

import polars as pl

t_start = time.perf_counter()
pl.scan_parquet("./data/weather_stations.parquet").select(
    "station_name",
    "measurement",
).group_by("station_name").agg(
    pl.min("measurement").alias("min"),
    pl.max("measurement").alias("max"),
    pl.mean("measurement").alias("avg"),
).collect(engine="streaming").show()
print(f"[Polars] {time.perf_counter() - t_start:.3f}s")
