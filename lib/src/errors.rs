use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error parsing url: {0}")]
    UrlParseError(#[from] url::ParseError),
}
