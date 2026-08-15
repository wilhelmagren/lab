"""
Simple script to test system metrics logging with MLflow.

File created: 2025-03-06
Last updated: 2025-03-06
"""

import mlflow
import time


if __name__ == "__main__":
    mlflow.set_tracking_uri("http://localhost:5000")
    mlflow.set_experiment("system-metrics-test")

    with mlflow.start_run(log_system_metrics=True):
        time.sleep(10)
