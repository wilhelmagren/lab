import time
import sys
import json
import os

import matplotlib.pyplot as plt
import pandas as pd
import polars as pl


POLARS_DTYPE_MAP = {
    "Int64": pl.Int64,
    "Utf8": pl.Utf8,
    "String": pl.String,
    "Boolean": pl.Boolean,
    "Datetime": pl.Datetime(time_unit="us", time_zone=None),
}

PANDAS_DTYPE_MAP = {
    "Int64": "int64",
    "Utf8": "string",
    "String": "string",
    "Boolean": "bool",
    "Datetime": "string",
}


def json_schema_to_polars(schema):
    return {key: POLARS_DTYPE_MAP[value] for key, value in schema.items()}


def json_schema_to_pandas(schema):
    return {key: PANDAS_DTYPE_MAP[value] for key, value in schema.items()}


def main():
    with open("example-schema2.json", "rb") as f:
        schema = json.load(f)

    polars_schema = json_schema_to_polars(schema)
    pandas_schema = json_schema_to_pandas(schema)

    files = os.listdir(".")

    polars_timings = {}
    pandas_timings = {}

    for i, file in enumerate(files):
        if ".jsonl" in file:
            filesize = file.split("-")[2].split(".")[0]
            print(f"{file=}")

            with open(file, "rb") as f:
                sys.stdout.write("POLARS ...")
                sys.stdout.flush()
                t_start = time.perf_counter()
                df = pl.read_ndjson(
                    source=f,
                    schema=polars_schema,
                )
                t_elapsed = time.perf_counter() - t_start
                sys.stdout.write(f" {t_elapsed:.4f} s\n")
                polars_timings[filesize] = t_elapsed

                if int(filesize) >= 50000000:
                    print("SKIPPING PANDAS OOM")
                    continue

                sys.stdout.write("PANDAS ...")
                sys.stdout.flush()
                t_start = time.perf_counter()
                df = pd.read_json(
                    path_or_buf=f,
                    lines=True,
                    convert_dates=["last_login"],
                    dtype=pandas_schema,
                )

                t_elapsed = time.perf_counter() - t_start
                sys.stdout.write(f" {t_elapsed:.4f} s\n")
                pandas_timings[filesize] = t_elapsed

    plt.figure(figsize=(10, 6))
    plt.plot(
        [int(k) for k in polars_timings.keys()],
        list(polars_timings.values()),
        marker="+",
        color="tab:blue",
        label="Polars (pl.read_ndjson)",
    )
    plt.plot(
        [int(k) for k in pandas_timings.keys()],
        list(pandas_timings.values()),
        marker=".",
        color="tab:orange",
        label="Pandas (pd.read_json)",
    )
    plt.xscale("log")
    plt.yscale("log")
    plt.xlabel("Number of rows")
    plt.ylabel("Time [s]")
    plt.title(".jsonl reading + dtype parsing", fontweight="bold")
    plt.grid(True, which="both", ls="--", lw=0.5)
    plt.legend()
    plt.tight_layout()
    plt.savefig("ndjson_benchmark.png")

    for a, b in polars_timings.items():
        print(a, b)

    for a, b in pandas_timings.items():
        print(a, b)


if __name__ == "__main__":
    main()
