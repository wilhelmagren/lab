import polars as pl


a = pl.DataFrame(
    {
        "id": [1, 2, 3, 5],
        "name": ["guldan", "godwyn", "gwynn", "artorias"],
        "age": [123445, 89, 10000000, 6969],
    }
)
a.show()

b = pl.DataFrame(
    {
        "id": [1, 2, 5, 3, 4, 1, 5],
        "job": [
            "warlock",
            "gigachad",
            "coolboy",
            "cringelord",
            "solaire",
            "wheelchair",
            "abysswalker",
        ],
        "salary": [-123000.41, 9999999.12, -3.14, 133.7, 69, 420, 9.41,],
    }
)
b.show()

a.write_parquet("users.parquet")
b.write_parquet("jobs.parquet")
