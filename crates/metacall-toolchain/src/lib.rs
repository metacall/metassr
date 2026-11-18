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
pub mod fetch;
pub mod index;
pub mod manifest;
pub mod resolve;
pub mod store;
pub mod target;
mod toolchain;
pub mod view;

pub use error::{Error, Result};
pub use fetch::{FetchPolicy, Fetched, HttpTransport, Transport};
pub use index::{Artifact, ArtifactKind, Index, TrustedKeys};
pub use manifest::Manifest;
pub use resolve::{resolve, LoaderSpec, Missing, Requirements, Resolution};
pub use store::{Installed, Store};
pub use target::{DottedVersion, Target};
pub use toolchain::{NoProgress, Options, Progress, Toolchain};
pub use view::View;
