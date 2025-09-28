//! Cache metadata management for bootc disk images
//!
//! This module provides functionality for storing and retrieving metadata about bootc disk images
//! using extended attributes (xattrs). This enables efficient caching by allowing bcvk to detect
//! when a disk image can be reused instead of regenerating it.
//!
//! The cache system stores two separate xattrs:
//! - A SHA256 hash of all build inputs for cache validation
//! - The container image digest for visibility and tracking

use cap_std_ext::cap_std::{self, fs::Dir};
use cap_std_ext::dirext::CapStdExtDirExt;
use color_eyre::{eyre::Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;

/// Extended attribute name for storing bootc cache hash
const BOOTC_CACHE_HASH_XATTR: &str = "user.bootc.cache_hash";

/// Extended attribute name for storing container image digest
const BOOTC_IMAGE_DIGEST_XATTR: &str = "user.bootc.image_digest";

/// Build inputs used to generate a cache hash
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheInputs {
    /// SHA256 digest of the source container image
    image_digest: String,

    /// Filesystem type used for installation (e.g., "ext4", "xfs", "btrfs")
    filesystem: Option<String>,

    /// Root filesystem size if specified
    root_size: Option<String>,

    /// Kernel arguments used during installation
    kernel_args: Vec<String>,

    /// Version of the cache format for future compatibility
    version: u32,
}

/// Metadata stored on disk images for caching purposes
#[derive(Debug, Clone)]
pub struct DiskImageMetadata {
    /// SHA256 digest of the source container image
    pub digest: String,

    /// Filesystem type used for installation (e.g., "ext4", "xfs", "btrfs")
    pub filesystem: Option<String>,

    /// Root filesystem size if specified
    pub root_size: Option<String>,

    /// Kernel arguments used during installation
    pub kernel_args: Vec<String>,

    /// Version of the metadata format for future compatibility
    pub version: u32,
}

impl DiskImageMetadata {
    /// Create new metadata for a disk image
    pub fn new(digest: &str) -> Self {
        Self {
            version: 1,
            digest: digest.to_owned(),
            filesystem: None,
            root_size: None,
            kernel_args: Default::default(),
        }
    }

    /// Generate SHA256 hash of all build inputs
    fn compute_cache_hash(&self) -> String {
        let inputs = CacheInputs {
            image_digest: self.digest.clone(),
            filesystem: self.filesystem.clone(),
            root_size: self.root_size.clone(),
            kernel_args: self.kernel_args.clone(),
            version: self.version,
        };

        let json = serde_json::to_string(&inputs).expect("Failed to serialize cache inputs");
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        format!("sha256:{:x}", hasher.finalize())
    }

    /// Write metadata to a file using extended attributes via rustix
    pub fn write_to_file(&self, file: &File) -> Result<()> {
        // Write the cache hash
        let cache_hash = self.compute_cache_hash();
        rustix::fs::fsetxattr(
            file,
            BOOTC_CACHE_HASH_XATTR,
            cache_hash.as_bytes(),
            rustix::fs::XattrFlags::empty(),
        )
        .with_context(|| "Failed to set cache hash xattr")?;

        // Write the image digest separately for visibility
        rustix::fs::fsetxattr(
            file,
            BOOTC_IMAGE_DIGEST_XATTR,
            self.digest.as_bytes(),
            rustix::fs::XattrFlags::empty(),
        )
        .with_context(|| "Failed to set image digest xattr")?;

        tracing::debug!(
            "Wrote cache hash {} and image digest {} to disk image",
            cache_hash,
            self.digest
        );
        Ok(())
    }

    /// Read image digest from a file path using extended attributes
    pub fn read_image_digest_from_path(path: &Path) -> Result<Option<String>> {
        // First check if file exists
        if !path.exists() {
            return Ok(None);
        }

        // Get the parent directory and file name
        let parent = path
            .parent()
            .ok_or_else(|| color_eyre::eyre::eyre!("Path has no parent directory"))?;
        let file_name = path
            .file_name()
            .ok_or_else(|| color_eyre::eyre::eyre!("Path has no file name"))?;

        // Open the parent directory with cap-std
        let dir = Dir::open_ambient_dir(parent, cap_std::ambient_authority())
            .with_context(|| format!("Failed to open directory {:?}", parent))?;

        // Get the image digest xattr
        let digest_data = match dir.getxattr(file_name, OsStr::new(BOOTC_IMAGE_DIGEST_XATTR))? {
            Some(data) => data,
            None => {
                tracing::debug!("No image digest xattr found on {:?}", path);
                return Ok(None);
            }
        };

        let digest = std::str::from_utf8(&digest_data)
            .with_context(|| "Invalid UTF-8 in image digest xattr")?;

        tracing::debug!("Read image digest from {:?}: {}", path, digest);
        Ok(Some(digest.to_string()))
    }
}

/// Check if a cached disk image can be reused by comparing cache hashes
pub fn check_cached_disk(
    path: &Path,
    image_digest: &str,
    filesystem: Option<&str>,
    root_size: Option<&str>,
    kernel_args: &[String],
) -> Result<bool> {
    if !path.exists() {
        tracing::debug!("Disk image {:?} does not exist", path);
        return Ok(false);
    }

    // Create metadata for the current request to compute expected hash
    let mut expected_meta = DiskImageMetadata::new(image_digest);
    expected_meta.filesystem = filesystem.map(ToOwned::to_owned);
    expected_meta.root_size = root_size.map(ToOwned::to_owned);
    expected_meta.kernel_args = Vec::from(kernel_args);
    let expected_hash = expected_meta.compute_cache_hash();

    // Read the cache hash from the disk image
    let parent = path
        .parent()
        .ok_or_else(|| color_eyre::eyre::eyre!("Path has no parent directory"))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| color_eyre::eyre::eyre!("Path has no file name"))?;

    let dir = Dir::open_ambient_dir(parent, cap_std::ambient_authority())
        .with_context(|| format!("Failed to open directory {:?}", parent))?;

    let cached_hash = match dir.getxattr(file_name, OsStr::new(BOOTC_CACHE_HASH_XATTR))? {
        Some(data) => std::str::from_utf8(&data)
            .with_context(|| "Invalid UTF-8 in cache hash xattr")?
            .to_string(),
        None => {
            tracing::debug!("No cache hash xattr found on {:?}", path);
            return Ok(false);
        }
    };

    let matches = expected_hash == cached_hash;
    if matches {
        tracing::info!(
            "Found cached disk image at {:?} matching cache hash {}",
            path,
            expected_hash
        );
    } else {
        tracing::debug!(
            "Cached disk at {:?} does not match requirements. \
             Expected hash: {}, found: {}",
            path,
            expected_hash,
            cached_hash
        );
    }

    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_hash_generation() {
        let mut metadata1 = DiskImageMetadata::new("sha256:abc123");
        metadata1.filesystem = Some("ext4".to_string());
        metadata1.root_size = Some("20G".to_string());
        metadata1.kernel_args = vec!["console=ttyS0".to_string()];

        let mut metadata2 = DiskImageMetadata::new("sha256:abc123");
        metadata2.filesystem = Some("ext4".to_string());
        metadata2.root_size = Some("20G".to_string());
        metadata2.kernel_args = vec!["console=ttyS0".to_string()];

        // Same inputs should generate same hash
        assert_eq!(
            metadata1.compute_cache_hash(),
            metadata2.compute_cache_hash()
        );

        // Different inputs should generate different hashes
        let mut metadata3 = DiskImageMetadata::new("sha256:xyz789");
        metadata3.filesystem = Some("ext4".to_string());
        metadata3.root_size = Some("20G".to_string());
        metadata3.kernel_args = vec!["console=ttyS0".to_string()];

        assert_ne!(
            metadata1.compute_cache_hash(),
            metadata3.compute_cache_hash()
        );

        // Different filesystem should generate different hash
        let mut metadata4 = DiskImageMetadata::new("sha256:abc123");
        metadata4.filesystem = Some("xfs".to_string());
        metadata4.root_size = Some("20G".to_string());
        metadata4.kernel_args = vec!["console=ttyS0".to_string()];

        assert_ne!(
            metadata1.compute_cache_hash(),
            metadata4.compute_cache_hash()
        );
    }

    #[test]
    fn test_cache_inputs_serialization() -> Result<()> {
        let inputs = CacheInputs {
            image_digest: "sha256:abc123".to_string(),
            filesystem: Some("ext4".to_string()),
            root_size: Some("20G".to_string()),
            kernel_args: vec!["console=ttyS0".to_string()],
            version: 1,
        };

        let json = serde_json::to_string(&inputs)?;
        let deserialized: CacheInputs = serde_json::from_str(&json)?;

        assert_eq!(inputs.image_digest, deserialized.image_digest);
        assert_eq!(inputs.filesystem, deserialized.filesystem);
        assert_eq!(inputs.root_size, deserialized.root_size);
        assert_eq!(inputs.kernel_args, deserialized.kernel_args);
        assert_eq!(inputs.version, deserialized.version);
        Ok(())
    }
}
