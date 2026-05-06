use std::path::Path;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
struct Rebase {
    #[arg(short, long, default_value_t = String::from("origin/master"))]
    onto: String,
}

#[derive(Subcommand, Debug)]
enum Command {
    Rebase(Rebase),
}

#[derive(Parser, Debug)]
struct Options {
    #[command(subcommand)]
    command: Command,
    #[arg(short, long, default_value_t = String::from("."))]
    dir: String,
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
}

fn rebase<P: AsRef<Path>>(info: &Rebase, dir: P) -> Result<(), Error> {
    let dir = dir.as_ref();

    for entry in std::fs::read_dir(dir)? {
        let path = &entry?.path();

        if !path.is_dir() {
            tracing::info!("skipping entry '{}', not a directory", path.to_string_lossy());
            continue;
        }

        if !path.join(".git").exists() {
            tracing::info!("skipping entry '{}', not a git repository", path.to_string_lossy());
            continue;
        }

        let output = std::process::Command::new("git").args(["rebase", &info.onto]).current_dir(&path).output()?;

        if !output.status.success() {
            return Err(Error::Rebase {
                repository: path.to_string_lossy().into(),
                error: String::from_utf8(output.stderr)?,
            });
        }

        tracing::info!("rebased '{}' onto {}", path.to_string_lossy(), info.onto);
    }

    Ok(())
}

fn main() {
    tracing_subscriber::fmt::init();
    let options = Options::parse();

    let res = match options.command {
        Command::Rebase(info) => rebase(&info, options.dir),
    };
    if let Err(e) = res {
        tracing::error!("{e}");
        std::process::exit(1);
    }
}
