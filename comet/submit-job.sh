#!/usr/bin/env bash

set -eou pipefail

uv run spark-submit \
  --jars $COMET_JAR \
  --conf spark.driver.extraClassPath=$COMET_JAR \
  --conf spark.executor.extraClassPath=$COMET_JAR \
  --conf spark.plugins=org.apache.spark.CometPlugin \
  --conf spark.shuffle.manager=org.apache.spark.sql.comet.execution.shuffle.CometShuffleManager \
  --conf spark.comet.explainFallback.enabled=true \
  --conf spark.memory.offHeap.enabled=true \
  --conf spark.memory.offHeap.size=16g \
  $1
