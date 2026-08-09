use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum BenchmarkError {
    #[error("I/O error reading {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid controlled benchmark JSON in {path}: {source}")]
    InvalidJson {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("controlled benchmark validation failed at {path}: {message}")]
    Validation { path: String, message: String },
    #[error("controlled benchmark pair validation failed for {scenario_id}: {message}")]
    PairValidation {
        scenario_id: String,
        message: String,
    },
    #[error("controlled benchmark hash mismatch for {what}: expected {expected}, found {found}")]
    HashMismatch {
        what: String,
        expected: String,
        found: String,
    },
    #[error("scripted world could not execute {scenario_id}: {message}")]
    World {
        scenario_id: String,
        message: String,
    },
}

impl BenchmarkError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            path: "<in-memory>".to_string(),
            message: message.into(),
        }
    }

    pub fn pair(scenario_id: &str, message: impl Into<String>) -> Self {
        Self::PairValidation {
            scenario_id: scenario_id.to_string(),
            message: message.into(),
        }
    }
}
