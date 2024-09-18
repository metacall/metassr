pub mod dist_dir;
pub mod src_dir;


use anyhow::Result;
pub trait DirectoryAnalyzer {
    type Output;
    fn analyze(&self) -> Result<Self::Output>;
}
