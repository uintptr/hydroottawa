use aws_cognito_srp::SrpError;
use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;
#[derive(Debug, Error)]
pub enum Error {
    //
    // 3rd party
    //
    #[error(transparent)]
    HttpError(#[from] reqwest::Error),
    #[error(transparent)]
    Srp(#[from] SrpError),
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    #[error(transparent)]
    InvalidHeaderValue(#[from] reqwest::header::ToStrError),

    //
    // Custom
    //
    #[error("Missing header: {0}")]
    MissingHeader(String),
    #[error("Invalid token format: {0}")]
    InvalidTokenFormat(String),
}
