//! libvirt integration for bcvk
//!
//! This module provides a comprehensive libvirt integration with subcommands for:
//! - `run`: Run a bootable container as a persistent VM
//! - `list`: List bootc domains with metadata
//! - `upload`: Upload bootc disk images to libvirt with metadata annotations
//! - `list-volumes`: List available bootc volumes with metadata

use clap::Subcommand;

/// Output format options for libvirt commands
#[derive(Debug, Clone, clap::ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
}

/// Default memory allocation for libvirt VMs
pub const LIBVIRT_DEFAULT_MEMORY: &str = "4G";

/// Default disk size for libvirt base disks
pub const LIBVIRT_DEFAULT_DISK_SIZE: &str = "20G";

pub mod base_disks;
pub mod base_disks_cli;
pub mod domain;
pub mod inspect;
pub mod list;
pub mod list_volumes;
pub mod rm;
pub mod rm_all;
pub mod run;
pub mod secureboot;
pub mod ssh;
pub mod start;
pub mod status;
pub mod stop;
pub mod upload;

/// Global options for libvirt operations
#[derive(Debug, Clone, Default)]
pub struct LibvirtOptions {
    /// Hypervisor connection URI (e.g., qemu:///system, qemu+ssh://host/system)
    pub connect: Option<String>,
}

impl LibvirtOptions {
    /// Create a virsh Command with the appropriate connection URI using host execution
    ///
    /// Note: This method may panic if host execution setup fails, but this should
    /// only happen in misconfigured environments where container lacks required privileges
    pub fn virsh_command(&self) -> std::process::Command {
        let mut cmd = crate::hostexec::command("virsh", None)
            .expect("Failed to setup host execution for virsh - ensure container has --privileged and --pid=host");
        if let Some(ref uri) = self.connect {
            cmd.arg("-c").arg(uri);
        }
        cmd
    }
}

/// Convert memory value with unit to megabytes (MiB)
/// Handles libvirt-style units distinguishing between decimal (KB, MB, GB - powers of 1000)
/// and binary (KiB, MiB, GiB - powers of 1024) units per libvirt specification
pub(crate) fn convert_memory_to_mb(value: u32, unit: &str) -> Option<u32> {
    // Use u128 for calculations to prevent overflow with large units like TB
    let value_u128 = value as u128;
    let mib_u128 = 1024 * 1024;

    let mb = match unit {
        // Binary prefixes (powers of 1024), converting to MiB
        "k" | "K" | "KiB" => value_u128 / 1024,
        "M" | "MiB" => value_u128,
        "G" | "GiB" => value_u128 * 1024,
        "T" | "TiB" => value_u128 * 1024 * 1024,

        // Decimal prefixes (powers of 1000), converting to MiB
        "B" | "bytes" => value_u128 / mib_u128,
        "KB" => (value_u128 * 1_000u128.pow(1)) / mib_u128,
        "MB" => (value_u128 * 1_000u128.pow(2)) / mib_u128,
        "GB" => (value_u128 * 1_000u128.pow(3)) / mib_u128,
        "TB" => (value_u128 * 1_000u128.pow(4)) / mib_u128,

        // Libvirt default is KiB for memory
        _ => value_u128 / 1024,
    };
    u32::try_from(mb).ok()
}

/// Parse memory value from a libvirt XML node with unit attribute
/// Returns the value in megabytes (MiB)
pub(crate) fn parse_memory_mb(node: &crate::xml_utils::XmlNode) -> Option<u32> {
    let value = node.text_content().parse::<u32>().ok()?;
    // Convert to MB based on unit attribute (default is KiB per libvirt spec)
    let unit = node
        .attributes
        .get("unit")
        .map(|s| s.as_str())
        .unwrap_or("KiB");
    convert_memory_to_mb(value, unit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_memory_to_mb() {
        // Test binary units (powers of 1024)
        assert_eq!(convert_memory_to_mb(4194304, "KiB"), Some(4096));
        assert_eq!(convert_memory_to_mb(2097152, "KiB"), Some(2048));
        assert_eq!(convert_memory_to_mb(2048, "MiB"), Some(2048));
        assert_eq!(convert_memory_to_mb(4096, "MiB"), Some(4096));
        assert_eq!(convert_memory_to_mb(4, "GiB"), Some(4096));
        assert_eq!(convert_memory_to_mb(2, "GiB"), Some(2048));

        // Test short forms (binary)
        assert_eq!(convert_memory_to_mb(4, "G"), Some(4096));
        assert_eq!(convert_memory_to_mb(2048, "M"), Some(2048));
        assert_eq!(convert_memory_to_mb(2097152, "K"), Some(2048));

        // Test decimal units (powers of 1000)
        assert_eq!(convert_memory_to_mb(1048576, "KB"), Some(1000));
        assert_eq!(convert_memory_to_mb(1024, "MB"), Some(976));
        assert_eq!(convert_memory_to_mb(4, "GB"), Some(3814));

        // Test default/unknown unit (defaults to KiB)
        assert_eq!(convert_memory_to_mb(4194304, "unknown"), Some(4096));
    }

    #[test]
    fn test_parse_memory_mb() {
        use crate::xml_utils::parse_xml_dom;

        // Test KiB (default unit)
        let xml = r#"<memory>4194304</memory>"#;
        let dom = parse_xml_dom(xml).unwrap();
        assert_eq!(parse_memory_mb(&dom), Some(4096));

        // Test MiB
        let xml = r#"<memory unit='MiB'>2048</memory>"#;
        let dom = parse_xml_dom(xml).unwrap();
        assert_eq!(parse_memory_mb(&dom), Some(2048));

        // Test GiB
        let xml = r#"<memory unit='GiB'>4</memory>"#;
        let dom = parse_xml_dom(xml).unwrap();
        assert_eq!(parse_memory_mb(&dom), Some(4096));

        // Test KB (decimal unit: 1000-based)
        let xml = r#"<memory unit='KB'>1048576</memory>"#;
        let dom = parse_xml_dom(xml).unwrap();
        assert_eq!(parse_memory_mb(&dom), Some(1000));
    }
}

/// libvirt subcommands for managing bootc disk images and domains
#[derive(Debug, Subcommand)]
pub enum LibvirtSubcommands {
    /// Run a bootable container as a persistent VM
    Run(run::LibvirtRunOpts),

    /// SSH to libvirt domain with embedded SSH key
    Ssh(ssh::LibvirtSshOpts),

    /// List bootc domains with metadata
    List(list::LibvirtListOpts),

    /// List available bootc volumes with metadata
    #[clap(name = "list-volumes")]
    ListVolumes(list_volumes::LibvirtListVolumesOpts),

    /// Stop a running libvirt domain
    Stop(stop::LibvirtStopOpts),

    /// Start a stopped libvirt domain
    Start(start::LibvirtStartOpts),

    /// Remove a libvirt domain and its resources
    #[clap(name = "rm")]
    Remove(rm::LibvirtRmOpts),

    /// Remove multiple libvirt domains and their resources
    #[clap(name = "rm-all")]
    RemoveAll(rm_all::LibvirtRmAllOpts),

    /// Show detailed information about a libvirt domain
    Inspect(inspect::LibvirtInspectOpts),

    /// Show libvirt environment status and capabilities
    Status(status::LibvirtStatusOpts),

    /// Upload bootc disk images to libvirt with metadata annotations
    Upload(upload::LibvirtUploadOpts),

    /// Manage base disk images used for VM cloning
    #[clap(name = "base-disks")]
    BaseDisks(base_disks_cli::LibvirtBaseDisksOpts),
}
