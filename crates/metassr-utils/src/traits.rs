use anyhow::Result;
pub trait AnalyzeDir {
    type Output;
    fn analyze(&self) -> Result<Self::Output>;
}
