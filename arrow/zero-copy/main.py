import polars as pl

df = pl.read_ndjson(
    "example-data-10000.jsonl",
    schema={
        "user_id": pl.Int64,
        "username": pl.Utf8,
        "last_login": pl.Datetime(time_zone=None),
        "banned": pl.Boolean,
    },
)
print(df.head())

# Convert the column to Arrow separately
arrow_from_col = df["user_id"].to_arrow()

# Convert the full DataFrame
arrow_from_df = df.to_arrow()

# Access the same column in the full Arrow table
arrow_col_in_table = arrow_from_df.column("user_id").chunk(0)

# Compare the memory addresses of the actual data buffer
buf_from_col = arrow_from_col.buffers()[1]
buf_from_table = arrow_col_in_table.buffers()[1]

print("Same buffer?", buf_from_col.address == buf_from_table.address)
