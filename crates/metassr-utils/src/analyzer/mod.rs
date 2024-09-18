use anyhow::Result;

/// Analyzes the `dist/` directory for scripts and style files. The `dist/` directory
/// contains bundled and compiled assets like JavaScript and CSS files.
pub mod dist_dir;

/// Analyzes the `src/` directory for source code files. The `src/` directory typically
/// contains raw, uncompiled TypeScript or JavaScript modules used in a project.
pub mod src_dir;

/// A trait for analyzing directories to extract and process files.
///
/// Implement this trait for any directory that needs to be analyzed. The trait defines
/// an associated `Output` type and a required `analyze` method to return a result of that type.
pub trait DirectoryAnalyzer {
    /// The output type returned by the `analyze` function.
    type Output;

    /// Analyzes the directory and returns the result of the analysis.
    ///
    /// # Errors
    /// If the directory or its contents cannot be analyzed (e.g., due to a missing directory,
    /// unreadable files, or unexpected formats), the function returns an error.
    fn analyze(&self) -> Result<Self::Output>;
}
