use std::path::Path;

pub fn to_js_path(path: &Path) -> String {
    path.to_str().unwrap().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_backslashes() {
        let path = Path::new("C:\\Users\\test\\file.tsx");
        assert_eq!(to_js_path(&path), "C:/Users/test/file.tsx");
    }

    #[test]
    fn handles_forward_slashes() {
        let path = Path::new("/home/user/file.tsx");
        assert_eq!(to_js_path(&path), "/home/user/file.tsx");
    }
}
