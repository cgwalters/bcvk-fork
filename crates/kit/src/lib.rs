// Re-export modules for conditional compilation in binary and tests

pub const CONTAINER_STATEDIR: &str = "/var/lib/bcvk";
pub mod arch;
#[cfg(target_os = "linux")]
pub mod boot_progress;
pub mod cli_json;
pub mod cmdext;
pub mod cmdfdext;
pub mod common_opts;
pub mod container_entrypoint;
#[cfg(target_os = "linux")]
pub(crate) mod containerenv;
#[cfg(target_os = "linux")]
pub mod domain_list;
#[cfg(target_os = "linux")]
pub mod envdetect;
#[cfg(target_os = "linux")]
pub mod ephemeral;
#[cfg(not(target_os = "linux"))]
#[path = "ephemeral_stub.rs"]
pub mod ephemeral;
pub mod hostexec;
pub mod images;
pub mod install_options;
#[cfg(target_os = "linux")]
pub mod libvirt;
#[cfg(target_os = "linux")]
pub mod libvirt_upload_disk;
#[allow(dead_code)]
pub mod podman;
#[cfg(target_os = "linux")]
#[allow(dead_code)]
pub mod qemu;
#[cfg(target_os = "linux")]
pub mod run_ephemeral;
#[cfg(target_os = "linux")]
pub mod run_ephemeral_ssh;
pub mod ssh;
#[allow(dead_code)]
pub mod sshcred;
#[cfg(target_os = "linux")]
pub mod status_monitor;
#[cfg(target_os = "linux")]
pub mod supervisor_status;
#[cfg(target_os = "linux")]
pub(crate) mod systemd;
pub mod to_disk;
pub mod utils;
#[cfg(target_os = "linux")]
pub mod xml_utils;
