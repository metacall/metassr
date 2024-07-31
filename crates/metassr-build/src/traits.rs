use anyhow::Result;
pub trait Exec {
    type Output;
    fn exec(&self) -> Result<Self::Output>;
}

pub trait Generate {
    type Output;
    fn generate(&self) -> Result<Self::Output>;
}

pub trait Build {
    type Output;
    fn build(&self) -> Result<Self::Output>;
}
