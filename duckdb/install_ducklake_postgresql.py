import duckdb as ddb


if __name__ == "__main__":
    with ddb.connect() as conn:
        conn.install_extension("postgres")
        conn.install_extension("ducklake")

        conn.load_extension("postgres")
        conn.load_extension("ducklake")

        conn.sql("SELECT * FROM duckdb_extensions()").show()
