use anyhow::{anyhow, Result};
use rspack::builder::{
    Builder as _, Devtool, ModuleOptionsBuilder, OptimizationOptionsBuilder, OutputOptionsBuilder,
};
use rspack_core::{
    Compiler, EntryDescription, Experiments, Filename, LibraryOptions, LibraryType, Mode,
    ModuleRule, ModuleRuleEffect, ModuleRuleUse, ModuleRuleUseLoader, ModuleType, PublicPath,
    Resolve, RuleSetCondition, StatsOptions,
};
use rspack_fs::{NativeFileSystem, WritableFileSystem};
use rspack_paths::Utf8PathBuf;
use rspack_regex::RspackRegex;
use serde_json::json;
use std::{collections::HashMap, ffi::OsStr, marker::Sized, path::Path, sync::Arc, vec};

#[derive(Debug)]
pub struct WebBundler<'a> {
    pub targets: HashMap<String, &'a Path>,
    pub dist_path: &'a Path,
}

impl<'a> WebBundler<'a> {
    pub fn new<S>(targets: &'a HashMap<String, String>, dist_path: &'a S) -> Result<Self>
    where
        S: AsRef<OsStr> + ?Sized,
    {
        let mut non_found_files = vec![];
        let targets: HashMap<String, &Path> = targets
            .iter()
            .map(|(k, path)| {
                let path = Path::new(path);
                if !path.exists() {
                    non_found_files.push(path.to_str().unwrap());
                }
                (k.into(), path)
            })
            .collect();

        if !non_found_files.is_empty() {
            return Err(anyhow!(
                "[bundler] Non Exist files found: {:?}",
                non_found_files
            ));
        }

        Ok(Self {
            targets,
            dist_path: Path::new(dist_path),
        })
    }

    pub fn exec(&self) -> Result<()> {
        let dist_path_utf8 = Utf8PathBuf::from_path_buf(self.dist_path.to_path_buf())
            .map_err(|e| anyhow!("Failed to convert path to Utf8PathBuf: {:?}", e))?;

        let native_fs = Arc::new(NativeFileSystem::new(false));

        let mut compiler = Compiler::builder();
        compiler
            .output(
                OutputOptionsBuilder::default()
                    .filename(Filename::from("[name].js"))
                    .library(LibraryOptions {
                        library_type: LibraryType::from("commonjs2"),
                        name: None,
                        export: None,
                        umd_named_define: None,
                        auxiliary_comment: None,
                        amd_container: None,
                    })
                    .path(dist_path_utf8.clone())
                    .public_path(PublicPath::Filename(Filename::from(""))),
            )
            .resolve(Resolve {
                extensions: Some(vec![
                    ".js".to_string(),
                    ".jsx".to_string(),
                    ".tsx".to_string(),
                    ".ts".to_string(),
                ]),
                ..Default::default()
            })
            .optimization(OptimizationOptionsBuilder::default().minimize(true))
            .module(ModuleOptionsBuilder::default().rules(create_module_rules()))
            .enable_loader_swc()
            .name("Client".to_string())
            .mode(Mode::Production)
            .devtool(Devtool::SourceMap)
            .experiments(Experiments::builder().css(true))
            .stats(StatsOptions { colors: true })
            .target(vec!["web".to_string()])
            .output_filesystem(native_fs.clone());

        for (entry_name, entry_path) in &self.targets {
            let entry_path_str = entry_path
                .to_str()
                .ok_or_else(|| anyhow!("Invalid UTF-8 in entry path: {:?}", entry_path))?;

            compiler.entry(entry_name.clone(), EntryDescription::from(entry_path_str));
        }
        let mut compiler = compiler
            .build()
            .map_err(|e| anyhow!("Failed to build rspack compiler: {}", e))?;

        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                // Tokio runtime exists (CLI) - use block_in_place
                tokio::task::block_in_place(|| {
                    handle.block_on(async {
                        native_fs
                            .create_dir_all(&dist_path_utf8)
                            .await
                            .map_err(|e| anyhow!("Failed to create directory: {}", e))?;
                        compiler
                            .run()
                            .await
                            .map_err(|e| anyhow!("Compilation failed: {}", e))?;
                        Ok::<(), anyhow::Error>(())
                    })
                })
                .map_err(|e| anyhow!("Block in place failed: {}", e))?
            }
            Err(_) => {
                // No Tokio runtime (tests) - create new one
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| anyhow!("Failed to create runtime: {}", e))?;

