use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// StarkNet development network.
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(kind = "Devnet", group = "dojo.stark", version = "v1", namespaced)]
#[kube(status = "DevnetStatus", shortname = "devnet")]
pub struct DevnetSpec {
    image: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
pub struct DevnetStatus {}
