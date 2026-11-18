use std::fmt;

/// Errors returned by the toolchain.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The index signature does not verify against any trusted key.
    #[error("index signature is invalid or not made by a trusted key")]
    BadSignature,

    /// The signature or a trusted key could not be decoded.
    #[error("malformed {what}: {reason}")]
    MalformedSignature { what: &'static str, reason: String },

    /// The index JSON is malformed or fails validation.
    #[error("malformed index: {0}")]
    MalformedIndex(String),

    /// The index uses a schema this build does not understand.
    #[error("unsupported index schema {found} (this build reads schema {supported})")]
    UnsupportedSchema { found: u32, supported: u32 },
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn malformed_index(reason: impl fmt::Display) -> Error {
    Error::MalformedIndex(reason.to_string())
}
