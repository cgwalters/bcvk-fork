//! Shared library code for integration tests
//!
//! This module contains constants and utilities that are shared between
//! the main test binary and helper binaries like cleanup.

// Unfortunately needed here to work with linkme
#![allow(unsafe_code)]

use linkme::distributed_slice;

/// Label used to identify containers created by integration tests
pub const INTEGRATION_TEST_LABEL: &str = "bcvk.integration-test=1";

/// Label used to identify libvirt VMs created by integration tests
pub const LIBVIRT_INTEGRATION_TEST_LABEL: &str = "bcvk-integration";

/// A test function that returns a Result
pub type TestFn = fn() -> color_eyre::Result<()>;

/// Metadata for a registered integration test
#[derive(Debug)]
pub struct IntegrationTest {
    pub name: &'static str,
    pub f: TestFn,
}

impl IntegrationTest {
    pub const fn new(name: &'static str, f: TestFn) -> Self {
        Self { name, f }
    }
}

/// Distributed slice holding all registered integration tests
#[distributed_slice]
pub static INTEGRATION_TESTS: [IntegrationTest];
