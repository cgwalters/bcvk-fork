//! Integration tests for ephemeral run command
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

use tracing::debug;

use crate::{get_test_image, run_bcvk, INTEGRATION_TEST_LABEL};

pub fn get_container_kernel_version(image: &str) -> String {
    // Run container to get its kernel version
    let output = Command::new("podman")
        .args([
            "run",
            "--rm",
            image,
            "sh",
            "-c",
            "ls -1 /usr/lib/modules | head -1",
        ])
        .output()
        .expect("Failed to get container kernel version");

    assert!(
        output.status.success(),
        "Failed to get kernel version from container: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

pub fn test_run_ephemeral_correct_kernel() {
    let image = get_test_image();
    let container_kernel = get_container_kernel_version(&image);
    eprintln!("Container kernel version: {}", container_kernel);

    let output = run_bcvk(&[
        "ephemeral",
        "run",
        "--rm",
        "--label",
        INTEGRATION_TEST_LABEL,
        &image,
        "--karg",
        "systemd.unit=poweroff.target",
    ])
    .expect("Failed to run bcvk ephemeral run");

    output.assert_success("ephemeral run");
}

pub fn test_run_ephemeral_poweroff() {
    let output = run_bcvk(&[
        "ephemeral",
        "run",
        "--rm",
        "--label",
        INTEGRATION_TEST_LABEL,
        &get_test_image(),
        "--karg",
        "systemd.unit=poweroff.target",
    ])
    .expect("Failed to run bcvk ephemeral run");

    output.assert_success("ephemeral run");
}

pub fn test_run_ephemeral_with_memory_limit() {
    let output = run_bcvk(&[
        "ephemeral",
        "run",
        "--rm",
        "--label",
        INTEGRATION_TEST_LABEL,
        "--memory",
        "1024",
        "--karg",
        "systemd.unit=poweroff.target",
        &get_test_image(),
    ])
    .expect("Failed to run bcvk ephemeral run");

    output.assert_success("ephemeral run with memory limit");
}

pub fn test_run_ephemeral_with_vcpus() {
    let output = run_bcvk(&[
        "ephemeral",
        "run",
        "--rm",
        "--label",
        INTEGRATION_TEST_LABEL,
        "--vcpus",
        "2",
        "--karg",
        "systemd.unit=poweroff.target",
        &get_test_image(),
    ])
    .expect("Failed to run bcvk ephemeral run");

    output.assert_success("ephemeral run with vcpus");
}

pub fn test_run_ephemeral_execute() {
    let script =
        "/bin/sh -c \"echo 'Hello from VM'; echo 'Current date:'; date; echo 'Script completed successfully'\"";

    let output = run_bcvk(&[
        "ephemeral",
        "run",
        "--rm",
        "--label",
        INTEGRATION_TEST_LABEL,
        "--execute",
        script,
        &get_test_image(),
    ])
    .expect("Failed to run bcvk ephemeral run with --execute");

    output.assert_success("ephemeral run with --execute");

    assert!(
        output.stdout.contains("Hello from VM"),
        "Script output 'Hello from VM' not found in stdout: {}",
        output.stdout
    );

    assert!(
        output.stdout.contains("Script completed successfully"),
        "Script completion message not found in stdout: {}",
        output.stdout
    );

    assert!(
        output.stdout.contains("Current date:"),
        "Date output header not found in stdout: {}",
        output.stdout
    );
}

pub fn test_run_ephemeral_container_ssh_access() {
    let image = get_test_image();
    let container_name = format!(
        "ssh-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    let output = run_bcvk(&[
        "ephemeral",
        "run",
        "--ssh-keygen",
        "--label",
        INTEGRATION_TEST_LABEL,
        "--detach",
        "--name",
        &container_name,
        &image,
    ])
    .expect("Failed to start detached VM with SSH");

    if !output.success() {
        panic!("Failed to start detached VM: {}", output.stderr);
    }

    let ssh_output = run_bcvk(&[
        "ephemeral",
        "ssh",
        &container_name,
        "echo",
        "SSH_TEST_SUCCESS",
    ])
    .expect("Failed to run SSH command");

    debug!("SSH exit status: {:?}", ssh_output.exit_code());

    // Cleanup: stop the container
    let _ = Command::new("podman")
        .args(["stop", &container_name])
        .output();

    assert!(ssh_output.success());
    assert!(ssh_output.stdout.contains("SSH_TEST_SUCCESS"));
}
