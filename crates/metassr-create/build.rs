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
        .push_str("pub fn load_templates() -> HashMap<String, HashMap<String, String>> {\n");
    generated_code.push_str("    let mut templates = HashMap::new();\n");

    for entry in WalkDir::new(templates_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let relative_path = path.strip_prefix(templates_dir).unwrap().to_str().unwrap();
        let template_name = relative_path.split('/').next().unwrap();
        let file_name = relative_path
            .split('/')
            .skip(1)
            .collect::<Vec<_>>()
            .join("/");

        generated_code.push_str(&format!(
            "    templates.entry(\"{}\".to_string()).or_insert_with(HashMap::new).insert(\"{}\".to_string(), include_str!(r#\"{}\"#).to_string());\n",
            template_name,
            file_name,
            path.display()
        ));
    }

    generated_code.push_str("    templates\n");
    generated_code.push_str("}\n");

    fs::write(&dest_path, generated_code).unwrap();
    println!("cargo:rerun-if-changed={}", templates_dir);
}
