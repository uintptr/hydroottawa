use aws_cognito_srp::SrpError;
use thiserror::Error;

/// Result alias for fallible `hydroottawa-api` operations.
pub type Result<T> = core::result::Result<T, Error>;

/// Error type for all fallible `hydroottawa-api` operations.
#[derive(Debug, Error)]
pub enum Error {
    //
    // 3rd party
    //
    /// HTTP transport, status, or body-decoding error.
    #[error(transparent)]
    HttpError(#[from] ureq::Error),
    /// SRP handshake failure.
    #[error(transparent)]
    Srp(#[from] SrpError),
    /// JSON serialization or deserialization failure.
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    /// A response header contained invalid characters.
    #[error(transparent)]
    InvalidHeaderValue(#[from] ureq::http::header::ToStrError),

    //
    // Custom
    //
    /// A required response header was missing.
    #[error("Missing header: {0}")]
    MissingHeader(String),
    /// A token did not have the expected format.
    #[error("Invalid token format: {0}")]
    InvalidTokenFormat(String),
}
