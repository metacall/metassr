use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub fn setup_page_path(page: &str, ext: &str) -> PathBuf {
    match Path::new(page) {
        path if path.file_stem() != Some(OsStr::new("index")) => {
            path.to_path_buf().with_extension("").join(format!("index.{ext}"))
        }

        path => path.to_path_buf().with_extension(ext),
    }
}
