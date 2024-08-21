use anyhow::Result;

pub trait Exec {
    fn exec(&self) -> Result<()>;
}
pub trait AsyncExec {
    async fn exec(&self) -> Result<()>;
}
