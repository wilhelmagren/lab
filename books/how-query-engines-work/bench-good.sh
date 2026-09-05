#!/usr/bin/env bash

hyperfine --warmup 1 --runs 5 '.venv/bin/python 1brc_duckdb.py' '.venv/bin/python 1brc_polars.py'
