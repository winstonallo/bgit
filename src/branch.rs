use std::path::Path;

use crate::errors::{Error, git};

pub fn create(repo: impl AsRef<Path>, branch: &str) -> Result<(), Error> {
    git(repo, "branch create", &["branch", branch]).map(drop)
}

pub fn delete(repo: impl AsRef<Path>, branch: &str) -> Result<(), Error> {
    git(repo, "branch delete", &["branch", "-D", branch]).map(drop)
}
