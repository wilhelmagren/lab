#!/usr/bin/env bash

set -eou pipefail

git clone https://github.com/apache/datafusion-comet.git
cd datafusion-comet

docker build -t datafusion-comet:latest -f kube/Dockerfile .
