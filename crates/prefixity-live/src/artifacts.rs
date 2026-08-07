//! Sanitized artifact writing.
//!
//! Artifacts live under `experiments/runs/<sanitized-experiment-id>/`.
//! Experiment ids are validated so they cannot escape the runs directory.
//! Credentials never appear in any artifact (the whole pipeline keeps them
//! out of every serialized value).

use crate::error::LiveError;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Maximum length of an experiment id.
pub const MAX_EXPERIMENT_ID_LEN: usize = 64;

/// Validate and sanitize an experiment id: non-empty, at most 64 chars, and
/// only ASCII alphanumerics, `-` and `_`. Path separators and `..` are
/// rejected.
pub fn sanitize_experiment_id(id: &str) -> Result<String, LiveError> {
    if id.is_empty() {
        return Err(LiveError::argument("experiment id must not be empty"));
    }
    if id.len() > MAX_EXPERIMENT_ID_LEN {
        return Err(LiveError::argument(format!(
            "experiment id exceeds maximum length ({MAX_EXPERIMENT_ID_LEN})"
        )));
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(LiveError::argument(
            "experiment id may only contain ASCII letters, digits, '-' and '_'",
        ));
    }
    if id.contains("..") || id.starts_with('.') {
        return Err(LiveError::argument(
            "experiment id may not contain '..' or start with '.'",
        ));
    }
    Ok(id.to_string())
}

/// The artifact directory for an experiment, guaranteed to be a child of
/// `runs_dir`.
pub fn artifact_dir(runs_dir: &Path, experiment_id: &str) -> Result<PathBuf, LiveError> {
    let id = sanitize_experiment_id(experiment_id)?;
    let dir = runs_dir.join(&id);
    if !dir.starts_with(runs_dir) {
        return Err(LiveError::guard("artifact path escapes the runs directory"));
    }
    Ok(dir)
}

/// Write a JSON value to `path` (pretty-printed), creating parent
/// directories as needed.
pub fn write_json(path: &Path, value: &impl Serialize) -> Result<(), LiveError> {
    let json = serde_json::to_string_pretty(value).map_err(|e| LiveError::Artifact {
        path: path.to_path_buf(),
        message: format!("serialization failed: {e}"),
    })?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| LiveError::Artifact {
            path: parent.to_path_buf(),
            message: e.to_string(),
        })?;
    }
    std::fs::write(path, json).map_err(|e| LiveError::Artifact {
        path: path.to_path_buf(),
        message: e.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn temp_dir(tag: &str) -> PathBuf {
        let unique = format!(
            "prefixity-live-artifacts-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let dir = env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn experiment_id_sanitization() {
        assert_eq!(
            sanitize_experiment_id("openai-stable-prefix-42").unwrap(),
            "openai-stable-prefix-42"
        );
        assert_eq!(sanitize_experiment_id("A_B-c1").unwrap(), "A_B-c1");
        assert!(sanitize_experiment_id("").is_err());
        assert!(sanitize_experiment_id("has space").is_err());
        assert!(sanitize_experiment_id("a/b").is_err());
        assert!(sanitize_experiment_id("a\\b").is_err());
        assert!(sanitize_experiment_id("..").is_err());
        assert!(sanitize_experiment_id("..\\.\\.\\etc").is_err());
        assert!(sanitize_experiment_id(&"x".repeat(65)).is_err());
    }

    #[test]
    fn artifact_dir_stays_within_runs_dir() {
        let runs = temp_dir("within");
        let dir = artifact_dir(&runs, "exp-1").unwrap();
        assert!(dir.starts_with(&runs));
        assert_eq!(dir, runs.join("exp-1"));
        // A hostile id must be rejected before path building.
        assert!(artifact_dir(&runs, "../escape").is_err());
        std::fs::remove_dir_all(&runs).ok();
    }

    #[test]
    fn write_json_creates_parent_dirs() {
        let runs = temp_dir("write");
        let dir = artifact_dir(&runs, "exp-1").unwrap();
        let path = dir.join("manifest.json");
        write_json(&path, &serde_json::json!({"a": 1})).unwrap();
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("\"a\": 1"));
        std::fs::remove_dir_all(&runs).ok();
    }
}
