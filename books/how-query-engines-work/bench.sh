#!/usr/bin/env bash

hyperfine --warmup 1 --runs 5 './target/release/tqe' '.venv/bin/python verify.py'
