use std::{fmt, io, path::PathBuf};

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

    /// A filesystem operation failed.
    #[error("{op} {}: {source}", path.display())]
    Io {
        op: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    /// A downloaded archive does not match the sha256 from the index.
    #[error("{id}: sha256 mismatch (expected {expected}, got {actual})")]
    HashMismatch {
        id: String,
        expected: String,
        actual: String,
    },

    /// An archive holds an entry that would escape the store or is not a
    /// plain file, directory or symlink.
    #[error("{id}: refusing to unpack: {reason}")]
    UnsafeArchive { id: String, reason: String },

    /// An artifact's `manifest.json` is missing or invalid.
    #[error("{id}: bad manifest: {reason}")]
    BadManifest { id: String, reason: String },

    /// The artifact is not in the store.
    #[error("{0} is not installed")]
    NotInstalled(String),

    /// The artifact belongs to the host package and cannot be removed.
    #[error("{0} is built in and cannot be removed")]
    External(String),

    /// Two components want the same file name in the view's loader directory.
    #[error("`{name}` is provided by both {first} and {second}")]
    ViewConflict {
        name: String,
        first: String,
        second: String,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn io_err(
    op: &'static str,
    path: impl Into<PathBuf>,
) -> impl FnOnce(io::Error) -> Error {
    let path = path.into();
    move |source| Error::Io { op, path, source }
}

pub(crate) fn malformed_index(reason: impl fmt::Display) -> Error {
    Error::MalformedIndex(reason.to_string())
}
