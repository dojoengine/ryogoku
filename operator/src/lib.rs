pub mod controller;
mod devnet;
mod error;

pub use self::{
    devnet::{Devnet, DevnetSpec, DevnetStatus},
    error::{Error, Result},
};

pub mod kube {
    pub use kube::*;
}

pub mod k8s_openapi {
    pub use k8s_openapi::*;
}
