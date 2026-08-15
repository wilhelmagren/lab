import orjson
import json
import polars as pl
import pyarrow as pa

schema = {
    "user_id": pl.Int64,
    "username": pl.Utf8,
    "last_login": pl.Datetime(time_unit="us", time_zone=None),
    "banned": pl.Boolean,
    "friends": pl.List(
        pl.Struct(
            [
                pl.Field("user_id", pl.Int64),
                pl.Field("username", pl.Utf8),
                pl.Field("since", pl.Datetime(time_unit="us", time_zone=None)),
            ]
        )
    ),
}

schema2 = {
    pl.Field("user_id", pl.Int64),
    pl.Field("username", pl.Utf8),
    pl.Field("last_login", pl.Datetime(time_unit="us", time_zone=None)),
    pl.Field("banned", pl.Boolean),
    pl.Field(
        "friends",
        pl.List(
            pl.Struct(
                [
                    pl.Field("user_id", pl.Int64),
                    pl.Field("username", pl.Utf8),
                    pl.Field("since", pl.Datetime(time_unit="us", time_zone=None)),
                ]
            )
        ),
    ),
}


def main() -> None:
    df = pl.read_ndjson(
        source="./example-data-1000000.jsonl",
        schema=schema,
    )

    print(df.head())

    friends_struct = pa.large_list(
        pa.struct(
            [
                pa.field("user_id", pa.int64()),
                pa.field("username", pa.large_string()),
                pa.field("since", pa.timestamp("us")),
            ],
        )
    )

    arrow_schema = pa.schema(
        [
            pa.field("user_id", pa.int64()),
            pa.field("username", pa.large_string()),
            pa.field("last_login", pa.timestamp("us")),
            pa.field("banned", pa.bool_()),
            pa.field("friends", friends_struct),
        ]
    )

    found_schema = df.to_arrow().schema

    if found_schema.equals(arrow_schema):
        print("Schema matches!")
    else:
        print("!!! Schema mismatch")
        print("Expected:\n", arrow_schema)
        print("Actual:\n", found_schema)

    """
    df = df.with_columns(
        [
            pl.col("friends").map_elements(
                lambda v: b"\0x1" + orjson.dumps(v),
                return_dtype=pl.Binary,
            )
        ]
    )
    """

    df = df.with_columns(
        [
            pl.col("friends")
            .map_elements(
                lambda s: s.str,
                return_dtype=pl.Binary,
            )
            .alias("friends_bin")
        ]
    )
    print(df.head())


if __name__ == "__main__":
    main()
