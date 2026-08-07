//! Loading trace and profile files with bounded, safe input handling.

use prefixity_core::error::PrefixityError;
use prefixity_core::limits;
use prefixity_core::model::{CostProfile, RequestTrace};
use std::path::Path;

/// Load and deserialize a trace file.
pub fn load_trace(path: &Path) -> Result<RequestTrace, PrefixityError> {
    let bytes = read_limited(path)?;
    serde_json::from_slice(&bytes).map_err(|source| PrefixityError::InvalidJson {
        path: path.to_path_buf(),
        source,
    })
}

/// Load, deserialize and validate a provider cost profile file.
pub fn load_cost_profile(path: &Path) -> Result<CostProfile, PrefixityError> {
    let bytes = read_limited(path)?;
    let profile: CostProfile =
        serde_json::from_slice(&bytes).map_err(|source| PrefixityError::InvalidJson {
            path: path.to_path_buf(),
            source,
        })?;
    prefixity_core::cost::validate_cost_profile(&profile).map_err(|message| {
        PrefixityError::InvalidCostProfile {
            path: path.to_path_buf(),
            message,
        }
    })?;
    Ok(profile)
}

/// Read a file, rejecting anything above the configured size limit before
/// allocating, and stripping a UTF-8 BOM if present (some Windows tooling
/// writes one; `serde_json` would otherwise reject the file).
fn read_limited(path: &Path) -> Result<Vec<u8>, PrefixityError> {
    let metadata = std::fs::metadata(path).map_err(|source| PrefixityError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if metadata.len() > limits::MAX_TRACE_FILE_BYTES {
        return Err(PrefixityError::FileTooLarge {
            path: path.to_path_buf(),
            limit: limits::MAX_TRACE_FILE_BYTES,
        });
    }
    let mut bytes = std::fs::read(path).map_err(|source| PrefixityError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        bytes.drain(0..3);
    }
    Ok(bytes)
}
