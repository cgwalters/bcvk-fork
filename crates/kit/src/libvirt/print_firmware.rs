//! Print firmware information command
//!
//! This module provides a command to display detected OVMF firmware paths
//! and configuration for debugging firmware detection issues.

use clap::Parser;
use color_eyre::{eyre::Context, Result};
use serde::{Deserialize, Serialize};

use super::secureboot::{find_ovmf_code_secboot, find_ovmf_vars};

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

/// Firmware information
#[derive(Debug, Serialize, Deserialize)]
pub struct FirmwareInfo {
    /// Path to OVMF_VARS.fd (or equivalent)
    pub vars_path: Option<String>,
    /// Path to OVMF_CODE.secboot.fd (or equivalent)
    pub code_secboot_path: Option<String>,
    /// Current architecture
    pub architecture: String,
}

/// Execute the print-firmware command
pub fn run(opts: LibvirtPrintFirmwareOpts) -> Result<()> {
    // Try to find OVMF_VARS
    let vars_path = match find_ovmf_vars() {
        Ok(path) => Some(path.to_string()),
        Err(e) => {
            tracing::debug!("Failed to find OVMF_VARS: {}", e);
            None
        }
    };

    // Try to find OVMF_CODE.secboot
    let code_secboot_path = match find_ovmf_code_secboot() {
        Ok(path) => Some(path.to_string()),
        Err(e) => {
            tracing::debug!("Failed to find OVMF_CODE.secboot: {}", e);
            None
        }
    };

    let info = FirmwareInfo {
        vars_path,
        code_secboot_path,
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
        let info = FirmwareInfo {
            vars_path: Some("/usr/share/edk2/ovmf/OVMF_VARS.fd".to_string()),
            code_secboot_path: Some("/usr/share/edk2/ovmf/OVMF_CODE.secboot.fd".to_string()),
            architecture: "x86_64".to_string(),
        };

        // Test YAML serialization
        let yaml = serde_yaml::to_string(&info).unwrap();
        assert!(yaml.contains("vars_path"));
        assert!(yaml.contains("OVMF_VARS.fd"));

        // Test JSON serialization
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("vars_path"));
        assert!(json.contains("OVMF_VARS.fd"));
    }
}
