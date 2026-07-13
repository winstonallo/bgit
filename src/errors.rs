use crate::branch;
use crate::rebase;
use crate::worktree;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),
    #[error("decoding error: {0}")]
    Decoding(#[from] std::string::FromUtf8Error),
    #[error("worktree error: {0}")]
    Worktree(#[from] worktree::Error),
    #[error("rebase error: {0}")]
    Rebase(#[from] rebase::Error),
    #[error("branch error: {0}")]
    Branch(#[from] branch::Error),
}
