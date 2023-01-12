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

## Hacking


### Development environment setup

Start by creating a local k8s cluster, for example using [kind](https://kind.sigs.k8s.io/docs/user/quick-start/).
In this case, we save the kube configuration file to `kube`.

```txt
$ kind create cluster --kubeconfig kube

Creating cluster "kind" ...
 ✓ Ensuring node image (kindest/node:v1.25.3) 🖼
 ✓ Preparing nodes 📦
 ✓ Writing configuration 📜
 ✓ Starting control-plane 🕹️
 ✓ Installing CNI 🔌
 ✓ Installing StorageClass 💾
Set kubectl context to "kind-kind"
You can now use your cluster with:

kubectl cluster-info --context kind-kind --kubeconfig kube

Not sure what to do next? 😅  Check out https://kind.sigs.k8s.io/docs/user/quick-start/
```

Next check you're ready to start developing:

```txt
$ kubectl --kubeconfig kube cluster-info

Kubernetes control plane is running at https://127.0.0.1:34461
CoreDNS is running at https://127.0.0.1:34461/api/v1/namespaces/kube-system/services/kube-dns:dns/proxy

To further debug and diagnose cluster problems, use 'kubectl cluster-info dump'.
```

You can set the `KUBECONFIG` env variable to automatically use the local cluster:

```txt
$ export KUBECONFIG=$(pwd)/kube
$ kubectl cluster-info
```