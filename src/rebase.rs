use std::path::Path;

use crate::errors::{Error, git};

#[derive(clap::Parser, Debug)]
pub struct Args {
    #[arg(long, default_value_t = String::from("origin/master"))]
    onto: String,
}

fn try_rebase<P: AsRef<Path>>(onto: &str, path: P) -> Result<bool, Error> {
    let path = path.as_ref();
    if !path.is_dir() {
        tracing::info!("skipping entry {path:?}: not a directory");
        return Ok(false);
    }
    if !path.join(".git").exists() {
        tracing::info!("skipping entry {path:?}: not a git repository");
        return Ok(false);
    }

    let out = git(path, "rebase", &["rebase", onto])?;
    tracing::info!("git > {}", out.trim());
    Ok(true)
}

pub fn rebase(info: &Args) -> Result<(), Error> {
    for entry in std::fs::read_dir(".")? {
        let path = &entry?.path();

        match try_rebase(&info.onto, path) {
            Err(e) => tracing::error!("{e}"),
            Ok(true) => tracing::info!("rebased {path:?} onto {}", &info.onto),
            Ok(false) => {}
        }
    }
    Ok(())
}
