use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Other(#[from] anyhow::Error),

    #[error("config error: {0:?}")]
    ConfigError(cfg_rs::ConfigError),

    #[error("Initialization for '{0}' has already been done.")]
    DuplicatedInit(&'static str),
}

impl From<cfg_rs::ConfigError> for Error {
    fn from(err: cfg_rs::ConfigError) -> Self {
        Error::ConfigError(err)
    }
}