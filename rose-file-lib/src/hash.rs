use std::{cmp::Ordering, path::Path};

use arrayvec::ArrayString;
use serde::{Deserialize, Serialize};

use crate::{error::RoseLibError, io::normalize_path};

pub const ROSE_HASH_LEN: usize = blake3::OUT_LEN;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoseHash(blake3::Hash);

impl RoseHash {
    #[inline]
    pub const fn as_bytes(&self) -> &[u8; ROSE_HASH_LEN] {
        self.0.as_bytes()
    }

    pub const fn from_bytes(bytes: [u8; ROSE_HASH_LEN]) -> Self {
        RoseHash(blake3::Hash::from_bytes(bytes))
    }

    pub fn to_hex(&self) -> ArrayString<{ 2 * ROSE_HASH_LEN }> {
        self.0.to_hex()
    }

    pub fn from_hex(hex: impl AsRef<[u8]>) -> Result<Self, RoseLibError> {
        blake3::Hash::from_hex(hex)
            .map(RoseHash)
            .map_err(|e| RoseLibError::Generic(e.to_string()))
    }

    /// Hash a file path
    ///
    /// NOTE: This normalizes the path before hashing (e.g. convert lower case,
    /// convert slash, etc.)
    pub fn from_path(path: &Path) -> Self {
        let s = normalize_path(path).unwrap_or_default();
        RoseHash(blake3::hash(s.as_bytes()))
    }
}

impl From<[u8; ROSE_HASH_LEN]> for RoseHash {
    #[inline]
    fn from(bytes: [u8; ROSE_HASH_LEN]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<&Path> for RoseHash {
    #[inline]
    fn from(path: &Path) -> Self {
        Self::from_path(path)
    }
}

impl From<RoseHash> for [u8; ROSE_HASH_LEN] {
    #[inline]
    fn from(hash: RoseHash) -> Self {
        *hash.0.as_bytes()
    }
}

impl core::str::FromStr for RoseHash {
    type Err = RoseLibError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        RoseHash::from_hex(s)
    }
}

impl Default for RoseHash {
    fn default() -> Self {
        RoseHash(blake3::Hash::from_bytes([0; 32]))
    }
}

impl PartialOrd for RoseHash {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RoseHash {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_bytes().cmp(other.as_bytes())
    }
}
