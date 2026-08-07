//! Shared helpers for integration tests. Loads fixtures and provider profiles
//! from the repository by resolving paths relative to `CARGO_MANIFEST_DIR`.
//!
//! Each test file is compiled as its own crate and may use only a subset of
//! these helpers, hence `allow(dead_code)`.

#![allow(dead_code)]

use prefixity_core::model::{CostProfile, RequestTrace};
use std::path::{Path, PathBuf};

/// Absolute path to the workspace root.
pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Absolute path to a fixture trace file.
pub fn fixture_path(name: &str) -> PathBuf {
    workspace_root().join("fixtures/traces").join(name)
}

/// Load and parse a fixture trace.
pub fn load_fixture(name: &str) -> RequestTrace {
    let path = fixture_path(name);
    let data = std::fs::read(&path).unwrap_or_else(|e| panic!("failed to read {path:?}: {e}"));
    serde_json::from_slice(&data).unwrap_or_else(|e| panic!("failed to parse {path:?}: {e}"))
}

/// Absolute path to a provider profile file.
pub fn profile_path(name: &str) -> PathBuf {
    workspace_root().join("provider-profiles").join(name)
}

/// Load and parse a provider profile.
pub fn load_profile(name: &str) -> CostProfile {
    let path = profile_path(name);
    let data = std::fs::read(&path).unwrap_or_else(|e| panic!("failed to read {path:?}: {e}"));
    serde_json::from_slice(&data).unwrap_or_else(|e| panic!("failed to parse {path:?}: {e}"))
}
