use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum Privacy {
    #[serde(alias = "cloudok", alias = "cloud_ok")]
    CloudOk = 0,
    #[serde(alias = "localonly", alias = "local_only")]
    LocalOnly = 1,
    #[serde(alias = "neverinject", alias = "never_inject")]
    NeverInject = 2,
    #[default]
    #[serde(alias = "sealed")]
    Sealed = 3,
}

impl Privacy {
    pub fn combine(&self, other: Privacy) -> Privacy {
        if *self > other {
            *self
        } else {
            other
        }
    }
}
