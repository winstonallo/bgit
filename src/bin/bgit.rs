use bgit::rebase;
use bgit::worktree;
use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
enum Command {
    Rebase(bgit::rebase::Args),
    Worktree(bgit::worktree::Args),
}

#[derive(Parser, Debug)]
struct Options {
    #[command(subcommand)]
    command: Command,
    #[arg(long, default_value_t = false)]
    verbose: bool,
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
        Command::Rebase(info) => rebase::rebase(&info).map_err(bgit::errors::Error::from),
        Command::Worktree(info) => worktree::worktree(&info).map_err(bgit::errors::Error::from),
    } {
        tracing::error!("{e}");
        std::process::exit(1);
    }
}
