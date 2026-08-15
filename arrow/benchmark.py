import io
import time
from datetime import datetime, timedelta

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import polars as pl
import psycopg2
from adbc_driver_postgresql.dbapi import connect
from sqlalchemy import create_engine, text
from testcontainers.postgres import PostgresContainer


def plot_results(results):
    rows = [r["rows"] for r in results]
    adbc_times = [r["ADBC"] for r in results]
    sqlalchemy_times = [r["SQLAlchemy"] for r in results]
    copy_times = [r["COPY EXPERT"] for r in results]

    plt.figure(figsize=(10, 6))
    plt.plot(rows, adbc_times, marker="o", label="ADBC postgresql driver")
    plt.plot(rows, sqlalchemy_times, marker="s", label="SQLAlchemy (pandas.to_sql)")
    plt.plot(rows, copy_times, marker="^", label="COPY EXPERT (psycopg2)")

    plt.xscale("log")
    plt.yscale("log")
    plt.xlabel("Number of rows")
    plt.ylabel("Average insert time (s)")
    plt.title("PostgreSQL Insert Benchmark")
    plt.grid(True, which="both", ls="--", lw=0.5)
    plt.legend()
    plt.tight_layout()

    plt.savefig("benchmark.png")


# --- Benchmark Settings ---
TABLE_NAME = "benchmark_table"
ROW_SIZES = [
    100,
    500,
    1_000,
    5_000,
    10_000,
    50_000,
    100_000,
    500_000,
    1_000_000,
    10_000_000,
]


def check_row_count(db_params: dict, expected: int):
    """Check that the table has the expected number of rows using psycopg2"""
    conn = psycopg2.connect(**db_params)
    cur = conn.cursor()
    cur.execute(f"SELECT COUNT(*) FROM {TABLE_NAME}")
    count = cur.fetchone()[0]
    cur.close()
    conn.close()
    if count != expected:
        raise ValueError(f"Row count mismatch: expected {expected}, got {count}")


# --- Helper to generate mixed-type test data ---
def generate_data(n_rows: int):
    rng = np.random.default_rng()

    # text col
    letters = np.array(list("abcdefghijklmnopqrstuvwxyz"))
    rand_chars = rng.choice(letters, size=(n_rows, 10))
    text_col = np.apply_along_axis(lambda row: "".join(row), 1, rand_chars)

    # int col
    int_col = rng.integers(0, 1_000_000, size=n_rows)

    # float col
    float_col = rng.random(size=n_rows) * 1000.0

    # datetime col (random last 365 days)
    start_date = datetime.now() - timedelta(days=365)
    datetimes = [
        start_date + timedelta(days=int(x)) for x in rng.integers(0, 365, size=n_rows)
    ]

    # extra text cols
    extra_cols = {}
    for i in range(1, 6):
        rc = rng.choice(letters, size=(n_rows, 8))
        col = np.apply_along_axis(lambda row: "".join(row), 1, rc)
        extra_cols[f"extra{i}"] = col

    data = {
        "id": np.arange(1, n_rows + 1),
        "text_col": text_col,
        "int_col": int_col,
        "float_col": float_col,
        "datetime_col": datetimes,
    }
    data.update(extra_cols)

    pdf = pd.DataFrame(data)
    pl_df = pl.from_pandas(pdf)
    return pdf, pl_df


NUM_REPEATS = 5


def benchmark_method(method, method_func, *args):
    """Run method_func NUM_REPEATS times and return average time"""
    print(f"====== BENCHMARKING {method} =====")
    times = []
    for i in range(NUM_REPEATS):
        print(f" - iter={i + 1} ...")
        t = method_func(*args)
        times.append(t)
    avg_time = sum(times) / NUM_REPEATS
    return avg_time


def insert_adbc(pl_df: pl.DataFrame, db_uri: str, db_params, expected_rows):
    with connect(uri=db_uri) as conn:
        with conn.cursor() as cur:
            cur.execute(f"DROP TABLE IF EXISTS {TABLE_NAME}")
            cols_sql = """
                id BIGINT,
                text_col TEXT,
                int_col BIGINT,
                float_col DOUBLE PRECISION,
                datetime_col TIMESTAMP,
                extra1 TEXT,
                extra2 TEXT,
                extra3 TEXT,
                extra4 TEXT,
                extra5 TEXT
            """
            cur.execute(f"CREATE TABLE {TABLE_NAME} ({cols_sql})")
            conn.commit()
        start = time.time()
        pl_df.write_database(
            table_name=TABLE_NAME, connection=conn, if_table_exists="append"
        )
        conn.commit()
        elapsed_t = time.time() - start
        print(f"    - time: {elapsed_t}")
        check_row_count(db_params, expected_rows)
        return elapsed_t


