use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub fn setup_page_path(page: &str, ext: &str) -> PathBuf {
    match Path::new(page) {
        path if path.file_stem() != Some(OsStr::new("index")) => {
            let mut path = path.to_path_buf();
            path.set_extension("");
            path.join(format!("index.{ext}"))
        }

        path => {
            let mut path = path.to_path_buf();
            path.set_extension(ext);
            path
        }
    }
}
