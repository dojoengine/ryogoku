use std::borrow::Cow;

use chrono::{Duration, Utc};

use ryogoku_operator::{
    k8s_openapi::apimachinery::pkg::apis::meta::v1::Time, kube::ResourceExt, Devnet,
};
use tabled::Tabled;

/// Wrapper around [Devnet], used to implement [Tabled].
pub struct DevnetOut(Devnet);

impl DevnetOut {
    pub fn new(devnet: Devnet) -> Self {
        DevnetOut(devnet)
    }
}

impl Tabled for DevnetOut {
    const LENGTH: usize = 4;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        let inner = &self.0;
        let namespace = inner.namespace().expect("devnet is namespaced");
        let name = inner.name_any();
        let state = format!("{}", inner.state());
        let age = inner
            .metadata
            .creation_timestamp
            .as_ref()
            .map(|t| time_ago(t).to_human())
            .unwrap_or_default();

        vec![
            Cow::Owned(namespace),
            Cow::Owned(name),
            Cow::Owned(state),
            Cow::Owned(age),
        ]
    }

    fn headers() -> Vec<Cow<'static, str>> {
        vec![
            Cow::Owned("NAMESPACE".to_string()),
            Cow::Owned("NAME".to_string()),
            Cow::Owned("STATE".to_string()),
            Cow::Owned("AGE".to_string()),
        ]
    }
}

fn time_ago(time: &Time) -> Duration {
    let now = Utc::now();
    now - time.0
}

pub trait HumanReadable {
    fn to_human(&self) -> String;
}

impl HumanReadable for Duration {
    fn to_human(&self) -> String {
        if self.num_days() > 0 {
            format!("{}d", self.num_days())
        } else if self.num_hours() > 0 {
            format!("{}h", self.num_hours())
        } else if self.num_minutes() > 0 {
            format!("{}m", self.num_minutes())
        } else {
            format!("{}s", self.num_seconds())
        }
    }
}
