use std::env;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest_path = out_dir.join("templates.rs");
    println!("==> {dest_path:#?}");
    let templates_dir = "templates";
    let mut generated_code = String::new();

    generated_code.push_str("use std::collections::HashMap;\n\n");
    generated_code
        .push_str("pub fn load_templates() -> HashMap<String, HashMap<String, Vec<u8>>> {\n");
    generated_code.push_str("    let mut templates = HashMap::new();\n");

    for entry in WalkDir::new(templates_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            !e.path().components().any(|component| {
                component.as_os_str() == "node_modules" || component.as_os_str() == "dist"
            })
        })
    {
        let path = entry.path();
        let relative_path = path.strip_prefix(templates_dir).unwrap();
        let mut components = relative_path.components();
        let template_name = components
            .next()
            .expect("template root dir should be present")
            .as_os_str()
            .to_string_lossy()
            .into_owned();
        let file_name = components
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        let canonical_path = dunce::canonicalize(path).unwrap();
        let canonical_path = canonical_path.to_string_lossy();

        generated_code.push_str(&format!(
            "    templates.entry({template_name:?}.to_string()).or_insert_with(HashMap::new).insert({file_name:?}.to_string(), include_bytes!({canonical_path:?}).to_vec());\n",
        ));
    }

    generated_code.push_str("    templates\n");
    generated_code.push_str("}\n");

    fs::write(&dest_path, generated_code).unwrap();
    println!("cargo:rerun-if-changed={}", templates_dir);
}
