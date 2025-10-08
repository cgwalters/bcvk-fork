//! libvirt integration for bcvk
//!
//! This module provides a comprehensive libvirt integration with subcommands for:
//! - `run`: Run a bootable container as a persistent VM
//! - `list`: List bootc domains with metadata
//! - `upload`: Upload bootc disk images to libvirt with metadata annotations
//! - `list-volumes`: List available bootc volumes with metadata

use clap::Subcommand;

/// Default memory allocation for libvirt VMs
pub const LIBVIRT_DEFAULT_MEMORY: &str = "4G";

/// Default vCPU count for libvirt VMs
pub const LIBVIRT_DEFAULT_VCPUS: u32 = 2;

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
