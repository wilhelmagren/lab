import json
from datetime import datetime
from tqdm import tqdm


def main() -> None:
    filename_base = "example-data-"
    rowset = [
        10,
        100,
        1000,
        5000,
        10000,
        15000,
        20000,
        50000,
        100000,
        200000,
        500000,
        1000000,
    ]

    data = {
        "user_id": 1337,
        "username": "dev0rce",
        "last_login": datetime.now().isoformat(),
        "banned": False,
        "friends": [
            {"user_id": 14, "username": "foo", "since": datetime.now().isoformat()},
            {"user_id": 9814455, "username": "bar"},
        ],
    }

    for rows in tqdm(rowset, desc="Generating files"):
        filename = filename_base + f"{rows}.jsonl"
        tqdm.write(f"generating file {filename}...")
        with open(filename, "a") as f:
            for row in tqdm(range(rows), leave=False, desc="Dump iterating"):
                json.dump(data, f)
                f.write("\n")


if __name__ == "__main__":
    main()
