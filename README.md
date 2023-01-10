# DoJo


## Hacking

### Project structure

```ml
manifests/
├── crds
│   └── dojo.yml: "dojo custom resource"
├── operator.yml: "operator deployment"
└── rbac.yml: "operator permission configuration"
src/
└── main.ts: "k8s operator entry point"
```


### Development environment setup

Start by creating a local k8s cluster, for example using [kind](https://kind.sigs.k8s.io/docs/user/quick-start/).
In this case, we save the kube configuration file to `kube`.

```txt
$ kind cluster create --kubeconfig kube

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

### Dojo Custom Resource Definition (CRD)

The `Dojo` CRD is defined in `manifests/crds/dojo.yml`. The development workflow is usually:

- Delete any existing `dojo` resource. You can get a list with `kubectl --kubeconfig kube get dojo -A`.
- Delete the existing dojo crd with `kubectl --kubeconfig kube delete crd dojos.dojo-on-chain.com`.
- Apply the new crd with `kubectl --kubeconfig kube apply -f manifists/crds/dojo.yml`.


### Dojo Operator

TODO