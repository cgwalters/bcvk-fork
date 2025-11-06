//! Integration tests for to-disk command
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

use camino::Utf8PathBuf;
use color_eyre::Result;
use integration_tests::{integration_test, parameterized_integration_test};

use std::process::Command;
use tempfile::TempDir;

use crate::{get_test_image, run_bcvk, CapturedOutput, INTEGRATION_TEST_LABEL};

/// Validate that a disk image was created successfully with proper bootc installation
///
/// This helper function verifies:
/// - The disk image file exists and has non-zero size
/// - The disk has valid partition table (using sfdisk, only for raw images)
/// - The installation completed successfully (from output messages)
///
/// Note: sfdisk can only read partition tables from raw disk images, not qcow2.
/// For qcow2 images, partition validation is skipped.
fn validate_disk_image(
    disk_path: &Utf8PathBuf,
    output: &CapturedOutput,
    context: &str,
) -> Result<()> {
    let metadata = std::fs::metadata(disk_path).expect("Failed to get disk metadata");
    assert!(metadata.len() > 0, "{}: Disk image is empty", context);

    // Only verify partitions for raw images - sfdisk can't read qcow2 format
    let is_qcow2 = disk_path.as_str().ends_with(".qcow2");
    if !is_qcow2 {
        // Verify the disk has partitions using sfdisk -l
        let sfdisk_output = Command::new("sfdisk")
            .arg("-l")
            .arg(disk_path.as_str())
            .output()
            .expect("Failed to run sfdisk");

        let sfdisk_stdout = String::from_utf8_lossy(&sfdisk_output.stdout);

        assert!(
            sfdisk_output.status.success(),
            "{}: sfdisk failed with exit code: {:?}",
            context,
            sfdisk_output.status.code()
        );

        assert!(
            sfdisk_stdout.contains("Disk ")
                && (sfdisk_stdout.contains("sectors") || sfdisk_stdout.contains("bytes")),
            "{}: sfdisk output doesn't show expected disk information",
            context
        );

        let has_partitions = sfdisk_stdout.lines().any(|line| {
            line.contains(disk_path.as_str()) && (line.contains("Linux") || line.contains("EFI"))
        });

        assert!(
            has_partitions,
            "{}: No bootc partitions found in sfdisk output. Output was:\n{}",
            context, sfdisk_stdout
        );
    }

    assert!(
        output.stdout.contains("Installation complete") || output.stderr.contains("Installation complete"),
        "{}: No 'Installation complete' message found in output. This indicates bootc install did not complete successfully. stdout: {}, stderr: {}",
        context,
        output.stdout, output.stderr
    );

    Ok(())
}

/// Test actual bootc installation to a disk image
fn test_to_disk() -> Result<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let disk_path = Utf8PathBuf::try_from(temp_dir.path().join("test-disk.img"))
        .expect("temp path is not UTF-8");

    let output = run_bcvk(&[
        "to-disk",
        "--label",
        INTEGRATION_TEST_LABEL,
        &get_test_image(),
        disk_path.as_str(),
    ])?;

    assert!(
        output.success(),
        "to-disk failed with exit code: {:?}. stdout: {}, stderr: {}",
        output.exit_code(),
        output.stdout,
        output.stderr
    );

    validate_disk_image(&disk_path, &output, "test_to_disk")?;
    Ok(())
}
integration_test!(test_to_disk);

/// Test bootc installation to a qcow2 disk image
fn test_to_disk_qcow2() -> Result<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let disk_path = Utf8PathBuf::try_from(temp_dir.path().join("test-disk.qcow2"))
        .expect("temp path is not UTF-8");

    let output = run_bcvk(&[
        "to-disk",
        "--format=qcow2",
        "--label",
        INTEGRATION_TEST_LABEL,
        &get_test_image(),
        disk_path.as_str(),
    ])?;

    assert!(
        output.success(),
        "to-disk with qcow2 failed with exit code: {:?}. stdout: {}, stderr: {}",
        output.exit_code(),
        output.stdout,
        output.stderr
    );

    // Verify the file is actually qcow2 format using qemu-img info
    let qemu_img_output = Command::new("qemu-img")
        .args(["info", disk_path.as_str()])
        .output()
        .expect("Failed to run qemu-img info");

    let qemu_img_stdout = String::from_utf8_lossy(&qemu_img_output.stdout);

    assert!(
        qemu_img_output.status.success(),
        "qemu-img info failed with exit code: {:?}",
        qemu_img_output.status.code()
    );

    assert!(
        qemu_img_stdout.contains("file format: qcow2"),
        "qemu-img info doesn't show qcow2 format. Output was:\n{}",
        qemu_img_stdout
    );

    validate_disk_image(&disk_path, &output, "test_to_disk_qcow2")?;
    Ok(())
}
integration_test!(test_to_disk_qcow2);

