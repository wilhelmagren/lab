import polars as pl

pl.read_parquet("./data/titanic.parquet").group_by("Pclass", "Sex").agg(
    pl.sum("Survived"),
).sort("Pclass", "Sex").show()

pl.scan_parquet("./data/weather_stations.parquet").select(
    "station_name",
    "measurement",
).filter(pl.col("station_name") == pl.lit("Hualien")).group_by("station_name").agg(
    pl.min("measurement").alias("min"),
    pl.max("measurement").alias("max"),
    pl.mean("measurement").alias("avg"),
).collect(engine="streaming").show()
