use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Targets(HashMap<i64, PathBuf>);

impl Targets {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, func_id: i64, path: &Path) {
        self.0.insert(func_id, path.to_path_buf());
    }

    pub fn ready_for_bundling(&self) -> HashMap<String, String> {
        self.0
            .values()
            .map(|path| {
                let mut name = path.strip_prefix("dist").unwrap().to_path_buf();
                name.set_extension("");
                (
                    name.to_str().unwrap().to_string(),
                    path.canonicalize().unwrap().to_str().unwrap().to_string(),
                )
            })
            .collect()
    }

    pub fn ready_for_exec(&self) -> HashMap<i64, String> {
        self.0
            .iter()
            .map(|(&id, path)| (id, path.to_str().unwrap().to_string()))
            .collect()
    }
}
