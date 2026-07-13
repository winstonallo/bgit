use std::path::Path;

use crate::branch;
use crate::errors::{Error, git};

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

fn create_one(repo: &str, path: &str, branch: &str) -> Result<(), Error> {
    git(repo, "worktree add", &["worktree", "add", path, branch]).map(drop)
}

fn remove_one(repo: &str, path: &str) -> Result<(), Error> {
    git(repo, "worktree remove", &["worktree", "remove", path]).map(drop)
}

fn create(info: &CreateArgs) -> Result<(), Error> {
    let path = Path::new(&info.path);
    if !path.is_absolute() {
        return Err(Error::NotAbsolute(info.path.clone()));
    }
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    if !path.is_dir() {
        return Err(Error::NotDir(info.path.clone()));
    }

    if info.create_branch {
        let mut created = vec![];
        for r in &info.repos {
            if let Err(e) = branch::create(r, &info.branch) {
                for c in created {
                    _ = branch::delete(c, &info.branch);
                }
                return Err(e);
            }
            created.push(r);
        }
    }

    let mut created = vec![];
    for r in &info.repos {
        let target = format!("{}/{r}", info.path);
        if let Err(e) = create_one(r, &target, &info.branch) {
            for r in created {
                _ = remove_one(r, &format!("{}/{r}", info.path));
            }
            return Err(e);
        }
        created.push(r);
    }

    Ok(())
}

pub fn worktree(info: &Args) -> Result<(), Error> {
    match &info.sub {
        Subcommand::Create(c) => create(c),
    }
}
