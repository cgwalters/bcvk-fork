//! Print firmware information command
//!
//! This module provides a command to display detected OVMF firmware paths
//! and configuration for debugging firmware detection issues.

use clap::Parser;
use color_eyre::{eyre::Context, Result};
use serde::{Deserialize, Serialize};

use super::secureboot::{find_ovmf_vars, find_secure_boot_firmware};

/// Options for the print-firmware command
#[derive(Debug, Parser)]
pub struct LibvirtPrintFirmwareOpts {
    /// Output format (yaml or json)
    #[clap(long, default_value = "yaml", value_enum)]
    pub format: OutputFormat,
}

/// Output format for print-firmware command
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    /// YAML format (default, human-readable)
    Yaml,
    /// JSON format (machine-readable)
    Json,
}

/// Firmware information for display
#[derive(Debug, Serialize, Deserialize)]
pub struct PrintFirmwareInfo {
    /// Path to OVMF_VARS.fd (or equivalent)
    pub vars_path: Option<String>,
    /// Path to OVMF_CODE.secboot.fd (or equivalent)
    pub code_secboot_path: Option<String>,
    /// Format of OVMF_CODE file (raw or qcow2)
    pub code_format: Option<String>,
    /// Format of OVMF_VARS file (raw or qcow2)
    pub vars_format: Option<String>,
    /// Current architecture
    pub architecture: String,
}

/// Execute the print-firmware command
pub fn run(opts: LibvirtPrintFirmwareOpts) -> Result<()> {
    // Try to find OVMF_VARS (non-secboot variant)
    let vars_path = match find_ovmf_vars() {
        Ok(path) => Some(path.to_string()),
        Err(e) => {
            tracing::debug!("Failed to find OVMF_VARS: {}", e);
            None
        }
    };

    // Try to find secure boot firmware (CODE and VARS with formats)
    let (code_secboot_path, code_format, vars_format) = match find_secure_boot_firmware() {
        Ok(fw_info) => (
            Some(fw_info.code_path.to_string()),
            Some(fw_info.code_format),
            Some(fw_info.vars_format),
        ),
        Err(e) => {
            tracing::debug!("Failed to find secure boot firmware: {}", e);
            (None, None, None)
        }
    };

    let info = PrintFirmwareInfo {
        vars_path,
        code_secboot_path,
        code_format,
        vars_format,
        architecture: std::env::consts::ARCH.to_string(),
    };

    // Output in requested format
    match opts.format {
        OutputFormat::Yaml => {
            println!(
                "{}",
                serde_yaml::to_string(&info)
                    .with_context(|| "Failed to serialize firmware info as YAML")?
            );
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&info)
                    .with_context(|| "Failed to serialize firmware info as JSON")?
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firmware_info_serialization() {
        let info = PrintFirmwareInfo {
            vars_path: Some("/usr/share/edk2/ovmf/OVMF_VARS.fd".to_string()),
            code_secboot_path: Some("/usr/share/edk2/ovmf/OVMF_CODE.secboot.fd".to_string()),
            code_format: Some("raw".to_string()),
            vars_format: Some("raw".to_string()),
            architecture: "x86_64".to_string(),
        };

        // Test YAML serialization
        let yaml = serde_yaml::to_string(&info).unwrap();
        assert!(yaml.contains("vars_path"));
        assert!(yaml.contains("OVMF_VARS.fd"));
        assert!(yaml.contains("code_format"));

        // Test JSON serialization
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("vars_path"));
        assert!(json.contains("OVMF_VARS.fd"));
        assert!(json.contains("code_format"));
    }
}
