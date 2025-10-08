//! Shared library code for integration tests
//!
//! This module contains constants and utilities that are shared between
//! the main test binary and helper binaries like cleanup.

/// Label used to identify containers created by integration tests
pub const INTEGRATION_TEST_LABEL: &str = "bcvk.integration-test=1";

/// Label used to identify libvirt VMs created by integration tests
pub const LIBVIRT_INTEGRATION_TEST_LABEL: &str = "bcvk-integration";
