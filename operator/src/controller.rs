use std::{collections::BTreeMap, sync::Arc, time::Duration};

use futures::{future::BoxFuture, FutureExt, StreamExt};
use k8s_openapi::{
    api::{self, core::v1::ServicePort},
    apimachinery::{self, pkg::apis::meta},
};
use kube::{
    api::{DeleteParams, ListParams, Patch, PatchParams, PostParams},
    runtime::{
        controller::Action,
        finalizer::{self, Event as Finalizer},
        Controller,
    },
    Api, Client, CustomResourceExt, ResourceExt,
};
use serde_json::json;
use tracing::{debug, error, info, warn};

use crate::{
    devnet::{Devnet, DevnetState, DevnetStatus},
    error::Result,
    Error,
};

static DEVNET_FINALIZER: &str = "devnets.ryogoku.stark";
static DEFAULT_IMAGE: &str = "shardlabs/starknet-devnet:latest";

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
        let name = self.name_any();
        let devnets: Api<Devnet> = Api::namespaced(ctx.client.clone(), &ns);

        debug!(state = ?self.state(), "reconcile from state");

        match self.state() {
            DevnetState::Created => {
                let deploy = self.setup_deployment(ctx.clone()).await?;
                let service = self.setup_service(ctx.clone()).await?;

                // update status
                let new_status = json!({
                    "apiVersion": "ryogoku.stark/v1",
                    "kind": "Devnet",
                    "status": DevnetStatus {
                        state: DevnetState::Running,
                    },
                });

                let pp = PatchParams::apply("ryogoku").force();
                devnets
                    .patch_status(&name, &pp, &Patch::Apply(new_status))
                    .await?;

                info!(
                    deploy = deploy.name_any(),
                    service = service.name_any(),
                    namespace = deploy.metadata.namespace,
                    "updating status to Running"
                );

                // check again in 5 minutes
                Ok(Action::requeue(Duration::from_secs(5 * 60)))
            }
            DevnetState::Running => {
                // Check deployment is still running
                self.setup_deployment(ctx.clone()).await?;

                // Check service is still running
                self.setup_service(ctx.clone()).await?;

                Ok(Action::await_change())
            }
            DevnetState::Errored => {
                // container did not start.
                todo!()
            }
        }
    }

    async fn setup_deployment(&self, ctx: Arc<Context>) -> Result<api::apps::v1::Deployment> {
        let ns = self.namespace().expect("devnet is namespaced");
        let deploys: Api<api::apps::v1::Deployment> = Api::namespaced(ctx.client.clone(), &ns);

        let existing = deploys.get_opt(&self.name_any()).await?;

        if let Some(deploy) = existing {
            info!(
                deploy = deploy.name_any(),
                namespace = deploy.metadata.namespace,
                "deployment already exists"
            );
            Ok(deploy)
        } else {
            let manifest = self.deploy_manifest();
            let pp = PostParams::default();
            let deploy = deploys.create(&pp, &manifest).await?;
            info!(
                deploy = deploy.name_any(),
                namespace = deploy.metadata.namespace,
                "deployment created"
            );
            Ok(deploy)
        }
    }

    async fn setup_service(&self, ctx: Arc<Context>) -> Result<api::core::v1::Service> {
        let ns = self.namespace().expect("devnet is namespaced");
        let services: Api<api::core::v1::Service> = Api::namespaced(ctx.client.clone(), &ns);

        let existing = services.get_opt(&self.name_any()).await?;

        if let Some(service) = existing {
            info!(
                service = service.name_any(),
                namespace = service.metadata.namespace,
                "service already exists"
            );
            Ok(service)
        } else {
            let service_manifest = self.service_manifest();
            let pp = PostParams::default();
            let service = services.create(&pp, &service_manifest).await?;
            info!(
                service = service.name_any(),
                namespace = service.metadata.namespace,
                "service created"
            );
            Ok(service)
        }
    }

    async fn cleanup(&self, ctx: Arc<Context>) -> Result<Action> {
        debug!("cleanup devnet");
        let ns = self.namespace().expect("devnet is namespaced");
        let deploys: Api<api::apps::v1::Deployment> = Api::namespaced(ctx.client.clone(), &ns);
        let dp = DeleteParams::default();
        let result = deploys.delete(&self.name_any(), &dp).await;
        if let Err(kube::Error::Api(_)) = result {
            warn!(
                deploy = self.name_any(),
                namespace = self.metadata.namespace,
                "No deployment was found to delete, assuming there is nothing to do.",
            );
        }
        else {
            info!(
                deploy = self.name_any(),
                namespace = self.metadata.namespace,
                "deployment deleted"
            );
        }
        Ok(Action::await_change())
    }

    fn deploy_manifest(&self) -> api::apps::v1::Deployment {
        use api::apps::v1::Deployment;
        let metadata = self.object_metadata();
        let spec = self.deploy_spec();

        Deployment {
            metadata,
            spec: Some(spec),
            ..Deployment::default()
        }
    }

    fn object_metadata(&self) -> meta::v1::ObjectMeta {
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
        let labels = BTreeMap::from([
            ("app.kubernetes.io/name".to_string(), self.name_any()),
            ("ryogoku.stark/devnet_name".to_string(), self.name_any())
            ]);

        ObjectMeta {
            name: self.metadata.name.clone(),
            owner_references: Some(vec![owner_ref]),
            labels: Some(labels),
            ..ObjectMeta::default()
        }
    }

    fn selector_labels(&self) -> Option<BTreeMap<String, String>> {
        Some(BTreeMap::from([
            ("ryogoku.stark/devnet_name".to_string(), self.name_any())
        ]))
    }

    fn deploy_spec(&self) -> api::apps::v1::DeploymentSpec {
        use api::core::v1::{Container, ContainerPort, PodTemplateSpec, PodSpec};
        use api::apps::v1::DeploymentSpec;
        let image = self
            .spec
            .image
            .clone()
            .unwrap_or_else(|| DEFAULT_IMAGE.to_string());

        let mut args = Vec::default();

        if self.spec.lite_mode.unwrap_or(false) {
            args.push("--lite-mode".to_string());
        }

        if self.spec.lite_mode_block_hash.unwrap_or(false) {
            args.push("--lite-mode-block-hash".to_string());
        }

        if self.spec.lite_mode_deploy_hash.unwrap_or(false) {
            args.push("--lite-mode-deploy-hash".to_string());
        }

        if let Some(accounts) = self.spec.accounts {
            args.push(format!("--accounts={}", accounts));
        }

        if let Some(initial_balance) = &self.spec.initial_balance {
            args.push(format!("--initial-balance={}", initial_balance));
        }

        if let Some(seed) = &self.spec.seed {
            args.push(format!("--seed={}", seed));
        }

        if let Some(start_time) = self.spec.start_time {
            args.push(format!("--start-time={}", start_time));
        }

        if let Some(gas_price) = &self.spec.gas_price {
            args.push(format!("--gas-price={}", gas_price));
        }

        if let Some(extra_args) = &self.spec.extra_args {
            args.extend(extra_args.clone());
        }
        
        DeploymentSpec {
            replicas: Some(1),
            selector: meta::v1::LabelSelector {
                match_labels: self.selector_labels(),
                ..meta::v1::LabelSelector::default()
            },
            template: PodTemplateSpec {
                metadata: Some(self.object_metadata()),
                spec: Some(PodSpec {
                    containers: vec![Container {
                        name: "starknet-devnet".to_string(),
                        image: Some(image),
                        args: Some(args),
                        ports: Some(vec![
                            ContainerPort {
                                container_port: 9575,
                                name: Some("rpc".to_string()),
                                ..ContainerPort::default()
                            },
                            ContainerPort {
                                container_port: 5050,
                                name: Some("gateway".to_string()),
                                ..ContainerPort::default()
                            }
                        ]),
                        ..Container::default()
                    }],
                    ..PodSpec::default()
                }),
                ..PodTemplateSpec::default()
            },
            ..DeploymentSpec::default()
        }
    }

    fn service_manifest(&self) -> api::core::v1::Service {
        use api::core::v1::Service;
        let metadata = self.object_metadata();
        let spec = self.service_spec();

        Service {
            metadata,
            spec: Some(spec),
            ..Service::default()
        }
    }

    fn service_spec(&self) -> api::core::v1::ServiceSpec {
        use api::core::v1::ServiceSpec;
        use apimachinery::pkg::util::intstr::IntOrString;

        ServiceSpec {
            selector: self.selector_labels(),
            type_: self.spec.service_type.clone(),
            ports: Some(vec![ServicePort {
                name: Some("rpc".to_string()),
                port: 9575,
                target_port: Some(IntOrString::String("rpc".to_string())),
                ..ServicePort::default()
            },
            ServicePort {
                name: Some("gateway".to_string()),
                port: 5050,
                target_port: Some(IntOrString::String("gateway".to_string())),
                ..ServicePort::default()
            }]),
            ..ServiceSpec::default()
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

    let deploys = Api::<api::apps::v1::Deployment>::all(client.clone());
    let services = Api::<api::core::v1::Service>::all(client.clone());

    let ctx = Context { client };
    let controller = Controller::new(devnets, ListParams::default())
        .owns(deploys, ListParams::default())
        .owns(services, ListParams::default())
        .run(reconcile_devnet, error_policy, ctx.into())
        .filter_map(|x| async move { std::result::Result::ok(x) })
        .for_each(|_| futures::future::ready(()))
        .boxed();

    Ok(controller)
}