                rt.block_on(async {
                    native_fs
                        .create_dir_all(&dist_path_utf8)
                        .await
                        .map_err(|e| anyhow!("Failed to create directory: {}", e))?;
                    compiler
                        .run()
                        .await
                        .map_err(|e| anyhow!("Compilation failed: {}", e))?;
                    Ok::<(), anyhow::Error>(())
                })
                .map_err(|e| anyhow!("Runtime block failed: {}", e))?
            }
        }
        // Check for compilation errors
        // Collect any errors from the compilation
        let errors: Vec<_> = compiler.compilation.get_errors().collect();

        if !errors.is_empty() {
            let error_messages: Vec<String> = errors.iter().map(|e| format!("{:#?}", e)).collect();

            return Err(anyhow!(
                "Bundling failed with {} error(s): {}",
                error_messages.len(),
                error_messages.join("\n")
            ));
        }

        Ok(())
    }
}

fn create_module_rules() -> Vec<ModuleRule> {
    let mut rules = Vec::new();

    let js_regex = RspackRegex::new(r"\.(jsx|js)$").unwrap();
    let js_exclude_regex = RspackRegex::new(r"node_modules").unwrap();
    let js_rule = ModuleRule {
        test: Some(RuleSetCondition::Regexp(js_regex)),
        exclude: Some(RuleSetCondition::Regexp(js_exclude_regex)),
        effect: ModuleRuleEffect {
            r#use: ModuleRuleUse::Array(vec![ModuleRuleUseLoader {
                loader: "builtin:swc-loader".to_string(),
                options: Some(
                    json!({
                        "jsc": {
                            "parser": {
                                "syntax": "ecmascript",
                                "jsx": true,
                                "dynamicImport": true,
                            },
                            "transform": {
                                "react": {
                                    "runtime": "automatic",
                                    "throwIfNamespace": true,
                                }
                            }
                        }
                    })
                    .to_string(),
                ),
            }]),
            r#type: Some(ModuleType::JsAuto),
            ..Default::default()
        },
        ..Default::default()
    };

    rules.push(js_rule);

    let ts_regex = RspackRegex::new(r"\.(tsx|ts)$").unwrap();
    let ts_exclude_regex = RspackRegex::new(r"node_modules").unwrap();
    let ts_rule = ModuleRule {
        test: Some(RuleSetCondition::Regexp(ts_regex)),
        exclude: Some(RuleSetCondition::Regexp(ts_exclude_regex)),
        effect: ModuleRuleEffect {
            r#use: ModuleRuleUse::Array(vec![ModuleRuleUseLoader {
                loader: "builtin:swc-loader".to_string(),
                options: Some(
                    json!({
                        "jsc": {
                            "parser": {
                                "syntax": "typescript",
                                "tsx": true,
                                "decorators": true
                            },
                            "transform": {
                                "react": {
                                    "runtime": "automatic",
                                    "throwIfNamespace": true,
                                }
                            }
                        }
                    })
                    .to_string(),
                ),
            }]),
            r#type: Some(ModuleType::JsAuto),
            ..Default::default()
        },
        ..Default::default()
    };

    rules.push(ts_rule);

    let asset_regex =
        RspackRegex::new(r"\.(png|svg|jpg|jpeg|gif|woff|woff2|eot|ttf|otf|webp)$").unwrap();
    let asset_rule = ModuleRule {
        test: Some(RuleSetCondition::Regexp(asset_regex)),
        effect: ModuleRuleEffect {
            r#type: Some(ModuleType::AssetInline),
            ..Default::default()
        },
        ..Default::default()
    };
    rules.push(asset_rule);
    rules
}

#[cfg(test)]
mod tests {

    use super::*;
    use metacall::initialize;

    fn clean() {
        let dist = Path::new("tests/dist");
        if dist.exists() {
            std::fs::remove_dir_all(dist).unwrap();
        }
    }

    #[test]
    fn bundling_works() {
        clean();
        let _metacall = initialize().unwrap();
        let targets = HashMap::from([("pages/home".to_owned(), "./tests/home.js".to_owned())]);

        match WebBundler::new(&targets, "tests/dist") {
            Ok(bundler) => {
                assert!(bundler.exec().is_ok());
                assert!(Path::new("tests/dist/pages/home.js").exists());
            }
            Err(err) => {
                panic!("BUNDLING TEST FAILED: {err:?}",)
            }
        }
        clean();
    }

    #[test]
    fn invalid_target_fails() {
        clean();
        let targets = HashMap::from([("invalid_path.tsx".to_owned(), "invalid_path".to_owned())]);

        let bundler = WebBundler::new(&targets, "tests/dist");
        assert!(bundler.is_err());
    }
}
