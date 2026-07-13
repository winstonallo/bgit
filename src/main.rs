use std::path::Path;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
struct Rebase {
    #[arg(long, default_value_t = String::from("origin/master"))]
    onto: String,
}

#[derive(Parser, Debug)]
struct WorktreeCreate {
    #[arg(long)]
    branch: String,
    #[arg(long, default_value_t = false)]
    create: bool,
    /// Must be absolute
    #[arg(long)]
    path: String,
    /// Paths to the repos for which worktrees are to be created, either absolute or relative to pwd.
    repos: Vec<String>,
}

#[derive(Subcommand, Debug)]
enum Worktree {
    Create(WorktreeCreate),
}

#[derive(clap::Args, Debug)]
struct WorktreeArgs {
    #[command(subcommand)]
    command: Worktree,
}

#[derive(Subcommand, Debug)]
enum Command {
    Rebase(Rebase),
    Worktree(WorktreeArgs),
}

#[derive(Parser, Debug)]
struct Options {
    #[command(subcommand)]
    command: Command,
    #[arg(long, default_value_t = String::from("."))]
    dir: String,
    #[arg(long, default_value_t = false)]
    verbose: bool,
}

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("could not rebase repository '{repository}': {error}")]
    Rebase { repository: String, error: String },
    #[error("I/O error: {error}")]
    IO {
        #[from]
        error: std::io::Error,
    },
    #[error("decoding error: {0}")]
    Decoding(#[from] std::string::FromUtf8Error),
    #[error("worktree error: {0}")]
    Worktree(Box<Error>),
    #[error("could not create worktree: {error}")]
    WorktreeCreate { repo: String, branch: String, error: String },
    #[error("could not remove worktree: {error}")]
    WorktreeRemove { repo: String, error: String },
    #[error("could not create branch '{branch}' for repo '{repo}': {error}")]
    CreateBranch { repo: String, branch: String, error: String },
}

fn create_branch<P: AsRef<Path>>(repo: P, branch_name: &str) -> Result<(), Error> {
    let output = std::process::Command::new("git").args(["branch", branch_name]).current_dir(&repo).output()?;
    if !output.status.success() {
        return Err(Error::CreateBranch {
            repo: repo.as_ref().to_string_lossy().into(),
            branch: branch_name.into(),
            error: String::from_utf8(output.stderr)?,
        });
    }
    Ok(())
}

fn delete_branch<P: AsRef<Path>>(repo: P, branch_name: &str) -> Result<(), Error> {
    let output = std::process::Command::new("git")
        .args(["branch", "-D", branch_name])
        .current_dir(&repo)
        .output()?;
    if !output.status.success() {
        return Err(Error::CreateBranch {
            repo: repo.as_ref().to_string_lossy().into(),
            branch: branch_name.into(),
            error: String::from_utf8(output.stderr)?,
        });
    }
    Ok(())
}

fn worktree_remove<P: AsRef<Path>>(path: P, repo: P) -> Result<(), Error> {
    let output = std::process::Command::new("git")
        .args(["worktree", "remove", path.as_ref().to_str().expect("path is valid utf-8")])
        .current_dir(&repo)
        .output()?;
    if !output.status.success() {
        return Err(Error::WorktreeRemove {
            repo: repo.as_ref().to_str().expect("repo is valid utf-8").into(),
            error: String::from_utf8(output.stderr)?,
        });
    };
    Ok(())
}

fn worktree_create<P: AsRef<Path>>(info: &WorktreeCreate, dir: P) -> Result<(), Error> {
    let path = Path::new(&info.path);
    if !path.is_absolute() {
        return Err(Error::WorktreeCreate {
            repo: "all".into(),
            branch: info.branch.clone(),
            error: format!("path {path:?} is not absolute"),
        });
    }

    let dir = dir.as_ref();
    if !dir.exists() {
        std::fs::create_dir_all(dir)?;
    }

    let mut created_branches = vec![];
    for repo in &info.repos {
        if info.create {
            if let Err(e) = create_branch(repo, &info.branch) {
                for r in created_branches {
                    _ = delete_branch(r, &info.branch);
                }
                return Err(Error::Worktree(e.into()));
            }
            created_branches.push(repo.to_string());
        }
    }

    let mut created_worktrees = vec![];
    for repo in &info.repos {
        let output = std::process::Command::new("git")
            .args(["worktree", "add", &format!("{}/{repo}", &info.path), &info.branch])
            .current_dir(repo)
            .output()?;
        if !output.status.success() {
            for r in created_worktrees {
                _ = worktree_remove(Path::new(&info.path).join(&r), r);
            }
            return Err(Error::WorktreeCreate {
                repo: repo.into(),
                branch: info.branch.clone(),
                error: String::from_utf8(output.stderr)?,
            });
        }
        created_worktrees.push(repo.into());
    }

    Ok(())
}

fn worktree<P: AsRef<Path>>(info: &WorktreeArgs, dir: P) -> Result<(), Error> {
    match &info.command {
        Worktree::Create(create) => worktree_create(create, dir),
    }
}

fn try_rebase<P: AsRef<Path>>(onto: &str, path: P) -> Result<bool, Error> {
    let path = path.as_ref();
    if !path.is_dir() {
        tracing::debug!("skipping entry '{}': not a directory", path.to_string_lossy());
        return Ok(false);
    }

    if !path.join(".git").exists() {
        tracing::debug!("skipping entry '{}': not a git repository", path.to_string_lossy());
        return Ok(false);
    }

    let output = std::process::Command::new("git").args(["rebase", onto]).current_dir(path).output()?;
    if !output.status.success() {
        return Err(Error::Rebase {
            repository: path.to_string_lossy().to_string(),
            error: String::from_utf8(output.stderr)?,
        });
    }

    tracing::debug!("> {}", String::from_utf8(output.stdout)?);

    Ok(true)
}

fn rebase<P: AsRef<Path>>(info: &Rebase, dir: P) -> Result<(), Error> {
    let dir = dir.as_ref();

    for entry in std::fs::read_dir(dir)? {
        let path = &entry?.path();

        match try_rebase(&info.onto, path) {
            Err(e) => tracing::error!("{e}"),
            Ok(true) => tracing::info!("rebased '{}' onto {}", path.to_string_lossy(), info.onto),
            Ok(false) => {}
        };
    }

    Ok(())
}

fn main() {
    let options = Options::parse();
    let max_level = match options.verbose {
        true => tracing::Level::DEBUG,
        false => tracing::Level::INFO,
    };

    tracing_subscriber::fmt()
        .without_time()
        .with_level(false)
        .with_max_level(max_level)
        .with_target(false)
        .init();

    if let Err(e) = match options.command {
        Command::Rebase(info) => rebase(&info, options.dir),
        Command::Worktree(info) => worktree(&info, options.dir),
    } {
        tracing::error!("{e}");
        std::process::exit(1);
    }
}
