use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterializationErrorCode {
    StaleSourceRequest,
    CandidateIdentityMismatch,
    CandidateSafetyRejected,
    EvaluationMismatch,
    UnsupportedTransformation,
    ArtifactMissing,
    ArtifactDuplicated,
    ArtifactContentMismatch,
    UnexpectedToolChange,
    UnexpectedEnvelopeChange,
    UnexpectedContentChange,
    TrustProvenanceMismatch,
    PlannedActualDiffMismatch,
    StructuralReanalysisMismatch,
    CertificateInvariantFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LivePreparationErrorCode {
    NonLoopbackEndpoint,
    InvalidEndpoint,
    LiveOptInRequired,
    EndpointUnavailable,
    ConnectionTimeout,
    RequestTimeout,
    MalformedResponse,
    NormalizationConflict,
    ContextLimitRejected,
    ServerError,
    IncompleteSequence,
    EvidenceWriteFailure,
    InvalidConfiguration,
    UnsafeMaterializedCandidate,
    EvidenceStateMismatch,
    ResponseTooLarge,
}

impl std::fmt::Display for LivePreparationErrorCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::NonLoopbackEndpoint => "non_loopback_endpoint",
            Self::InvalidEndpoint => "invalid_endpoint",
            Self::LiveOptInRequired => "live_opt_in_required",
            Self::EndpointUnavailable => "endpoint_unavailable",
            Self::ConnectionTimeout => "connection_timeout",
            Self::RequestTimeout => "request_timeout",
            Self::MalformedResponse => "malformed_response",
            Self::NormalizationConflict => "normalization_conflict",
            Self::ContextLimitRejected => "context_limit_rejected",
            Self::ServerError => "server_error",
            Self::IncompleteSequence => "incomplete_sequence",
            Self::EvidenceWriteFailure => "evidence_write_failure",
            Self::InvalidConfiguration => "invalid_configuration",
            Self::UnsafeMaterializedCandidate => "unsafe_materialized_candidate",
            Self::EvidenceStateMismatch => "evidence_state_mismatch",
            Self::ResponseTooLarge => "response_too_large",
        };
        formatter.write_str(value)
    }
}

impl std::fmt::Display for MaterializationErrorCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::StaleSourceRequest => "stale_source_request",
            Self::CandidateIdentityMismatch => "candidate_identity_mismatch",
            Self::CandidateSafetyRejected => "candidate_safety_rejected",
            Self::EvaluationMismatch => "evaluation_mismatch",
            Self::UnsupportedTransformation => "unsupported_transformation",
            Self::ArtifactMissing => "artifact_missing",
            Self::ArtifactDuplicated => "artifact_duplicated",
            Self::ArtifactContentMismatch => "artifact_content_mismatch",
            Self::UnexpectedToolChange => "unexpected_tool_change",
            Self::UnexpectedEnvelopeChange => "unexpected_envelope_change",
            Self::UnexpectedContentChange => "unexpected_content_change",
            Self::TrustProvenanceMismatch => "trust_provenance_mismatch",
            Self::PlannedActualDiffMismatch => "planned_actual_diff_mismatch",
            Self::StructuralReanalysisMismatch => "structural_reanalysis_mismatch",
            Self::CertificateInvariantFailed => "certificate_invariant_failed",
        };
        formatter.write_str(value)
    }
}

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
    #[error("candidate materialization failed [{code}]: {message}")]
    Materialization {
        code: MaterializationErrorCode,
        message: String,
    },
    #[error("live experiment harness failed [{code}]: {message}")]
    LiveHarness {
        code: LivePreparationErrorCode,
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

    pub fn materialization(code: MaterializationErrorCode, message: impl Into<String>) -> Self {
        Self::Materialization {
            code,
            message: message.into(),
        }
    }

    pub fn live_harness(code: LivePreparationErrorCode, message: impl Into<String>) -> Self {
        Self::LiveHarness {
            code,
            message: message.into(),
        }
    }
}
