# Kubernetes

**Kubernetes**, also known as K8s, is an open source system for automating deployment,
scaling, and management of containerized applications. It groups containers that make up
an application into logical units for easy management and discovery.

Kubernetes ships pre-built binaries for each component as well as a standard set of client
applications to bootstrap or interact with a cluster. All Kubernetes container images are
deployed to the `registry.k8s.io` container image registry.

It is recommended to run Kubernetes components as container images wherever that is
possible, and to have Kubernetes manage those components.

The central component of Kubernetes is a cluster, which is itself made up of multiple
physical or virtual machines. Each cluster component performs a specific function as
either a master or a worker - a master controls and manages the containers in the nodes,
while a worker hosts the groups of one or more containers.

![Kubernetes cluster example](https://www.armosec.io/wp-content/uploads/2022/01/Picture1.png)

In the figure above you can see a feature-rich cluster that contains six components,
namely, an API server, scheduler, controller, **etcd**, **kubelet**, and **kube-proxy**.
The first four components run on the master, whereas the rest of the functions run on the
worker.


## minikube and kubeadm

When it comes to designing a cluster, minikube and kubeadm are the predominant solutions.
Designed for learning, development and testing, **minikube** is a fast and simple solution
for deploying a single node cluster.

![Minikube architecture](https://www.armosec.io/wp-content/uploads/2022/01/Picture2.png)


**kubeadm** builds a minimum viable, production-ready Kubernetes cluster that fonforms to
best practices. It also allows us to choose the container runtime, though it had Docker by
default. This solution requires a minimum of two VMs to run the master and worker.

![Kubeadm architecture](https://www.armosec.io/wp-content/uploads/2022/01/Picture3.png)


## Standalone Kubernetes Cluster: minikube (with Docker)

minikube is the local Kubernetes cluster used primarily for learning and exploration of
Kubernetes. Start the minikube installation:

```
curl -LO https://github.com/kubernetes/minikube/releases/latest/download/minikube-darwin-arm64
sudo install minikube-darwin-arm64 /usr/local/bin/minikube
```

Verify the installed version and then start the local cluster with:

```
minikube version
...

minikube start --driver=docker
```

Now you can see the minikube container in your Docker desktop instance. To simplify using
kubectl you can create an alias:

```
alias kubectl="minikube kubectl --"

kubectl get pods -A
...
```

To get the cluster node details, use:

```
kubectl get nodes

NAME       STATUS   ROLES           AGE     VERSION
minikube   Ready    control-plane   2m25s   v1.32.0
```

To run the Kubernetes dashboard you run:

```
minikube dashboard
```

this command enables the dashboard add-on and opens the proxy in the default web browser.
You can create Kubernetes resources directly in the dashboard such as Deployment and
Service.


## Deploying apps to Kubernetes

A Kubernetes **pod** is a group of one or more Containers, tied together for the purposes
of administration and networking. The pod in this tutorial has only one container. A
Kubernetes **Deployment** checks on the health of your pod and restarts the pod's
container if it terminates. Deployments are the recommended way to manage the creation and
scaling of pods.

Use the `kubectl create` command to create a Deployment that manages a pod. The pod runs a
container based on the provided image:

```
kubectl create deployment hello-node --image=registry.k8s.io/e2e-test-images/agnhost:2.39 -- /agnhost netexec --http-port=8080
```

View the deployments:

```
kubectl get deployments

NAME         READY   UP-TO-DATE   AVAILABLE   AGE
hello-node   1/1     1            1           13s
```

View the pod:

```
kubectl get pods

NAME                         READY   STATUS    RESTARTS   AGE
hello-node-c74958b5d-rzjr6   1/1     Running   0          58s


# view cluster events
kubectl get events

# view the kubectl configuration
kubectl config view

# view applications logs
kubectl logs hello-node-c74958b5d-rzjr6

I0311 08:50:25.285112       1 log.go:195] Started HTTP server on port 8080
I0311 08:50:25.285319       1 log.go:195] Started UDP server on port  8081
```


## kubectl

The Kubernetes command-line tool, **kubectl**, allows you to run commands against
Kubernetes clusters. You can use kubectl to deploy applications, inspect and manage
cluster resources, and view logs.


