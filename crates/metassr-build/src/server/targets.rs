use std::{
    collections::{hash_map::Iter, HashMap},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct Targets(HashMap<PathBuf, i64>);

impl Targets {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, func_id: i64, path: &Path) {
        self.0.insert(path.to_path_buf(), func_id);
    }

    pub fn ready_for_bundling(&self, dist_path: &PathBuf) -> HashMap<String, String> {
        self.0
            .keys()
            .map(|path| {
                let mut name = match path.strip_prefix(dist_path) {
                    Ok(p) => p,
                    Err(e) => panic!(
                        "Couldn't \"{}\".strip_prefix(\"{}\"): {e}",
                        dist_path.display(),
                        path.display()
                    ),
                }
                .to_path_buf();
                name.set_extension("");
                (
                    name.to_str().unwrap().to_string(),
                    path.canonicalize().unwrap().to_str().unwrap().to_string(),
                )
            })
            .collect()
    }

    pub fn ready_for_exec(&self) -> HashMap<String, i64> {
        self.0
            .iter()
            .map(|(path, &id)| (path.to_str().unwrap().to_string(), id))
            .collect()
    }

    pub fn iter(&self) -> Iter<'_, PathBuf, i64> {
        self.0.iter()
    }
}

impl Default for Targets {
    fn default() -> Self {
        Self::new()
    }
}
