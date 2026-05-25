use crate::command::run_git;
use crate::errors::Result;
use sha2::{Digest, Sha256};
use std::path::Path;

pub fn read_remote_url_hash(root: &Path) -> Result<Option<String>> {
    match run_git(root, &["config", "--get", "remote.origin.url"]) {
        Ok(Some(url)) => Ok(Some(hash_remote_url(&url))),
        Ok(None) => Ok(None),
        Err(_) => Ok(None),
    }
}

pub fn hash_remote_url(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    hex::encode(hasher.finalize())
}
