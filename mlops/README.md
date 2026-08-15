# MLOps

**MLOps** is a paradigm that aims to deploy and maintain machine learning models in
production reliably and efficiently. It revolves around aspects like best practices, sets
of concepts, as well as a development culture when it comes to end-to-end
conceptualization, implementation, monitoring, deployment, and scalability of machine
learning products. Most of all, it is an engineering practice that leverages three
contributing discplines: machine learning, software engineering, and data engineering.


## MLflow

MLflow is an open-source platform, purpose-built to assist machine learning practitioners
and teams in handling the complexities of the machine learning process. MLflow focuses on
the full lifecycle for machine learning projects, ensuring that each phase is manageable,
traceable, and reproducible.

Install MLlow from PyPI:

```
py -m pip install mlflow
```

MLflow UI is a web-based interface that allows users to visualize and manage their machine
learning experiments. The MLflow Server is a backend component that hosts the MLflow
Tracking Service. It can be set up on a local machine or a remote server, providing a
centralized repository for experiment data. Its capabilities include:

- **Remote access:** share experiments and collaborate with team members across different
  environments.
- **Scalability:** handle a larger volume of experiments and runs, suitable for
  enterprise-level deployment.
- **Storage flexibility:** configure backend and artifact storage to use local
  filesystems, cloud storage, or database systems.

You can use either of `mlflow ui` or `mlflow server` when running locally (individually),
whereas `mlflow server` is to be used if you are planning to have a dedicated host serving
an entire team.

### MLflow Tracking Service

To start the MLflow Tracking Service you can run the following command:

```
mlflow server --host 127.0.0.1 --port 8080
```

and once the server starts running you should see something like the following:

```
[2023-11-01 10:28:12 +0900] [28550] [INFO] Starting gunicorn 20.1.0
[2023-11-01 10:28:12 +0900] [28550] [INFO] Listening at: http://127.0.0.1:8080 (28550)
[2023-11-01 10:28:12 +0900] [28550] [INFO] Using worker: sync
[2023-11-01 10:28:12 +0900] [28552] [INFO] Booting worker with pid: 28552
[2023-11-01 10:28:12 +0900] [28553] [INFO] Booting worker with pid: 28553
[2023-11-01 10:28:12 +0900] [28555] [INFO] Booting worker with pid: 28555
[2023-11-01 10:28:12 +0900] [28558] [INFO] Booting worker with pid: 28558
...
```

> [!IMPORTANT]
> The server listens on `http://localhost:5000` by default and only accepts connections
> from the local machine. To let the server accept connections from other machines, you
> will need to pass `--host 0.0.0.0` to listen on all network interfaces (or a specific
> interface address). This is typically required when running the server in a Kubernetes
> pod or a Docker container.
>
> Note that doing this for a server running on a public network is not recommended for
> security reasons, and you should consider using a reverse proxy like NGINX.


Once you started the tracking server, you can connect your local clients by either:

- setting the `MLFLOW_TRACKING_URI` environment variable to the hosts URI along with its
  scheme and port (e.g. `http://10.0.0.1:5000`)
- or by calling `mlflow.set_tracking_uri(<your-host-uri>)` from your Python code.

The `mlflow.start_run()`, `mlflow.log_param()`, and `mlflow.log_metric()` calls then
simply makes API requests to your remote tracking server.


### Configuring the Tracking Server

By default, the tracking server logs runs metadata to the local filesystem under the
`./mlruns` directory. You can configure a different backend store by adding the
`--backend-store-uri` option with e.g. value as `sqlite:///my.db` which will create a new
SQLite database `my.db` in the current directory, and any logging requests from the
clients will be directed to this database.

The `--host` option exposes the service on all interfaces. If you are running a server in
production, we would recommend not exposing the built-in server broadly (as it is
unauthenticated and unencrypted), and instead put it behin a reverse proxy like NGINX.

