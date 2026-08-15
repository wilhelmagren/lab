import json
import uuid
import adbc_driver_postgresql.dbapi as adbc
import polars as pl


def main():
    print("Hello from adbc!")
    json_obj1 = {"inner": "ket", "another": 1337}
    json_obj2 = {"inner": "zot", "another": 4}
    json_obj3 = {"inner": "sad", "details": [{"cool": False}, {"cat": "dog"}]}

    json_objs = [json_obj1, json_obj3]

    df = pl.DataFrame({
        "a": [uuid.uuid4().bytes, uuid.uuid4().bytes],
        "b": ["foo", "bar"],
        "c": [1234, 567],
        # "d": [b"\x01" + json.dumps(obj).encode("utf-8") for obj in json_objs],
        "d": [json.dumps(json_obj2), json.dumps(json_obj3)],
    })

    df = df.with_columns([pl.col("d").map_elements(lambda x: b"\x01" + x.encode("utf-8"))])

    print(df.head())
    print(df.schema)

    with adbc.connect("postgresql://postgres:123@127.0.0.1:5432/postgres") as adbc_conn:
        df.write_database("test.cool", adbc_conn, if_table_exists="append")


if __name__ == "__main__":
    main()
