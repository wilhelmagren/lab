#!/usr/bin/env bash

set -eou pipefail

git clone https://github.com/apache/datafusion-comet.git && cd datafusion-comet
make release PROFILES="-Pspark-3.5.5 -Pscala-2.12.18"

