use std::path::Path;

use crate::branch;

#[derive(clap::Parser, Debug)]
pub struct CreateArgs {
    /// Target branch for the worktree.
    #[arg(long)]
    branch: String,
    /// Create `branch`.
    #[arg(long, default_value_t = false)]
    create_branch: bool,
    /// Target path where the worktrees will be written. Must be absolute.
    #[arg(long)]
    path: String,
    /// Paths to the repos for which worktrees are to be created, either
    /// absolute or relative to PWD.
    repos: Vec<String>,
}

#[derive(clap::Subcommand, Debug)]
enum Subcommand {
    Create(CreateArgs),
}

#[derive(clap::Args, Debug)]
pub struct Args {
    #[command(subcommand)]
    sub: Subcommand,
}

#[derive(thiserror::Error, Debug)]
pub enum CreateError {
    #[error("target path {path} is not an absolute path")]
    TargetPathNotAbsolute { path: String },
    #[error("could not create target directory {path}: {error}")]
    TargetPathCreation { path: String, error: std::io::Error },
    #[error("target path is not a directory")]
    TargetPathNotDir { path: String },
    #[error("could not create branch: {0}")]
    CouldNotCreateBranch(#[from] branch::Error),
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("creation failed at {target} for {repo}: {error}")]
    CouldNotCreateWorktree { target: String, repo: String, error: String },
}

#[derive(thiserror::Error, Debug)]
pub enum RemoveError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("removal failed at {target} for {repo}: {error}")]
    CouldNotRemoveWorktree { target: String, repo: String, error: String },
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("could not create worktree: {0}")]
    Create(#[from] CreateError),
    #[error("could not remove worktree: {0}")]
    Remove(#[from] RemoveError),
}

fn create_one(repo: &str, target: &str, branch: &str) -> Result<(), CreateError> {
    let output = std::process::Command::new("git")
        .args(["worktree", "add", target, branch])
        .current_dir(repo)
        .output()?;

    if !output.status.success() {
        return Err(CreateError::CouldNotCreateWorktree {
            target: target.into(),
            repo: repo.into(),
            error: String::from_utf8_lossy(&output.stderr).into(),
        });
    }

    Ok(())
}

fn create_inner(info: &CreateArgs) -> Result<(), CreateError> {
    let path = Path::new(&info.path);
    if !path.is_absolute() {
        return Err(CreateError::TargetPathNotAbsolute { path: info.path.clone() });
    }
    if !path.exists() {
        std::fs::create_dir_all(path).map_err(|e| CreateError::TargetPathCreation {
            path: info.path.clone(),
            error: e,
        })?;
    }
    if !path.is_dir() {
        return Err(CreateError::TargetPathNotDir { path: info.path.clone() });
    }

    if info.create_branch {
        let mut created = vec![];
        for r in &info.repos {
            if let Err(e) = branch::create(r, &info.branch) {
                for c in created {
                    _ = branch::delete(c, &info.branch);
                }
                return Err(CreateError::CouldNotCreateBranch(e));
            }
            created.push(r);
        }
    }

    let mut created = vec![];
    for r in &info.repos {
        if let Err(e) = create_one(r, &format!("{}/{r}", info.path), &info.branch) {
            for r in created {
                _ = remove_one(&info.path, r);
            }
            return Err(e);
        }
        created.push(r);
    }

    Ok(())
}

fn create(info: &CreateArgs) -> Result<(), Error> {
    Ok(create_inner(info)?)
}

fn remove_one(path: &str, repo: &str) -> Result<(), RemoveError> {
    let output = std::process::Command::new("git")
        .args(["worktree", "remove", path])
        .current_dir(repo)
        .output()?;

    if !output.status.success() {
        return Err(RemoveError::CouldNotRemoveWorktree {
            target: path.into(),
            repo: repo.into(),
            error: String::from_utf8_lossy(&output.stderr).into(),
        });
    }

    Ok(())
}

pub fn worktree(info: &Args) -> Result<(), Error> {
    match &info.sub {
        Subcommand::Create(c) => create(c),
    }
}
