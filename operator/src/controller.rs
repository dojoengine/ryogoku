use std::{sync::Arc, time::Duration};

use futures::{future::BoxFuture, FutureExt, StreamExt};
use k8s_openapi::{
    api,
    apimachinery::{self, pkg::apis::meta},
};
use kube::{
    api::{ListParams, PostParams},
    runtime::{
        controller::Action,
        finalizer::{self, Event as Finalizer},
        Controller,
    },
    Api, Client, CustomResourceExt, ResourceExt,
};
use tracing::{debug, error, info, warn};

use crate::{
    devnet::{Devnet, DevnetState},
    error::Result,
    Error,
};

static DEVNET_FINALIZER: &str = "devnets.ryogoku.stark";

/// Reconciler context.
#[derive(Clone)]
pub struct Context {
    /// kube client.
    pub client: Client,
}

/// Reconcile devnet state.
async fn reconcile_devnet(devnet: Arc<Devnet>, ctx: Arc<Context>) -> Result<Action> {
    let ns = devnet.namespace().expect("devnet is namespaced");
    let devnets: Api<Devnet> = Api::namespaced(ctx.client.clone(), &ns);

    info!(
        devnet = %devnet.name_any(),
        namespace = %ns,
        "reconcile devnet"
    );

    finalizer::finalizer(&devnets, DEVNET_FINALIZER, devnet, |event| async {
        match event {
            Finalizer::Apply(devnet) => devnet.reconcile(ctx.clone()).await,
            Finalizer::Cleanup(devnet) => devnet.cleanup(ctx.clone()).await,
        }
    })
    .await
    .map_err(|err| Error::Finalizer(Box::new(err)))
}

impl Devnet {
    async fn reconcile(&self, ctx: Arc<Context>) -> Result<Action> {
        let ns = self.namespace().expect("devnet is namespaced");
        let _name = self.name_any();
        let _devnets: Api<Devnet> = Api::namespaced(ctx.client.clone(), &ns);
        let pods: Api<api::core::v1::Pod> = Api::namespaced(ctx.client.clone(), &ns);

        debug!(state = ?self.state(), "reconcile from state");

        match self.state() {
            DevnetState::Created => {
                // create pod
                let pod_manifest = self.pod_manifest();
                let pp = PostParams::default();
                let pod = pods.create(&pp, &pod_manifest).await?;
                info!(pod = ?pod, "pod created");
                // check again in 5 minutes
                Ok(Action::requeue(Duration::from_secs(5 * 60)))
            }
            DevnetState::Running => {
                // nothing to do?
                // maybe can update image
                todo!()
            }
            DevnetState::Errored => {
                // container did not start.
                todo!()
            }
        }
    }

    async fn cleanup(&self, _ctx: Arc<Context>) -> Result<Action> {
        // delete pod
        todo!()
    }

    fn pod_manifest(&self) -> api::core::v1::Pod {
        use api::core::v1::Pod;
        let metadata = self.pod_metadata();
        let spec = self.pod_spec();

        Pod {
            metadata,
            spec: Some(spec),
            ..Pod::default()
        }
    }

    fn pod_metadata(&self) -> meta::v1::ObjectMeta {
        use apimachinery::pkg::apis::meta::v1::OwnerReference;
        use meta::v1::ObjectMeta;
        let api_resource = Devnet::api_resource();
        let owner_ref = OwnerReference {
            api_version: api_resource.api_version,
            kind: api_resource.kind,
            name: self.name_any(),
            uid: self.uid().expect("devnet has uid"),
            block_owner_deletion: Some(true),
            controller: Some(true),
        };
        ObjectMeta {
            name: self.metadata.name.clone(),
            owner_references: Some(vec![owner_ref]),
            ..ObjectMeta::default()
        }
    }

    fn pod_spec(&self) -> api::core::v1::PodSpec {
        use api::core::v1::{Container, ContainerPort, PodSpec};
        PodSpec {
            containers: vec![Container {
                // TODO: image should be configurable
                name: "starknet-devnet".to_string(),
                image: Some("shardlabs/starknet-devnet:latest".to_string()),
                ports: Some(vec![ContainerPort {
                    container_port: 9575,
                    name: Some("rpc".to_string()),
                    ..ContainerPort::default()
                }]),
                ..Container::default()
            }],
            ..PodSpec::default()
        }
    }
}

fn error_policy(_devnet: Arc<Devnet>, error: &Error, _ctx: Arc<Context>) -> Action {
    warn!(error = ?error, "reconcile failed");
    Action::requeue(Duration::from_secs(10))
}

/// Start the controller.
pub async fn init(client: Client) -> Result<BoxFuture<'static, ()>> {
    let devnets = Api::<Devnet>::all(client.clone());

    if devnets.list(&ListParams::default()).await.is_err() {
        error!("devnet CRD is not queryable.");
        info!("install CRD with ryogoku crd install");
        return Err(Error::CrdNotInstalled);
    }

    info!("starting operator");

    let ctx = Context { client };
    let controller = Controller::new(devnets, ListParams::default())
        .run(reconcile_devnet, error_policy, ctx.into())
        .filter_map(|x| async move { std::result::Result::ok(x) })
        .for_each(|_| futures::future::ready(()))
        .boxed();

    Ok(controller)
}
