//! Install, pin and wire MetaCall loaders together with their runtimes.
//!
//! MetaCall loads every loader from a single directory and reads loader
//! configs from a single `global.json`. This crate keeps each downloaded
//! artifact in its own store directory, and builds per-project *views*: a
//! directory of symlinks plus generated configs that MetaCall runs against.
//!
//! The crate never prompts and prints nothing. Callers decide when to
//! download.

pub mod error;
pub mod index;
pub mod target;

pub use error::{Error, Result};
pub use index::{Artifact, ArtifactKind, Index, TrustedKeys};
pub use target::{DottedVersion, Target};
