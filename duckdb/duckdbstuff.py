import duckdb


if __name__ == "__main__":
    with duckdb.connect("pond.duckdb") as conn:
        conn.load_extension("postgres")
        conn.load_extension("ducklake")

        conn.sql("INSTALL ducklake;")

        # This creates a file called `metadata.ducklake` which is a DuckDB database with
        # the DuckLake schema.
        conn.sql("ATTACH 'ducklake:metadata.ducklake' AS my_ducklake;")
        conn.sql("USE my_ducklake;")

        # There is no need to explicitly define a schema for the service table,
        # nor is it necessary to use a COPY...FROM statement. The shcema is
        # inferred and automatically detects that it is a gzip-compressed CSV file.
        print("CREATE TABLE IF NOT EXISTS services AS FROM 'services-2023.csv.gz';")
        conn.sql("CREATE TABLE IF NOT EXISTS services AS FROM 'services-2023.csv.gz';")
        conn.sql("SELECT * FROM services LIMIT 20;").show()

        print("SELECT format('{:,}', count(*)) AS num_services FROM services;")
        conn.sql("SELECT format('{:,}', count(*)) AS num_services FROM services;").show()

        print("now we find the busiest station per month")
        conn.sql("""
            SELECT 
                 month("Service:Date") AS month,
                 "Stop:Station name" AS station,
                 count(*) AS num_services
            FROM services GROUP BY month, station LIMIT 5;
        """).show()

        print("Using GROUP BY ALL feature instead...")
        conn.sql("""
            CREATE TABLE IF NOT EXISTS services_per_month AS
                SELECT
                    month("Service:Date") AS month,
                    "Stop:Station name" AS station,
                    count(*) AS num_services
                FROM services GROUP BY ALL;
        """)

        conn.sql("""
            SELECT * FROM services_per_month
        """).show()

        conn.sql("""
            SELECT
                month,
                arg_max(station, num_services) AS station,
                max(num_services) AS num_services
            FROM services_per_month WHERE month <= 6 GROUP BY ALL;
        """).show()

        conn.sql("""
            SELECT month, month_name, array_agg(station) AS top3_stations
            FROM (
                SELECT
                    month,
                    strftime(make_date(2023, month, 1), '%B') AS month_name,
                    rank() OVER
                        (PARTITION BY month ORDER BY num_services DESC) AS rank,
                    station,
                    num_services
                FROM services_per_month
                WHERE month BETWEEN 6 AND 8
            )
            WHERE RANK <= 3 GROUP BY ALL ORDER BY MONTH;
        """).show()

        conn.sql("""
            SELECT
                month,
                strftime(make_date(2023, month, 1), '%B') AS month_name,
                max_by(station, num_services, 3) AS stations,
            FROM services_per_month
            WHERE month BETWEEN 6 AND 8 GROUP BY ALL ORDER BY month;
        """).show()

        print("we can directly query parquet files over https or s3")
        conn.sql("""
            SELECT "Service:Date", "Stop:Station name"
            FROM 'https://blobs.duckdb.org/nl-railway/services-2023.parquet' LIMIT 3;
        """).show()

