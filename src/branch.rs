use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("could not create branch: {0}")]
    Create(#[from] CreateError),
    #[error("could not delete branch: {0}")]
    Delete(#[from] DeleteError),
}

#[derive(thiserror::Error, Debug)]
pub enum CreateError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("could not create branch {branch} for repo {repo}: {error}")]
    CouldNotCreate { repo: String, branch: String, error: String },
    #[error("decoding error: {0}")]
    Decoding(#[from] std::string::FromUtf8Error),
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("could not delete branch {branch} for repo {repo}: {error}")]
    CouldNotDelete { repo: String, branch: String, error: String },
    #[error("decoding error: {0}")]
    Decoding(#[from] std::string::FromUtf8Error),
}

fn create_inner<P: AsRef<Path>>(repo: P, branch: &str) -> Result<(), CreateError> {
    let output = std::process::Command::new("git")
        .args(["branch", branch])
        .current_dir(&repo)
        .output()
        .map_err(CreateError::from)?;

    if !output.status.success() {
        return Err(CreateError::CouldNotCreate {
            repo: repo.as_ref().to_string_lossy().into(),
            branch: branch.to_string(),
            error: String::from_utf8(output.stderr)?,
        });
    }

    Ok(())
}

pub fn create<P: AsRef<Path>>(repo: P, branch: &str) -> Result<(), Error> {
    Ok(create_inner(repo, branch)?)
}

fn delete_innner<P: AsRef<Path>>(repo: P, branch: &str) -> Result<(), DeleteError> {
    let output = std::process::Command::new("git")
        .args(["branch", "-D", branch])
        .current_dir(&repo)
        .output()
        .map_err(DeleteError::from)?;

    if !output.status.success() {
        return Err(DeleteError::CouldNotDelete {
            repo: repo.as_ref().to_string_lossy().into(),
            branch: branch.to_string(),
            error: String::from_utf8(output.stderr)?,
        });
    }

    Ok(())
}

pub fn delete<P: AsRef<Path>>(repo: P, branch: &str) -> Result<(), Error> {
    Ok(delete_innner(repo, branch)?)
}
