pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Git(#[from] git2::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("cmd '{0}' failed with: {1}")]
    Execute(String, std::process::ExitStatus),
}

pub mod cli;
