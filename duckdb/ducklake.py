import duckdb as ddb


if __name__ == "__main__":
    ddb.load_extension("ducklake")

    print("Using lake: ankdamm.ducklake")
    ddb.sql("ATTACH 'ducklake:ankdamm.ducklake' AS ankdamm;")
    ddb.sql("USE ankdamm;")

    print("Import the dataset into a new table using https and csv_reader")
    ddb.sql("CREATE TABLE IF NOT EXISTS nl_train_stations AS FROM 'https://blobs.duckdb.org/nl_stations.csv';")

    print("Contents of csv")
    ddb.sql("SELECT * FROM nl_train_stations LIMIT 5;").show()

    print("globbing local file storage")
    ddb.sql("FROM glob('ankdamm.ducklake.files/*');").show()
    ddb.sql("FROM 'ankdamm.ducklake.files/*.parquet' LIMIT 10;").show()

    print("Update name of station")
    ddb.sql("""
        UPDATE nl_train_stations SET name_long = 'Johan Cruijff ArenA' WHERE code = 'ASB';
    """)

    ddb.sql("SELECT name_long FROM nl_train_stations WHERE code = 'ASB';").show()
    print("More files have appeared now!")
    ddb.sql("FROM glob('ankdamm.ducklake.files/*');").show()

    ddb.sql("FROM 'ankdamm.ducklake.files/ducklake-*-delete.parquet';").show()

    ddb.sql("FROM ankdamm.snapshots()").show()

    print("We can query the snapshotted tables")
    print(" !!!!!!!!!!!!!!!!!!! TIME TRAVEL !!!!!!!!!!!!!!!!")
    print("past - old name:")
    ddb.sql("SELECT name_long FROM nl_train_stations AT (VERSION => 1) WHERE code = 'ASB';").show()
    print("present - New name:")
    ddb.sql("SELECT name_long FROM nl_train_stations AT (VERSION => 2) WHERE code = 'ASB';").show()
