use kube::{core::object::HasStatus, CustomResource};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// StarkNet development network.
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema, Default)]
#[kube(kind = "Devnet", group = "ryogoku.stark", version = "v1", namespaced)]
#[kube(status = "DevnetStatus", shortname = "devnet")]
pub struct DevnetSpec {
    image: Option<String>,
}

/// State of the devnet.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, JsonSchema, PartialEq)]
pub enum DevnetState {
    /// Devnet created.
    Created,
    /// Devnet is running.
    Running,
    /// Devnet started but errored.
    Errored,
}

/// Devnet status.
#[derive(Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
pub struct DevnetStatus {
    /// Devnet state.
    state: DevnetState,
}

impl Default for DevnetState {
    fn default() -> Self {
        DevnetState::Created
    }
}

impl Devnet {
    pub fn state(&self) -> DevnetState {
        self.status().map(|s| s.state).unwrap_or_default()
    }
}
