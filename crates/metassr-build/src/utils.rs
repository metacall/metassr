use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub fn setup_page_path(page: &str, ext: &str) -> PathBuf {
    match Path::new(page) {
        path if path.file_stem() != Some(OsStr::new("index")) => path
            .to_path_buf()
            .with_extension("")
            .join(format!("index.{ext}")),

        path => path.to_path_buf().with_extension(ext),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_index_filename() {
        assert_eq!(setup_page_path("index", "js"), PathBuf::from("index.js"));
    }

    #[test]
    fn wraps_non_index_in_subdir() {
        assert_eq!(
            setup_page_path("about", "js"),
            PathBuf::from("about/index.js"),
        );
    }

    // `pages` entries come from fs-analyzer with their original extension
    // (e.g. `about.tsx`). The output extension should win.
    #[test]
    fn replaces_source_extension() {
        assert_eq!(
            setup_page_path("about.tsx", "server.js"),
            PathBuf::from("about/index.server.js"),
        );
        assert_eq!(
            setup_page_path("index.tsx", "server.js"),
            PathBuf::from("index.server.js"),
        );
    }
}
