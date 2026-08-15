import duckdb as ddb
# import pandas as pd
import polars as pl


if __name__ == "__main__":
    ddb.sql("SELECT 42").show()

    # Relations can be references in subsequent queries by storing them inside
    # variables and using them as tables. This way queries can be cosntructed
    # incrementally.
    r1 = ddb.sql("SELECT 42 AS i")
    ddb.sql("SELECT i * 2 AS k FROM r1").show()

    # DuckDB can directly query Pandas DataFrames, Polars DataFrames and
    # Arrow tables. These are, however, read-only.
    # df = pd.DataFrame({"a": 42})
    # ddb.sql("SELECT * FROM db").show()

    pl_df = pl.DataFrame({"b": [34.5]})
    ddb.sql("SELECT * FROM pl_df").show()

