//! Integration tests for ephemeral run-ssh command
//!
//! ⚠️  **CRITICAL INTEGRATION TEST POLICY** ⚠️
//!
//! INTEGRATION TESTS MUST NEVER "warn and continue" ON FAILURES!
//!
//! If something is not working:
//! - Use `todo!("reason why this doesn't work yet")`
//! - Use `panic!("clear error message")`
//! - Use `assert!()` and `unwrap()` to fail hard
//!
//! NEVER use patterns like:
//! - "Note: test failed - likely due to..."
//! - "This is acceptable in CI/testing environments"
//! - Warning and continuing on failures

use std::process::Command;
use std::thread;
use std::time::Duration;

use crate::{get_alternative_test_image, get_test_image, run_bcvk, INTEGRATION_TEST_LABEL};

/// Test running a non-interactive command via SSH
pub fn test_run_ephemeral_ssh_command() {
    let output = run_bcvk(&[
        "ephemeral",
        "run-ssh",
        "--label",
        INTEGRATION_TEST_LABEL,
        &get_test_image(),
        "--",
        "echo",
        "hello world from SSH",
    ])
    .expect("Failed to run bcvk ephemeral run-ssh");

    output.assert_success("ephemeral run-ssh");

    assert!(
        output.stdout.contains("hello world from SSH"),
        "Expected output not found. Got: {}",
        output.stdout
    );
}

/// Test that the container is cleaned up when SSH exits
pub fn test_run_ephemeral_ssh_cleanup() {
    let container_name = format!("test-ssh-cleanup-{}", std::process::id());

    let output = run_bcvk(&[
        "ephemeral",
        "run-ssh",
        "--name",
        &container_name,
        "--label",
        INTEGRATION_TEST_LABEL,
        &get_test_image(),
        "--",
        "echo",
        "testing cleanup",
    ])
    .expect("Failed to run bcvk ephemeral run-ssh");

    output.assert_success("ephemeral run-ssh");

    thread::sleep(Duration::from_secs(1));

    let check_output = Command::new("podman")
        .args(["ps", "-a", "--format", "{{.Names}}"])
        .output()
        .expect("Failed to list containers");

    let containers = String::from_utf8_lossy(&check_output.stdout);
    assert!(
        !containers.contains(&container_name),
        "Container {} was not cleaned up after SSH exit. Active containers: {}",
        container_name,
        containers
    );
}

/// Test running system commands via SSH
pub fn test_run_ephemeral_ssh_system_command() {
    let output = run_bcvk(&[
        "ephemeral",
        "run-ssh",
        "--label",
        INTEGRATION_TEST_LABEL,
        &get_test_image(),
        "--",
        "/bin/sh",
        "-c",
        "systemctl is-system-running || true",
    ])
    .expect("Failed to run bcvk ephemeral run-ssh");

    output.assert_success("ephemeral run-ssh");
}

/// Test that ephemeral run-ssh properly forwards exit codes
pub fn test_run_ephemeral_ssh_exit_code() {
    let output = run_bcvk(&[
        "ephemeral",
        "run-ssh",
        "--label",
        INTEGRATION_TEST_LABEL,
        &get_test_image(),
        "--",
        "exit",
        "42",
    ])
    .expect("Failed to run bcvk ephemeral run-ssh");

    let exit_code = output.exit_code().expect("Failed to get exit code");
    assert_eq!(
        exit_code, 42,
        "Exit code not properly forwarded. Expected 42, got {}",
        exit_code
    );
}

/// Test SSH functionality across different bootc images (Fedora and CentOS)
/// This test verifies that our systemd version compatibility fix works correctly
/// with both newer systemd (Fedora) and older systemd (CentOS Stream 9)
pub fn test_run_ephemeral_ssh_cross_distro_compatibility() {
    test_ssh_with_image(&get_test_image(), "primary");
    test_ssh_with_image(&get_alternative_test_image(), "alternative");
}

fn test_ssh_with_image(image: &str, image_type: &str) {
    let output = run_bcvk(&[
        "ephemeral",
        "run-ssh",
        "--label",
        INTEGRATION_TEST_LABEL,
        image,
        "--",
        "systemctl",
        "--version",
    ])
    .expect("Failed to run bcvk ephemeral run-ssh");

    assert!(
        output.success(),
        "{} image SSH test failed: {}",
        image_type,
        output.stderr
    );

    assert!(
        output.stdout.contains("systemd"),
        "{} image: systemd version not found. Got: {}",
        image_type,
        output.stdout
    );

    // Log systemd version for diagnostic purposes
    if let Some(version_line) = output.stdout.lines().next() {
        eprintln!("{} image systemd version: {}", image_type, version_line);

        let version_parts: Vec<&str> = version_line.split_whitespace().collect();
        if version_parts.len() >= 2 {
            if let Ok(version_num) = version_parts[1].parse::<u32>() {
                if version_num >= 254 {
                    eprintln!(
                        "✓ {} supports vmm.notify_socket (version {})",
                        image_type, version_num
                    );
                } else {
                    eprintln!(
                        "✓ {} falls back to SSH polling (version {} < 254)",
                        image_type, version_num
                    );
                }
            }
        }
    }
}
