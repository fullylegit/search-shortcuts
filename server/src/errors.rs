use thiserror::Error;

pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("A required environment variable isn't set: {0}")]
    EnvVarMissing(&'static str),
    #[error("A required environment variable isn't valid utf-8: {0}")]
    EnvVarInvalidUtf8(&'static str),
    #[error("IoError: {0}")]
    Io(#[from] std::io::Error),
    #[error("Error determining redirect url: {0}")]
    Url(#[from] search_shortcuts::errors::Error),
    #[error("Error configuring TLS: {0}")]
    Tls(&'static str),
}

impl actix_web::ResponseError for Error {}
