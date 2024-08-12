use anyhow::Result;
use axum::response::Html;
use std::{fs, path::PathBuf};

pub struct Fallback(String);

impl Fallback {
    pub fn from_file(path: PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(Self(content))
    }

    pub fn to_html(&self) -> Html<String> {
        Html(self.0.clone())
    }
}

impl Default for Fallback {
    fn default() -> Self {
        Self("<h1>Service not found.</h1>".to_string())
    }
}
