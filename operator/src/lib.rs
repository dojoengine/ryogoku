mod controller;
mod devnet;

pub use self::devnet::{Devnet, DevnetSpec, DevnetStatus};
pub use kube::{CustomResource, CustomResourceExt};
