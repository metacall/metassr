pub mod client;
pub(crate) mod page;
pub mod server;
pub(crate) mod shared;
pub mod traits;
pub(crate) mod utils;

// Re-export commonly used types for convenience
pub use client::{
    config::{ClientConfig, BundlerOptions},
    ClientBuilder,
};
pub use server::{
    config::{ServerConfig, BuildingType, ServerBundlerOptions, ManifestOptions},
    ServerSideBuilder,
};
pub use traits::{Build, Generate, Exec};
