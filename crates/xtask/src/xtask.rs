//! See https://github.com/matklad/cargo-xtask
//! This is kind of like "Justfile but in Rust".

use std::process::Command;

use color_eyre::eyre::{eyre, Context, Report};
use color_eyre::Result;
use xshell::Shell;

mod man;

#[allow(clippy::type_complexity)]
const TASKS: &[(&str, fn(&Shell) -> Result<()>)] = &[
    ("manpages", manpages),
    ("update-manpages", update_manpages),
    ("sync-manpages", sync_manpages),
    ("package", package),
];

fn install_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::fmt;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::EnvFilter;

    let fmt_layer = fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}

fn main() -> Result<(), Report> {
    install_tracing();
    color_eyre::install()?;
    // Ensure our working directory is the toplevel
    {
        let toplevel_path = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .context("Invoking git rev-parse")?;
        if !toplevel_path.status.success() {
            return Err(eyre!("Failed to invoke git rev-parse"));
        }
        let path = String::from_utf8(toplevel_path.stdout)?;
        std::env::set_current_dir(path.trim()).context("Changing to toplevel")?;
    }

    let task = std::env::args().nth(1);

    let sh = xshell::Shell::new()?;
    if let Some(cmd) = task.as_deref() {
        let f = TASKS
            .iter()
            .find_map(|(k, f)| (*k == cmd).then_some(*f))
            .unwrap_or(print_help);
        f(&sh)?;
    } else {
        print_help(&sh)?;
    }
    Ok(())
}

fn print_help(_sh: &Shell) -> Result<()> {
    println!("Tasks:");
    for (name, _) in TASKS {
        println!("  - {name}");
    }
    Ok(())
}

fn manpages(sh: &Shell) -> Result<()> {
    man::generate_man_pages(sh)
}

fn update_manpages(sh: &Shell) -> Result<()> {
    man::update_manpages(sh)
}

fn sync_manpages(sh: &Shell) -> Result<()> {
    man::sync_all_man_pages(sh)
}

fn package(sh: &Shell) -> Result<()> {
    use std::env;
    use xshell::cmd;

    // Get version from Cargo.toml
    let version = man::get_raw_package_version()?;

    println!("Creating release archives for version {}", version);

    // Get the git commit timestamp for reproducible builds
    let source_date_epoch = cmd!(sh, "git log -1 --format=%ct").read()?;
    env::set_var("SOURCE_DATE_EPOCH", source_date_epoch.trim());

    // Create target directory if it doesn't exist
    sh.create_dir("target")?;

    // Create temporary directory for intermediate files
    let tempdir = tempfile::tempdir()?;
    let temp_tar = tempdir.path().join(format!("bcvk-{}.tar", version));

    // Create source archive using git archive (uncompressed initially)
    let source_archive = format!("target/bcvk-{}.tar.zstd", version);
    cmd!(
        sh,
        "git archive --format=tar --prefix=bcvk-{version}/ HEAD -o {temp_tar}"
    )
    .run()?;

    // Create vendor archive
    let vendor_archive = format!("target/bcvk-{}-vendor.tar.zstd", version);
    cmd!(
        sh,
        "cargo vendor-filterer --format=tar.zstd {vendor_archive}"
    )
    .run()?;

    println!("Created vendor archive: {}", vendor_archive);

    // Create vendor config for the source archive
    let vendor_config_content = r#"[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
"#;
    let vendor_config_path = tempdir.path().join(".cargo-vendor-config.toml");
    std::fs::write(&vendor_config_path, vendor_config_content)?;

    // Add vendor config to source archive
    cmd!(sh, "tar --owner=0 --group=0 --numeric-owner --sort=name --mtime=@{source_date_epoch} -rf {temp_tar} --transform='s|.*/.cargo-vendor-config.toml|bcvk-{version}/.cargo/vendor-config.toml|' {vendor_config_path}").run()?;

    // Compress the final source archive
    cmd!(sh, "zstd {temp_tar} -f -o {source_archive}").run()?;

    println!("Created source archive: {}", source_archive);

    println!("Release archives created successfully:");
    println!("  Source: {}", source_archive);
    println!("  Vendor: {}", vendor_archive);

    Ok(())
}
