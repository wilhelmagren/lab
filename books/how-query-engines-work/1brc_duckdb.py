import time

import duckdb

t_start = time.perf_counter()
duckdb.sql("""
    SELECT
        station_name,
        min(measurement) AS min,
        max(measurement) AS max,
        avg(measurement) AS avg
    FROM "data/weather_stations.parquet"
    GROUP BY station_name
""").show()
print(f"[DuckDB] {time.perf_counter() - t_start:.3f}s")
