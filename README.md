# Ryogoku


## Ryogoku Operator

The project uses a Kubernetes Operator to schedule all the different components.

The first step is to install the Custom Resource Definition (CRD) into your cluster.

You can do that using the `ryogoku` cli tool, in two different ways.

 - Print the CRD to stdout and use `kubectl apply`

```txt
$ ryogoku crd print | kubectl apply -f -
```

 - Install directly from the cli.

```txt
$ ryogoku crd install
```

Then simply running the `ryogoku-operator` binary in the cluster operates it.

## Hacking


### Development environment setup

Start by creating a local k8s cluster, for example using [minikube](https://minikube.sigs.k8s.io/docs/start/).
Minikube will automatically change the current `kubectl` context.

```txt
$ minikube start
```

Next check you're ready to start developing:

```txt
$ kubectl cluster-info

Kubernetes control plane is running at https://127.0.0.1:34461
CoreDNS is running at https://127.0.0.1:34461/api/v1/namespaces/kube-system/services/kube-dns:dns/proxy

To further debug and diagnose cluster problems, use 'kubectl cluster-info dump'.
```

To expose the devnet, simply create a devnet using `--expose`, and the use minikube to Tunnel:

```txt
$ minikube service [devnet-name]
```
