use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum Privacy {
    #[serde(
        alias = "cloudok",
        alias = "cloud_ok",
        alias = "Public",
        alias = "public"
    )]
    CloudOk = 0,
    #[serde(
        alias = "localonly",
        alias = "local_only",
        alias = "ProjectLocal",
        alias = "projectlocal",
        alias = "project_local"
    )]
    LocalOnly = 1,
    #[serde(
        alias = "neverinject",
        alias = "never_inject",
        alias = "Private",
        alias = "private"
    )]
    NeverInject = 2,
    #[default]
    #[serde(alias = "sealed", alias = "Sealed")]
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
