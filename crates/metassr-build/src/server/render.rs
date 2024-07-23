use crate::{
    shared::{APP_PATH_TAG, FUNC_ID_TAG, PAGE_PATH_TAG},
    traits::Generate,
};
use anyhow::Result;
use metassr_utils::rand::Rand;
use std::{ffi::OsStr, path::PathBuf};

const RENDER_FILE_TEMPLATE: &str = include_str!("../scripts/render.js.template");

pub struct ServerRender {
    app_path: PathBuf,
    page_path: PathBuf,
}

impl ServerRender {
    pub fn new<'a, S>(app_path: &'a S, page_path: &'a S) -> Self
    where
        S: AsRef<OsStr> + ?Sized,
    {
        Self {
            app_path: PathBuf::from(app_path),
            page_path: PathBuf::from(page_path),
        }
    }
}

impl Generate for ServerRender {
    type Output = (i64, String);
    fn generate(&self) -> Result<Self::Output> {
        let func_id = Rand::new().val();
        let mut app_path = self.app_path.canonicalize()?;
        let mut page_path = self.page_path.canonicalize()?;

        app_path.set_extension("");
        page_path.set_extension("");

        Ok((
            func_id,
            RENDER_FILE_TEMPLATE
                .replace(APP_PATH_TAG, app_path.to_str().unwrap())
                .replace(PAGE_PATH_TAG, page_path.to_str().unwrap())
                .replace(FUNC_ID_TAG, &func_id.to_string()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generate_render_file() {
        println!(
            "{:?}",
            ServerRender::new("src/_app.tsx", "src/pages/home.jsx")
                .generate()
                .unwrap()
        );
    }
}
