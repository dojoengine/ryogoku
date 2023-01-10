mod controller;
mod devnet;

pub use self::devnet::{Devnet, DevnetSpec, DevnetStatus};
pub mod kube {
    pub use kube::*;
}

pub mod k8s_openapi {
    pub use k8s_openapi::*;
}
