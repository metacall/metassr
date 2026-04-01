use anyhow::{anyhow, Result};
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};
use tracing::info;

static BUNDLE_SCRIPT: &str = include_str!("../vendor/bundle.js");
static PACKAGE_JSON: &str = include_str!("../vendor/package.json");

const PACKAGE_MANAGERS: &[&str] = &["npm", "pnpm", "yarn", "bun"];

fn vendor_dir() -> Result<PathBuf> {
    // Windows: use APPDATA (e.g. C:\Users\X\AppData\Roaming\metassr\vendor\bundler)
    // Unix: use HOME (e.g. ~/.metassr/vendor/bundler)
    if cfg!(windows) {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| anyhow!("APPDATA environment variable not set"))?;
        Ok(PathBuf::from(appdata)
            .join("metassr")
            .join("vendor")
            .join("bundler"))
    } else {
        let home =
            std::env::var("HOME").map_err(|_| anyhow!("HOME environment variable not set"))?;
        Ok(PathBuf::from(home)
            .join(".metassr")
            .join("vendor")
            .join("bundler"))
    }
}

fn detect_package_manager() -> Result<String> {
    for pm in PACKAGE_MANAGERS {
        // On Windows, package managers are .cmd scripts (npm.cmd, pnpm.cmd, etc.)
        // Using `cmd /C` ensures they are found via PATHEXT resolution.
        let found = if cfg!(windows) {
            Command::new("cmd")
                .args(["/C", pm, "--version"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        } else {
            Command::new(pm)
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok()
        };
        if found {
            return Ok(pm.to_string());
        }
    }
    Err(anyhow!(
        "No JavaScript package manager found.\n\
         Install one of: npm, pnpm, yarn, or bun.\n\
         npm comes with Node.js: https://nodejs.org"
    ))
}

/// Ensures the vendored `esbuild` is installed in `~/.metassr/vendor/bundler/`.
///
/// On first run, writes `bundle.js` and `package.json` to the vendor directory
/// and runs `npm install` (or pnpm/yarn/bun if npm is not available).
///
/// On subsequent runs, only re-installs if the `package.json` version has changed
/// (e.g., after a metassr update).
///
/// Returns the path to `bundle.js` for loading via MetaCall.
pub fn ensure_vendor_setup() -> Result<PathBuf> {
    let dir = vendor_dir()?;
    let bundle_path = dir.join("bundle.js");
    let package_json_path = dir.join("package.json");
    let esbuild_marker = dir.join("node_modules").join("esbuild");

    // Check if we need to (re-)install: missing node_modules or package.json version changed
    let needs_install = if esbuild_marker.exists() && package_json_path.exists() {
        let existing = std::fs::read_to_string(&package_json_path).unwrap_or_default();
        existing.trim() != PACKAGE_JSON.trim()
    } else {
        true
    };

    // Check if bundle.js needs updating independently of a full reinstall
    let bundle_outdated = if bundle_path.exists() {
        let existing = std::fs::read_to_string(&bundle_path).unwrap_or_default();
        existing.trim() != BUNDLE_SCRIPT.trim()
    } else {
        true
    };

    if needs_install {
        std::fs::create_dir_all(&dir)?;
        std::fs::write(&bundle_path, BUNDLE_SCRIPT)?;
        std::fs::write(&package_json_path, PACKAGE_JSON)?;

        let pm = detect_package_manager()?;
        info!("Installing vendored packages using {pm}...");

        let status = if cfg!(windows) {
            Command::new("cmd")
                .args(["/C", &pm, "install"])
                .current_dir(&dir)
                .status()
        } else {
            Command::new(&pm).arg("install").current_dir(&dir).status()
        }
        .map_err(|e| anyhow!("Failed to run {pm}: {e}"))?;

        if !status.success() {
            return Err(anyhow!("{pm} install failed in {}", dir.display()));
        }

        info!("Vendored esbuild installed successfully.");
    } else if bundle_outdated {
        // package.json unchanged but bundle.js was updated — just overwrite it
        std::fs::write(&bundle_path, BUNDLE_SCRIPT)?;
    }

    Ok(bundle_path)
}
