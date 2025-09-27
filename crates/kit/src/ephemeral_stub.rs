//! macOS stub for ephemeral commands

use clap::Subcommand;
use color_eyre::Result;

#[derive(Debug, Subcommand)]
pub enum EphemeralCommands {
    /// Run bootc containers as ephemeral VMs (Linux only)
    #[clap(name = "run")]
    Run {
        #[clap(help = "Container image (not supported on macOS)")]
        image: String,
    },
    /// Run ephemeral VM and SSH into it (Linux only)
    #[clap(name = "run-ssh")]
    RunSsh {
        #[clap(help = "Container image (not supported on macOS)")]
        image: String,
    },
    /// Connect to running VMs via SSH (Linux only)
    #[clap(name = "ssh")]
    Ssh {
        #[clap(help = "Container name (not supported on macOS)")]
        container_name: String,
    },
    /// List ephemeral VM containers (Linux only)
    #[clap(name = "ps")]
    Ps {
        #[clap(long, help = "Output as JSON (not supported on macOS)")]
        json: bool,
    },
    /// Remove all ephemeral VM containers (Linux only)
    #[clap(name = "rm-all")]
    RmAll {
        #[clap(short, long, help = "Force removal (not supported on macOS)")]
        force: bool,
    },
}

impl EphemeralCommands {
    pub fn run(self) -> Result<()> {
        todo!("Ephemeral VMs are not supported on macOS")
    }
}