You can then pass authentication headers to MLflow using the following environment
variables:

- `MLFLOW_TRACKING_USERNAME`: username to use with basic HTTP authentication,
- `MLFLOW_TRACKING_PASSWORD`: password to use with basic HTTP authentication,
- `MLFLOW_TRACKING_TOKEN`: token to use with HTTP Bearer authentication,
- `MLFLOW_TRACKING_INSECURE_TLS`: if set to the literal `true`, then MLflow does not
  verify the TLS connection,
- `MLFLOW_TRACKING_SERVER_CERT_PATH`: path to a CA bundle to use,
- `MLFLOW_TRACKING_CLIENT_CERT_PATH`: path to the ssl clients cert file (.pem).

You can verify the version of the MLflow Tracking Service by querying the `/version`
endpoint, and to verify that your MLflow client has the same version as the server you can
run the following:

```python
import mlflow
import requests

response = requests.get("http://<mlflow-host>:<mlflow-port>/version")
assert response.text == mlflow.__version__

```


### MLflow System Metrics

MLflow allows users to log system metrics including CPU stats, GPU stats, memory usage,
network traffic, and disk usage during execution of an MLflow run. To log metrics, install
`psutil` (for CPU) and `pynvml` (for GPU) with:

```
py -m pip install psutil
py -m pip install pynvml
```

There are three ways to enable or disable system metrics logging:

- set the environment variable `MLFLOW_ENABLE_SYSTEM_METRICS_LOGGING` to either `true` or
  `false`,
- use the `mlflow.enable_system_metrics_logging()` method to enable logging and the
  `mlflow.disable_system_metrics_logging()` to disable logging of system metrics,
- use the `log_system_metrics` parameter in the `mlflow.start_run()` method to control
  system metrics logging for the **current** MLflow run.

You can view the system metrics within the MLflow UI that your Tracking Server exposes in
the same place as you would see your ML metrics.


### MLflow Model and Deployment

**MLflow Model** is a standard format that packages a machine learning model with its
metadata, such as dependencies and inference schema. You typically create a model as a
result of a training execution using the Tracking APIs. To use MLflow deployment, you must
first create a model.

MLflow uses Docker containers to package models with their dependencies, enabling
deployment to various destinations without environment compatibility issues. 

MLflow allows you to deploy your model locally using the cli. But before deploying, you
must have an MLflow Model, when you have that you can use the following command to deploy
your model for inference:

```
mlflow models serve -m runs:/<run-id>/model -p 5000
```

which starts a local server that listens on the specified port and serves your model. You
can then invoke your model for inference with:

```
curl http://127.0.0.1:5000/invocations -H "Content-Type:application/json"  --data '{"inputs": [[1, 2], [3, 4], [5, 6]]}'
```

The inference server provides 4 endpoints:

- `/invocations`: an inference endpoint that accepts `POST` requests with input data and
  returns predictions,
- `/ping`: used for health checks,
- `/health`: same as `/ping`,
- `/version`: returns the MLflow version running on the inference server.

The `/invocations` endpoint accepts either CSV or JSON input and the input format must be
specified in the `Content-Type` header as either `application/csv` or `application/json`.

MLflow uses FastAPI by default for serving the model on the inference server.

You can execute a single batch inference job on local files using the following command:

```
mlflow models predict -m runs:/<run-id>/model -i input.csv -o output.csv
```

To automatically register your model in the Model Registry you need to add the following
fields to your `.log_model()` code:

```python
...

mlflow.sklearn.log_model(
    sk_model=model,
    artifact_path="sklearn-model",
    signature=signature,
    registered_model_name="sk-learn-random-forest-reg-model",
)

```

or you can register the model **after** the experiment is done with:

```python
result = mlflow.register_model(
    "runs:/abcdefghijklmnopqrstuvwxyz1234567890/sklearn-model",
    "sk-learn-random-forest-reg-model",
)

```