/// Test disk image caching functionality
fn test_to_disk_caching() -> Result<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let disk_path = Utf8PathBuf::try_from(temp_dir.path().join("test-disk-cache.img"))
        .expect("temp path is not UTF-8");

    // First run: Create the disk image
    let output1 = run_bcvk(&[
        "to-disk",
        "--label",
        INTEGRATION_TEST_LABEL,
        &get_test_image(),
        disk_path.as_str(),
    ])?;

    assert!(
        output1.success(),
        "First to-disk run failed with exit code: {:?}. stdout: {}, stderr: {}",
        output1.exit_code(),
        output1.stdout,
        output1.stderr
    );

    let metadata1 =
        std::fs::metadata(&disk_path).expect("Failed to get disk metadata after first run");
    assert!(metadata1.len() > 0, "Disk image is empty after first run");

    assert!(
        output1.stdout.contains("Installation complete")
            || output1.stderr.contains("Installation complete"),
        "No 'Installation complete' message found in first run output"
    );

    // Second run: Should reuse the cached disk
    let output2 = run_bcvk(&[
        "to-disk",
        "--label",
        INTEGRATION_TEST_LABEL,
        &get_test_image(),
        disk_path.as_str(),
    ])?;

    assert!(
        output2.success(),
        "Second to-disk run failed with exit code: {:?}. stdout: {}, stderr: {}",
        output2.exit_code(),
        output2.stdout,
        output2.stderr
    );

    assert!(
        output2.stdout.contains("Reusing existing cached disk image"),
        "Second run should have reused cached disk, but cache reuse message not found. stdout: {}, stderr: {}",
        output2.stdout, output2.stderr
    );

    let metadata2 =
        std::fs::metadata(&disk_path).expect("Failed to get disk metadata after second run");
    assert_eq!(
        metadata1.len(),
        metadata2.len(),
        "Disk size changed between runs, indicating it was recreated instead of reused"
    );

    assert!(
        !output2.stdout.contains("Installation complete") && !output2.stderr.contains("Installation complete"),
        "Second run should not have performed installation, but found 'Installation complete' message"
    );
    Ok(())
}
integration_test!(test_to_disk_caching);

/// Test that different image references with the same digest create separate cached disks
fn test_to_disk_different_imgref_same_digest() -> Result<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // First, pull the test image
    let test_image = get_test_image();
    let output = Command::new("podman")
        .args(["pull", &test_image])
        .output()
        .expect("Failed to run podman pull");
    assert!(
        output.status.success(),
        "Failed to pull test image: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Create a second tag pointing to the same digest
    let second_tag = format!("{}-alias", test_image);
    let output = Command::new("podman")
        .args(["tag", &test_image, &second_tag])
        .output()
        .expect("Failed to run podman tag");
    assert!(
        output.status.success(),
        "Failed to tag image: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Create first disk with original image reference
    let disk_path = Utf8PathBuf::try_from(temp_dir.path().join("test-disk.img"))
        .expect("temp path is not UTF-8");

    let output1 = run_bcvk(&[
        "to-disk",
        "--label",
        INTEGRATION_TEST_LABEL,
        &test_image,
        disk_path.as_str(),
    ])?;

    assert!(
        output1.success(),
        "First to-disk run failed with exit code: {:?}. stdout: {}, stderr: {}",
        output1.exit_code(),
        output1.stdout,
        output1.stderr
    );

    let metadata1 =
        std::fs::metadata(&disk_path).expect("Failed to get disk metadata after first run");
    assert!(metadata1.len() > 0, "Disk image is empty");

    // Use --dry-run with the aliased image reference (same digest, different imgref)
    // to verify it would regenerate instead of reusing the cache
    let output2 = run_bcvk(&[
        "to-disk",
        "--dry-run",
        "--label",
        INTEGRATION_TEST_LABEL,
        &second_tag,
        disk_path.as_str(),
    ])?;

    assert!(
        output2.success(),
        "Dry-run with different imgref failed with exit code: {:?}. stdout: {}, stderr: {}",
        output2.exit_code(),
        output2.stdout,
        output2.stderr
    );

    // The dry-run should report it would regenerate because the source imgref is different
    assert!(
        output2.stdout.contains("would-regenerate"),
        "Dry-run should report 'would-regenerate' for different imgref. stdout: {}, stderr: {}",
        output2.stdout,
        output2.stderr
    );

    // Clean up: remove the aliased tag
    let _ = Command::new("podman").args(["rmi", &second_tag]).output();

    Ok(())
}
integration_test!(test_to_disk_different_imgref_same_digest);

/// Test to-disk with various bootc images to ensure compatibility
///
/// This parameterized test runs to-disk with multiple container images,
/// particularly testing AlmaLinux which had cross-device link issues (issue #125)
fn test_to_disk_for_image(image: &str) -> Result<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let disk_path = Utf8PathBuf::try_from(temp_dir.path().join("test-disk.img"))
        .expect("temp path is not UTF-8");

    let output = run_bcvk(&[
        "to-disk",
        "--label",
        INTEGRATION_TEST_LABEL,
        // Not all iamges have one
        "--filesystem=ext4",
        image,
        disk_path.as_str(),
    ])?;

    assert!(
        output.success(),
        "to-disk with image {} failed with exit code: {:?}. stdout: {}, stderr: {}",
        image,
        output.exit_code(),
        output.stdout,
        output.stderr
    );

    validate_disk_image(
        &disk_path,
        &output,
        &format!("test_to_disk_multi_image({})", image),
    )?;
    Ok(())
}
parameterized_integration_test!(test_to_disk_for_image);
