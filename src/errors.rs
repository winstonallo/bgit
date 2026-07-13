use std::path::Path;
use std::process::Command;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("git {op} failed in {repo}: {stderr}")]
    Git { op: String, repo: String, stderr: String },
    #[error("path is not absolute: {0}")]
    NotAbsolute(String),
    #[error("path is not a directory: {0}")]
    NotDir(String),
}

/// Run git in `repo`, returning stdout. Non-zero exit → Error::Git with stderr.
pub fn git(repo: impl AsRef<Path>, op: &str, args: &[&str]) -> Result<String, Error> {
    let repo = repo.as_ref();
    let out = Command::new("git").args(args).current_dir(repo).output()?;
    if !out.status.success() {
        return Err(Error::Git {
            op: op.into(),
            repo: repo.to_string_lossy().into(),
            stderr: String::from_utf8_lossy(&out.stderr).trim().into(),
        });
    }
    Ok(String::from_utf8_lossy(&out.stdout).into())
}
