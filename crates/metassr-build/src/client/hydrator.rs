const HYDRATED_FILE_TEMPLATE: &str = include_str!("./scripts/hydrate.js.template");

const APP_PATH_TAG: &str = "%APP_PATH%";
const PAGE_PATH_TAG: &str = "%PAGE_PATH%";
const ROOT_ID_TAG: &str = "%ROOT_ID%";

#[derive(Debug, Clone)]
pub struct Hydrator {
    app_path: String,
    page_path: String,
    root_id: String,
}

impl Hydrator {
    pub fn new<'a>(app_path: &'a str, page_path: &'a str, root_id: &'a str) -> Self {
        Self {
            app_path: app_path.to_string(),
            page_path: page_path.to_string(),
            root_id: root_id.to_string(),
        }
    }

    pub fn generate(&self) -> String {
        HYDRATED_FILE_TEMPLATE
            .replace(APP_PATH_TAG, &self.app_path)
            .replace(PAGE_PATH_TAG, &self.page_path)
            .replace(ROOT_ID_TAG, &self.root_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generate_hydrated_file() {
        println!(
            "{}",
            Hydrator::new("src/_app.tsx", "src/pages/home.jsx", "root").generate()
        );
    }
}
