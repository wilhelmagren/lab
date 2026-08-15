#!/usr/bin/env bash

set -eou pipefail

docker run -d -e POSTGRES_PASSWORD=123lol -p 5432:5432 postgres
