use std::{path::Path, string::FromUtf8Error};

#[derive(clap::Parser, Debug)]
pub struct Args {
    #[arg(long, default_value_t = String::from("origin/master"))]
    onto: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("could not rebase {repo} onto {onto}: {error}")]
    CouldNotRebase { repo: String, onto: String, error: String },
    #[error("utf-8 decoding error (this should never happen): {0}")]
    Utf8Error(#[from] FromUtf8Error),
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

    let output = std::process::Command::new("git").args(["rebase", onto]).current_dir(path).output()?;
    if !output.status.success() {
        return Err(Error::CouldNotRebase {
            repo: path.to_string_lossy().into(),
            onto: onto.into(),
            error: String::from_utf8(output.stderr)?,
        });
    }

    tracing::info!("git > {}", String::from_utf8(output.stdout)?.trim());

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
