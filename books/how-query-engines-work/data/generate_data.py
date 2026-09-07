from pathlib import Path

import polars as pl


DATA_DIR = Path("./")
DATA_DIR.mkdir(exist_ok=True)


users = pl.DataFrame(
    {
        "id": pl.Series(
            [1, 2, 3, 4, 5, 6, 7, 8],
            dtype=pl.Int64,
        ),
        "name": pl.Series(
            [
                "guldan",
                "godwyn",
                "gwynn",
                "malenia",
                "artorias",
                "sif",
                "radahn",
                "morgott",
            ],
            dtype=pl.String,
        ),
        "age": pl.Series(
            [
                120,  # included
                40,   # filtered out
                101,  # included
                99,   # filtered out
                250,  # included
                100,  # included, boundary case
                500,  # included, but has no job
                20,   # filtered out
            ],
            dtype=pl.Int64,
        ),
    }
)


jobs = pl.DataFrame(
    {
        "id": pl.Series(
            [
                1,
                1,
                1,   # 1 -> three matches
                2,
                3,
                5,
                5,   # 5 -> two matches
                6,
                6,
                99,  # no matching user
            ],
            dtype=pl.Int64,
        ),
        "job": pl.Series(
            [
                "warlock",
                "shaman",
                "destroyer",
                "knight",
                "king",
                "knight",
                "abyss_walker",
                "wolf",
                "guardian",
                "orphan",
            ],
            dtype=pl.String,
        ),
        "salary": pl.Series(
            [
                -123_000.41,
                420.0,
                -61_290.205,
                89.0,
                133.7,
                -3.14,
                9.41,
                10.0,
                None,       # test aggregate NULL handling
                1_000_000.0,
            ],
            dtype=pl.Float64,
        ),
    }
)


users.write_parquet(DATA_DIR / "users.parquet")
jobs.write_parquet(DATA_DIR / "jobs.parquet")


print("users.parquet")
print(users)

print("\njobs.parquet")
print(jobs)
