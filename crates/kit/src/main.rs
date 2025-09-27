use std::ffi::OsString;

#[cfg(target_os = "linux")]
use cap_std_ext::cap_std::fs::Dir;
use clap::{Parser, Subcommand};
use color_eyre::{eyre::Context as _, Report, Result};

mod arch;
#[cfg(target_os = "linux")]
mod boot_progress;
mod cli_json;
mod cmdext;
mod cmdfdext;
mod common_opts;
mod container_entrypoint;
#[cfg(target_os = "linux")]
pub(crate) mod containerenv;
#[cfg(target_os = "linux")]
mod domain_list;
#[cfg(target_os = "linux")]
mod envdetect;
#[cfg(target_os = "linux")]
mod ephemeral;
#[cfg(not(target_os = "linux"))]
#[path = "ephemeral_stub.rs"]
mod ephemeral;
mod hostexec;
mod images;
mod install_options;
#[cfg(target_os = "linux")]
mod libvirt;
#[cfg(target_os = "linux")]
mod libvirt_upload_disk;
#[allow(dead_code)]
mod podman;
#[cfg(target_os = "linux")]
#[allow(dead_code)]
mod qemu;
#[cfg(target_os = "linux")]
mod run_ephemeral;
#[cfg(target_os = "linux")]
mod run_ephemeral_ssh;
mod ssh;
#[allow(dead_code)]
mod sshcred;
#[cfg(target_os = "linux")]
mod status_monitor;
#[cfg(target_os = "linux")]
mod supervisor_status;
#[cfg(target_os = "linux")]
pub(crate) mod systemd;
mod to_disk;
mod utils;
#[cfg(target_os = "linux")]
mod xml_utils;

/// A comprehensive toolkit for bootc containers and local virtualization.
///
/// bcvk provides a complete workflow for building, testing, and managing
/// bootc containers using ephemeral VMs. Run bootc images as temporary VMs,
/// install them to disk, or manage existing installations - all without
/// requiring root privileges.
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Execute a command in the host context from within a container.
///
/// This allows containers to run host commands with proper isolation
/// and resource management through the host execution system.
#[derive(Parser)]
struct HostExecOpts {
    /// Binary executable to run on the host system
    ///
    /// Can be a full path or a command name available in PATH.
    bin: OsString,

    /// Command-line arguments to pass to the binary
    ///
    /// All arguments after the binary name, including flags and options.
    /// Supports arguments starting with hyphens.
    #[clap(allow_hyphen_values = true)]
    args: Vec<OsString>,
}

#[derive(Parser)]
struct DebugInternalsOpts {
    #[command(subcommand)]
    command: DebugInternalsCmds,
}

#[derive(Subcommand)]
enum DebugInternalsCmds {
    OpenTree { path: std::path::PathBuf },
}

/// Internal diagnostic and tooling commands for development
#[derive(Parser)]
struct InternalsOpts {
    #[command(subcommand)]
    command: InternalsCmds,
}

#[derive(Subcommand)]
enum InternalsCmds {
    /// Dump CLI structure as JSON for man page generation
    #[cfg(feature = "docgen")]
    DumpCliJson,
}

/// Available bcvk commands for container and VM management.
#[derive(Subcommand)]
enum Commands {
    /// Execute commands on the host system from within containers
    #[clap(hide = true)]
    Hostexec(HostExecOpts),

    /// Manage and inspect bootc container images
    #[clap(subcommand)]
    Images(images::ImagesOpts),

    /// Manage ephemeral VMs for bootc containers
    #[clap(subcommand)]
    Ephemeral(ephemeral::EphemeralCommands),

    /// Install bootc images to persistent disk images
    #[clap(name = "to-disk")]
    ToDisk(to_disk::ToDiskOpts),

    /// Manage libvirt integration for bootc containers
    #[clap(subcommand)]
    #[cfg(target_os = "linux")]
    Libvirt(libvirt::LibvirtCommands),

    /// Upload bootc disk images to libvirt (deprecated)
    #[cfg(target_os = "linux")]
    #[clap(name = "libvirt-upload-disk", hide = true)]
    LibvirtUploadDisk(libvirt_upload_disk::LibvirtUploadDiskOpts),

    /// Internal container entrypoint command (hidden from help)
    #[clap(hide = true)]
    ContainerEntrypoint(container_entrypoint::ContainerEntrypointOpts),

    /// Internal debugging and diagnostic tools (hidden from help)
    #[clap(hide = true)]
    DebugInternals(DebugInternalsOpts),

    /// Internal diagnostic and tooling commands for development
    #[clap(hide = true)]
    Internals(InternalsOpts),
}

/// Install and configure the tracing/logging system.
///
/// Sets up structured logging with environment-based filtering,
/// error layer integration, and console output formatting.
/// Logs are filtered by RUST_LOG environment variable, defaulting to 'info'.
fn install_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::fmt;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::EnvFilter;

    let fmt_layer = fmt::layer().with_target(false).with_writer(std::io::stderr);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}

/// Main entry point for the bcvk CLI application.
///
/// Initializes logging, error handling, and command dispatch for all
/// bcvk operations including VM management, SSH access, and
/// container image handling.
fn main() -> Result<(), Report> {
    install_tracing();
    color_eyre::install()?;

    let cli = Cli::parse();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("Init tokio runtime")?;

    match cli.command {
        Commands::Hostexec(opts) => {
            hostexec::run(opts.bin, opts.args)?;
        }
        Commands::Images(opts) => {
            #[cfg(target_os = "linux")]
            {
                opts.run()?
            }
            #[cfg(not(target_os = "linux"))]
            {
                let _ = opts; // Suppress unused variable warning
                todo!("Images command not yet supported on macOS")
            }
        }
        Commands::Ephemeral(cmd) => cmd.run()?,
        Commands::ToDisk(opts) => {
            to_disk::run(opts)?;
        }
        #[cfg(target_os = "linux")]
        Commands::Libvirt(cmd) => {
            cmd.run()?;
        }
        #[cfg(target_os = "linux")]
        Commands::LibvirtUploadDisk(opts) => {
            eprintln!(
                "Warning: 'libvirt-upload-disk' is deprecated. Use 'libvirt upload' instead."
            );
            libvirt_upload_disk::run(opts)?;
        }
        Commands::ContainerEntrypoint(opts) => {
            // Create a tokio runtime for async container entrypoint operations
            rt.block_on(container_entrypoint::run(opts))?;
        }
        Commands::DebugInternals(opts) => match opts.command {
            DebugInternalsCmds::OpenTree { path } => {
                #[cfg(target_os = "linux")]
                {
                    let fd = rustix::mount::open_tree(
                        rustix::fs::CWD,
                        path,
                        rustix::mount::OpenTreeFlags::OPEN_TREE_CLOEXEC
                            | rustix::mount::OpenTreeFlags::OPEN_TREE_CLONE,
                    )?;
                    let fd = Dir::reopen_dir(&fd)?;
                    tracing::debug!("{:?}", fd.entries()?.into_iter().collect::<Vec<_>>());
                }
                #[cfg(not(target_os = "linux"))]
                {
                    let _ = path; // Suppress unused variable warning
                    eprintln!("OpenTree debug command is only available on Linux");
                }
            }
        },
        Commands::Internals(opts) => match opts.command {
            #[cfg(feature = "docgen")]
            InternalsCmds::DumpCliJson => {
                let json = cli_json::dump_cli_json()?;
                println!("{}", json);
            }
        },
    }
    tracing::debug!("exiting");
    std::process::exit(0)
}
