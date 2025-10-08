//! Helper functions for interacting with qemu-img

use camino::Utf8Path;
use color_eyre::{eyre::Context, Result};
use serde::Deserialize;
use std::process::Command;

/// Information returned by `qemu-img info --output=json`
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
pub struct QemuImgInfo {
    pub virtual_size: u64,
    pub filename: String,
    pub format: String,
    pub actual_size: Option<u64>,
    pub cluster_size: Option<u64>,
    pub backing_filename: Option<String>,
    pub full_backing_filename: Option<String>,
    pub dirty_flag: Option<bool>,
}

/// Run `qemu-img info --force-share --output=json` on a disk image
///
/// The `--force-share` flag allows reading disk info even when the image
/// is locked by a running VM.
pub fn info(path: &Utf8Path) -> Result<QemuImgInfo> {
    let output = Command::new("qemu-img")
        .args(["info", "--force-share", "--output=json", path.as_str()])
        .output()
        .with_context(|| format!("Failed to run qemu-img info on {:?}", path))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(color_eyre::eyre::eyre!(
            "qemu-img info failed for {:?}: {}",
            path,
            stderr
        ));
    }

    serde_json::from_slice(&output.stdout)
        .with_context(|| format!("Failed to parse qemu-img info JSON for {:?}", path))
}
