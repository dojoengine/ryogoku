use std::fmt::Display;

use kube::{core::object::HasStatus, CustomResource};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// StarkNet development network.
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema, Default)]
#[kube(kind = "Devnet", group = "ryogoku.stark", version = "v1", namespaced)]
#[kube(status = "DevnetStatus", shortname = "devnet")]
pub struct DevnetSpec {
    /// The devnet docker image and version. Defaults to `shardlabs/starknet-devnet:latest`.
    pub image: Option<String>,
    /// Applies all `lite-mode-*` optimizations by disabling some features.
    pub lite_mode: Option<bool>,
    /// Disables block hash calculation.
    pub lite_mode_block_hash: Option<bool>,
    /// Disable deploy tx hash calculation.
    pub lite_mode_deploy_hash: Option<bool>,
    /// Specify the number of accounts to be pre-deployed.
    pub accounts: Option<usize>,
    /// Specify the initial balance of pre-deployed accounts.
    pub initial_balance: Option<String>,
    /// Specify the pre-deployed accounts randomness seed.
    pub seed: Option<String>,
    /// Specify the start time of the genesis block in Unix time seconds.
    pub start_time: Option<u64>,
    /// Specify the gas price in wei.
    pub gas_price: Option<String>,
    /// Extra arguments for the container.
    pub extra_args: Option<Vec<String>>,
    /// Specify how the service is exposed.
    pub service_type: Option<String>,
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

impl Display for DevnetState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DevnetState::Created => write!(f, "CREATED"),
            DevnetState::Running => write!(f, "RUNNING"),
            DevnetState::Errored => write!(f, "ERRORED"),
        }
    }
}