def insert_sqlalchemy(pdf: pd.DataFrame, db_uri: str, db_params, expected_rows):
    engine = create_engine(db_uri)
    with engine.begin() as conn:
        conn.execute(text(f"DROP TABLE IF EXISTS {TABLE_NAME}"))
        cols_sql = """
            id BIGINT,
            text_col TEXT,
            int_col BIGINT,
            float_col DOUBLE PRECISION,
            datetime_col TIMESTAMP,
            extra1 TEXT,
            extra2 TEXT,
            extra3 TEXT,
            extra4 TEXT,
            extra5 TEXT
        """
        conn.execute(text(f"CREATE TABLE {TABLE_NAME} ({cols_sql})"))
    start = time.time()
    pdf.to_sql(
        TABLE_NAME,
        engine,
        if_exists="append",
        index=False,
        chunksize=10_000,
    )
    elapsed_t = time.time() - start
    print(f"    - time: {elapsed_t}")
    check_row_count(db_params, expected_rows)
    return elapsed_t


def insert_copy(pdf: pd.DataFrame, db_params: dict, expected_rows):
    conn = psycopg2.connect(**db_params)
    cur = conn.cursor()
    cur.execute(f"DROP TABLE IF EXISTS {TABLE_NAME}")
    cols_sql = """
        id BIGINT,
        text_col TEXT,
        int_col INT,
        float_col DOUBLE PRECISION,
        datetime_col TIMESTAMP,
        extra1 TEXT,
        extra2 TEXT,
        extra3 TEXT,
        extra4 TEXT,
        extra5 TEXT
    """
    cur.execute(f"CREATE TABLE {TABLE_NAME} ({cols_sql})")
    conn.commit()

    csv_buf = io.StringIO()
    # use ISO format for datetime
    pdf_out = pdf.copy()
    pdf_out["datetime_col"] = pdf_out["datetime_col"].apply(lambda x: x.isoformat())
    pdf_out.to_csv(csv_buf, header=False, index=False)
    csv_buf.seek(0)

    start = time.time()
    cols_list = ", ".join(pdf.columns)
    cur.copy_expert(f"COPY {TABLE_NAME} ({cols_list}) FROM STDIN WITH CSV", csv_buf)
    conn.commit()
    cur.close()
    conn.close()
    elapsed_t = time.time() - start
    print(f"    - time: {elapsed_t}")
    check_row_count(db_params, expected_rows)
    return elapsed_t


def main():
    with PostgresContainer("postgres:16") as postgres:
        db_uri = postgres.get_connection_url()
        db_params = {
            "host": postgres.get_container_host_ip(),
            "port": postgres.get_exposed_port(postgres.port),
            "dbname": postgres.dbname,
            "user": postgres.username,
            "password": postgres.password,
        }

        adbc_uri = db_uri.replace("+psycopg2", "")

        results = []
        for n in ROW_SIZES:
            print(f"Running {n} rows...")
            pdf, pl_df = generate_data(n)
            t1 = benchmark_method("ADBC", insert_adbc, pl_df, adbc_uri, db_params, n)
            t2 = benchmark_method(
                "SQLAlchemy", insert_sqlalchemy, pdf, db_uri, db_params, n
            )
            t3 = benchmark_method("COPY EXPERT", insert_copy, pdf, db_params, n)
            results.append({"rows": n, "ADBC": t1, "SQLAlchemy": t2, "COPY EXPERT": t3})

        headers = ["Rows", "ADBC (s)", "SQLAlchemy (s)", "COPY EXPERT (s)"]
        print("\n| " + " | ".join(headers) + " |")
        print("|" + "|".join(["---"] * len(headers)) + "|")
        for r in results:
            print(
                f"| {r['rows']:,} | {r['ADBC']:.4f} | {r['SQLAlchemy']:.4f} | {r['COPY EXPERT']:.4f} |"
            )

        plot_results(results)


if __name__ == "__main__":
    main()
