//! libvirt list command - list bootc domains
//!
//! This module provides functionality to list libvirt domains that were
//! created from bootc container images, showing their status and metadata.

use clap::Parser;
use color_eyre::Result;

/// Options for listing libvirt domains
#[derive(Debug, Parser)]
pub struct LibvirtListOpts {
    /// Output format
    #[clap(long, default_value = "table")]
    pub format: String,

    /// Show all domains including stopped ones
    #[clap(long, short = 'a')]
    pub all: bool,

    /// Filter domains by label
    #[clap(long)]
    pub label: Option<String>,
}

/// Execute the libvirt list command
pub fn run(global_opts: &crate::libvirt::LibvirtOptions, opts: LibvirtListOpts) -> Result<()> {
    use crate::domain_list::DomainLister;
    use color_eyre::eyre::Context;

    // Use libvirt as the source of truth for domain listing
    let connect_uri = global_opts.connect.as_ref();
    let lister = match connect_uri {
        Some(uri) => DomainLister::with_connection(uri.clone()),
        None => DomainLister::new(),
    };

    let mut domains = if opts.all {
        lister
            .list_bootc_domains()
            .with_context(|| "Failed to list bootc domains from libvirt")?
    } else {
        lister
            .list_running_bootc_domains()
            .with_context(|| "Failed to list running bootc domains from libvirt")?
    };

    // Filter by label if specified
    if let Some(ref filter_label) = opts.label {
        domains.retain(|d| d.labels.contains(filter_label));
    }

    match opts.format.as_str() {
        "table" => {
            if domains.is_empty() {
                if opts.all {
                    println!("No VMs found");
                    println!("Tip: Create VMs with 'bcvk libvirt run <image>'");
                } else {
                    println!("No running VMs found");
                    println!(
                        "Use --all to see stopped VMs or 'bcvk libvirt run <image>' to create one"
                    );
                }
                return Ok(());
            }
            println!(
                "{:<20} {:<40} {:<12} {:<20}",
                "NAME", "IMAGE", "STATUS", "MEMORY"
            );
            println!("{}", "=".repeat(92));
            for domain in &domains {
                let image = match &domain.image {
                    Some(img) => {
                        if img.len() > 38 {
                            format!("{}...", &img[..35])
                        } else {
                            img.clone()
                        }
                    }
                    None => "<no metadata>".to_string(),
                };
                let memory = match domain.memory_mb {
                    Some(mem) => format!("{}MB", mem),
                    None => "unknown".to_string(),
                };
                println!(
                    "{:<20} {:<40} {:<12} {:<20}",
                    domain.name,
                    image,
                    domain.status_string(),
                    memory
                );
            }
            println!(
                "\nFound {} domain{} (source: libvirt)",
                domains.len(),
                if domains.len() == 1 { "" } else { "s" }
            );
        }
        "json" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&domains)
                    .with_context(|| "Failed to serialize domains as JSON")?
            );
        }
        _ => {
            return Err(color_eyre::eyre::eyre!(
                "Unsupported format: {}",
                opts.format
            ))
        }
    }
    Ok(())
}
