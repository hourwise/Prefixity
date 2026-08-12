//! Neutral observation and runtime-capability contracts.
//!
//! These versioned serde types describe context identity, observed cache and
//! inference accounting, and runtime capability evidence. They are deliberately
//! observation-only: they do not rewrite requests, route cache work, or claim
//! that a runtime capability is safe merely because it exists.

use crate::{error::PrefixityError, hash, model::EvidenceProvenance};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Version of the neutral ContextArtifact contract.
pub const CONTEXT_ARTIFACT_SCHEMA_VERSION: u32 = 1;
/// Version of the neutral CacheObservation contract.
pub const CACHE_OBSERVATION_SCHEMA_VERSION: u32 = 1;
/// Version of the neutral RuntimeCacheCapabilities contract.
pub const RUNTIME_CACHE_CAPABILITIES_SCHEMA_VERSION: u32 = 1;

const MAX_IDENTIFIER_BYTES: usize = 512;
const MAX_ARTIFACT_REFERENCES: usize = 100_000;
const MAX_RAW_FIELDS: usize = 64;
const MAX_RAW_BYTES: usize = 16 * 1024;

/// A value whose absence is meaningful to an observation contract.
///
/// Unknown means the value cannot currently be established. NotObserved means
/// the recorder did not attempt to observe it. Neither state is a fabricated
/// negative result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(tag = "state", content = "value", rename_all = "snake_case")]
pub enum Observed<T> {
    /// The value was established by the source or observation.
    Known(T),
    /// The value is not established.
    Unknown,
    /// The value was not collected by this observation.
    #[default]
    NotObserved,
}

/// Extensible artifact kinds. Other prevents the first schema version from
/// becoming text-only while retaining a stable representation for known kinds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "name", rename_all = "snake_case")]
pub enum ArtifactType {
    Text,
    SourceFile,
    ToolSchema,
    ToolResult,
    Image,
    Video,
    ReasoningState,
    Unknown,
    Other(String),
}

/// Stability is separate from lifecycle. A transient artifact may still be
/// stable for the duration of one request, and a persistent artifact may be
/// volatile across versions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactStability {
    Immutable,
    Stable,
    AppendOnly,
    Volatile,
    Unknown,
}

/// Lifecycle is separate from change frequency.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactLifecycle {
    PersistentVersioned,
    Transient,
    Unknown,
}

/// Cacheability of a logical artifact, independent of one runtime's cache
/// state vocabulary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Cacheability {
    Cacheable,
    NotCacheable,
    #[default]
    Unknown,
}

/// Trust metadata where an upstream architecture supplies it. This is not a
/// policy decision and must not be inferred from cache reuse.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    Trusted,
    Untrusted,
    Mixed,
    Unknown,
}

/// A token count carries its accounting scope because token units are not
/// universal across providers, models, protocols, or tokenizers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TokenCount {
    pub count: u64,
    #[serde(default)]
    pub provider: Observed<String>,
    #[serde(default)]
    pub model: Observed<String>,
    #[serde(default)]
    pub tokenizer: Observed<String>,
}

/// Size/accounting fields for one logical artifact.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ArtifactSizes {
    #[serde(default)]
    pub byte_size: Observed<u64>,
    #[serde(default)]
    pub logical_size: Observed<u64>,
    #[serde(default)]
    pub token_size: Observed<TokenCount>,
}

/// Cache/materialisation information that remains neutral across runtimes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ArtifactCacheState {
    #[serde(default)]
    pub cacheability: Cacheability,
    #[serde(default)]
    pub materialized: Observed<MaterializationState>,
    /// Backend-defined residency label, such as device, host_ram, or disk.
    /// It is not interpreted as one universal runtime state machine.
    #[serde(default)]
    pub residency: Observed<String>,
}

/// Generic materialisation states used only when a backend exposes them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MaterializationState {
    Materialized,
    NotMaterialized,
}

/// A logical context artifact independent of provider-specific rendering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextArtifact {
    pub schema_version: u32,
    pub artifact_id: String,
    /// Stable logical origin identity.
    pub origin_id: String,
    /// Concrete source of the model-visible representation, when distinct
    /// from origin_id.
    #[serde(default)]
    pub content_source_id: Observed<String>,
    #[serde(default)]
    pub content_hash: Observed<String>,
    #[serde(default)]
    pub revision: Observed<String>,
    pub artifact_type: ArtifactType,
    pub stability: ArtifactStability,
    pub lifecycle: ArtifactLifecycle,
    #[serde(default)]
    pub sizes: ArtifactSizes,
    #[serde(default)]
    pub cache: ArtifactCacheState,
    #[serde(default)]
    pub trust: Observed<TrustLevel>,
    /// Existing field-level provenance vocabulary is reused rather than
    /// introducing a parallel provenance system.
    #[serde(default)]
    pub provenance: BTreeMap<String, EvidenceProvenance>,
    #[serde(default)]
    pub metadata: BTreeMap<String, serde_json::Value>,
}

impl ContextArtifact {
    /// Validate the structural invariants of this versioned artifact record.
    pub fn validate(&self) -> Result<(), PrefixityError> {
        validate_version(
            "ContextArtifact",
            self.schema_version,
            CONTEXT_ARTIFACT_SCHEMA_VERSION,
        )?;
        validate_identifier("artifact_id", &self.artifact_id)?;
        validate_identifier("origin_id", &self.origin_id)?;
        validate_observed_string("content_source_id", &self.content_source_id)?;
        validate_observed_string("revision", &self.revision)?;
        if let Observed::Known(value) = &self.content_hash {
            if !hash::is_valid_sha256_hex(value) {
                return invalid("content_hash must be a 64-character lowercase SHA-256 digest");
            }
        }
        if let ArtifactType::Other(name) = &self.artifact_type {
            validate_identifier("artifact_type.name", name)?;
        }
        validate_token_count("sizes.token_size", &self.sizes.token_size)?;
        validate_observed_string("cache.residency", &self.cache.residency)?;
        validate_metadata("metadata", &self.metadata)
    }
}

/// A reference to an artifact in an observed request context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArtifactReference {
    pub artifact_id: String,
    #[serde(default)]
    pub content_hash: Observed<String>,
}

/// Context identity fields observed or calculated for one request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ContextIdentity {
    #[serde(default)]
    pub artifacts: Vec<ArtifactReference>,
    #[serde(default)]
    pub serialized_request_identity: Observed<String>,
    #[serde(default)]
    pub reusable_prefix_identity: Observed<String>,
}

/// Runtime/provider identity. backend is required; other dimensions may be
/// explicitly unknown because provider/model/protocol/runtime combinations can
/// expose different cache semantics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RuntimeIdentity {
    pub backend: String,
    #[serde(default)]
    pub provider: Observed<String>,
    #[serde(default)]
    pub model: Observed<String>,
    #[serde(default)]
    pub protocol: Observed<String>,
    #[serde(default)]
    pub runtime: Observed<String>,
    #[serde(default)]
    pub runtime_version: Observed<String>,
    #[serde(default)]
    pub session_id: Observed<String>,
    #[serde(default)]
    pub slot_id: Observed<String>,
    #[serde(default)]
    pub conversation_id: Observed<String>,
}

impl RuntimeIdentity {
    fn validate(&self, field: &str) -> Result<(), PrefixityError> {
        validate_identifier(&format!("{field}.backend"), &self.backend)?;
        for (name, value) in [
            ("provider", &self.provider),
            ("model", &self.model),
            ("protocol", &self.protocol),
            ("runtime", &self.runtime),
            ("runtime_version", &self.runtime_version),
            ("session_id", &self.session_id),
            ("slot_id", &self.slot_id),
            ("conversation_id", &self.conversation_id),
        ] {
            validate_observed_string(&format!("{field}.{name}"), value)?;
        }
        Ok(())
    }
}

/// Token accounting fields intentionally remain distinct.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TokenAccounting {
    #[serde(default)]
    pub transmitted_input_tokens: Observed<TokenCount>,
    #[serde(default)]
    pub provider_cached_tokens: Observed<TokenCount>,
    #[serde(default)]
    pub fresh_prefill_tokens: Observed<TokenCount>,
    #[serde(default)]
    pub reconstructed_context_tokens: Observed<TokenCount>,
    #[serde(default)]
    pub output_tokens: Observed<TokenCount>,
}

/// Timing fields are optional and preserve partial backend observability.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TimingObservation {
    #[serde(default)]
    pub prefill_duration_ms: Observed<u64>,
    #[serde(default)]
    pub time_to_first_token_ms: Observed<u64>,
    #[serde(default)]
    pub generation_duration_ms: Observed<u64>,
    #[serde(default)]
    pub wall_duration_ms: Observed<u64>,
}

/// Resource measurements are absent when the backend cannot expose them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ResourceUsage {
    #[serde(default)]
    pub ram_bytes: Observed<u64>,
    #[serde(default)]
    pub vram_bytes: Observed<u64>,
    #[serde(default)]
    pub kv_cache_bytes: Observed<u64>,
    #[serde(default)]
    pub other_bytes: BTreeMap<String, Observed<u64>>,
}

/// Cache temperature exposed by an observation, not inferred from token math.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CacheTemperature {
    Cold,
    Warm,
}

/// Runtime-neutral cache materialisation events.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CacheMaterialization {
    Resident,
    Restored,
    Rebuilt,
}

/// Cache behaviour observed for one request/run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CacheBehavior {
    #[serde(default)]
    pub temperature: Observed<CacheTemperature>,
    #[serde(default)]
    pub materialization: Observed<CacheMaterialization>,
    #[serde(default)]
    pub restore_duration_ms: Observed<u64>,
    #[serde(default)]
    pub rebuild_duration_ms: Observed<u64>,
    #[serde(default)]
    pub cache_read: Observed<bool>,
    #[serde(default)]
    pub cache_write: Observed<bool>,
    #[serde(default)]
    pub cache_hit: Observed<bool>,
    #[serde(default)]
    pub cache_miss: Observed<bool>,
}

/// Neutral task/result outcome association. This is not a benchmark score.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskResultStatus {
    Succeeded,
    Failed,
    Partial,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ObservationOutcome {
    #[serde(default)]
    pub status: Observed<TaskResultStatus>,
    #[serde(default)]
    pub task_result_reference: Observed<String>,
    #[serde(default)]
    pub quality_reference: Observed<String>,
}

/// One observed inference request/run. It records what was observable without
/// requiring every backend to expose every accounting, timing, or cache field.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CacheObservation {
    pub schema_version: u32,
    pub observation_id: String,
    pub observed_at: String,
    pub runtime: RuntimeIdentity,
    #[serde(default)]
    pub context: ContextIdentity,
    #[serde(default)]
    pub accounting: TokenAccounting,
    #[serde(default)]
    pub timing: TimingObservation,
    #[serde(default)]
    pub resources: ResourceUsage,
    #[serde(default)]
    pub cache: CacheBehavior,
    #[serde(default)]
    pub outcome: ObservationOutcome,
    /// Bounded adapter-native telemetry kept separate from normalized fields.
    #[serde(default)]
    pub raw_telemetry: BTreeMap<String, serde_json::Value>,
}

impl CacheObservation {
    /// Validate the structural invariants of this observation record.
    pub fn validate(&self) -> Result<(), PrefixityError> {
        validate_version(
            "CacheObservation",
            self.schema_version,
            CACHE_OBSERVATION_SCHEMA_VERSION,
        )?;
        validate_identifier("observation_id", &self.observation_id)?;
        validate_identifier("observed_at", &self.observed_at)?;
        self.runtime.validate("runtime")?;
        if self.context.artifacts.len() > MAX_ARTIFACT_REFERENCES {
            return invalid("context.artifacts exceeds the bounded reference count");
        }
        for reference in &self.context.artifacts {
            validate_identifier("context.artifact_id", &reference.artifact_id)?;
            if let Observed::Known(value) = &reference.content_hash {
                if !hash::is_valid_sha256_hex(value) {
                    return invalid("context artifact content_hash is not a SHA-256 digest");
                }
            }
        }
        validate_observed_string(
            "context.serialized_request_identity",
            &self.context.serialized_request_identity,
        )?;
        validate_observed_string(
            "context.reusable_prefix_identity",
            &self.context.reusable_prefix_identity,
        )?;
        validate_token_accounting(&self.accounting)?;
        validate_observed_string(
            "outcome.task_result_reference",
            &self.outcome.task_result_reference,
        )?;
        validate_observed_string("outcome.quality_reference", &self.outcome.quality_reference)?;
        validate_metadata("raw_telemetry", &self.raw_telemetry)
    }
}

/// Whether a capability is supported, unsupported, or not established.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CapabilitySupport {
    Supported,
    Unsupported,
    #[default]
    Unknown,
}

/// Evidence state for a capability claim.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityEvidence {
    Documented,
    ExperimentallyObserved,
    #[default]
    Unverified,
}

/// A capability claim whose evidence state prevents undocumented absence from
/// being encoded as false.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Capability<T> {
    #[serde(default)]
    pub support: CapabilitySupport,
    #[serde(default)]
    pub evidence: CapabilityEvidence,
    #[serde(default)]
    pub details: Observed<T>,
}

impl<T> Default for Capability<T> {
    fn default() -> Self {
        Self {
            support: CapabilitySupport::Unknown,
            evidence: CapabilityEvidence::Unverified,
            details: Observed::NotObserved,
        }
    }
}

impl<T> Capability<T> {
    fn validate(&self, field: &str) -> Result<(), PrefixityError> {
        if self.support != CapabilitySupport::Unknown
            && self.evidence == CapabilityEvidence::Unverified
        {
            return invalid(format!(
                "{field} cannot claim support status with unverified evidence"
            ));
        }
        if self.support == CapabilitySupport::Unknown
            && self.evidence == CapabilityEvidence::ExperimentallyObserved
        {
            return invalid(format!(
                "{field} cannot be experimentally observed while support remains unknown"
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PrefixCacheCapabilities {
    #[serde(default)]
    pub prefix_reuse: Capability<bool>,
    #[serde(default)]
    pub automatic_cache_population: Capability<bool>,
    #[serde(default)]
    pub explicit_cache_population: Capability<bool>,
    #[serde(default)]
    pub minimum_cacheable_prefix_tokens: Capability<TokenCount>,
    #[serde(default)]
    pub cache_unit: Observed<String>,
    #[serde(default)]
    pub matching_constraints: Observed<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ResidencyCapabilities {
    #[serde(default)]
    pub device_kv_state: Capability<bool>,
    #[serde(default)]
    pub host_ram_caching: Capability<bool>,
    #[serde(default)]
    pub disk_persistence: Capability<bool>,
    #[serde(default)]
    pub explicit_save_restore: Capability<bool>,
    #[serde(default)]
    pub checkpoint_support: Capability<bool>,
    #[serde(default)]
    pub idle_slot_session_caching: Capability<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SessionCapabilities {
    #[serde(default)]
    pub slots_or_sessions: Capability<bool>,
    #[serde(default)]
    pub cache_affinity_routing_hints: Capability<bool>,
    #[serde(default)]
    pub retention_or_ttl_controls: Capability<bool>,
    #[serde(default)]
    pub conversation_chaining: Capability<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MetricCapabilities {
    #[serde(default)]
    pub input_tokens: Capability<bool>,
    #[serde(default)]
    pub cached_tokens: Capability<bool>,
    #[serde(default)]
    pub prompt_evaluation_duration: Capability<bool>,
    #[serde(default)]
    pub time_to_first_token: Capability<bool>,
    #[serde(default)]
    pub cache_state: Capability<bool>,
    #[serde(default)]
    pub ram_usage: Capability<bool>,
    #[serde(default)]
    pub vram_usage: Capability<bool>,
    #[serde(default)]
    pub kv_usage: Capability<bool>,
    #[serde(default)]
    pub restore_or_rebuild: Capability<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct KvCacheCapabilities {
    #[serde(default)]
    pub precision_options: Observed<Vec<String>>,
    #[serde(default)]
    pub quantization_support: Capability<bool>,
    /// Evidence that a precision option preserves task quality is deliberately
    /// separate from support for the option itself.
    #[serde(default)]
    pub quality_regression_evidence: Observed<String>,
}

/// Versioned capability description for one provider/model/protocol/runtime
/// combination. Every unsupported claim carries explicit evidence state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuntimeCacheCapabilities {
    pub schema_version: u32,
    pub identity: RuntimeIdentity,
    #[serde(default)]
    pub prefix_cache: PrefixCacheCapabilities,
    #[serde(default)]
    pub residency: ResidencyCapabilities,
    #[serde(default)]
    pub sessions: SessionCapabilities,
    #[serde(default)]
    pub metrics: MetricCapabilities,
    #[serde(default)]
    pub kv_cache: KvCacheCapabilities,
    /// Bounded backend-specific capability evidence not yet normalized.
    #[serde(default)]
    pub raw_capabilities: BTreeMap<String, serde_json::Value>,
}

impl RuntimeCacheCapabilities {
    /// Validate the structural and evidence-state invariants.
    pub fn validate(&self) -> Result<(), PrefixityError> {
        validate_version(
            "RuntimeCacheCapabilities",
            self.schema_version,
            RUNTIME_CACHE_CAPABILITIES_SCHEMA_VERSION,
        )?;
        self.identity.validate("identity")?;
        validate_capability_group(&self.prefix_cache)?;
        validate_capability_group(&self.residency)?;
        validate_capability_group(&self.sessions)?;
        validate_capability_group(&self.metrics)?;
        validate_capability_group(&self.kv_cache)?;
        validate_observed_string("prefix_cache.cache_unit", &self.prefix_cache.cache_unit)?;
        validate_observed_string(
            "kv_cache.quality_regression_evidence",
            &self.kv_cache.quality_regression_evidence,
        )?;
        validate_metadata("raw_capabilities", &self.raw_capabilities)
    }
}

fn validate_capability_group<T>(group: &T) -> Result<(), PrefixityError>
where
    T: CapabilityGroup,
{
    group.validate()
}

trait CapabilityGroup {
    fn validate(&self) -> Result<(), PrefixityError>;
}

impl CapabilityGroup for PrefixCacheCapabilities {
    fn validate(&self) -> Result<(), PrefixityError> {
        self.prefix_reuse.validate("prefix_cache.prefix_reuse")?;
        self.automatic_cache_population
            .validate("prefix_cache.automatic_cache_population")?;
        self.explicit_cache_population
            .validate("prefix_cache.explicit_cache_population")?;
        self.minimum_cacheable_prefix_tokens
            .validate("prefix_cache.minimum_cacheable_prefix_tokens")?;
        Ok(())
    }
}

impl CapabilityGroup for ResidencyCapabilities {
    fn validate(&self) -> Result<(), PrefixityError> {
        self.device_kv_state.validate("residency.device_kv_state")?;
        self.host_ram_caching
            .validate("residency.host_ram_caching")?;
        self.disk_persistence
            .validate("residency.disk_persistence")?;
        self.explicit_save_restore
            .validate("residency.explicit_save_restore")?;
        self.checkpoint_support
            .validate("residency.checkpoint_support")?;
        self.idle_slot_session_caching
            .validate("residency.idle_slot_session_caching")?;
        Ok(())
    }
}

impl CapabilityGroup for SessionCapabilities {
    fn validate(&self) -> Result<(), PrefixityError> {
        self.slots_or_sessions
            .validate("sessions.slots_or_sessions")?;
        self.cache_affinity_routing_hints
            .validate("sessions.cache_affinity_routing_hints")?;
        self.retention_or_ttl_controls
            .validate("sessions.retention_or_ttl_controls")?;
        self.conversation_chaining
            .validate("sessions.conversation_chaining")?;
        Ok(())
    }
}

impl CapabilityGroup for MetricCapabilities {
    fn validate(&self) -> Result<(), PrefixityError> {
        self.input_tokens.validate("metrics.input_tokens")?;
        self.cached_tokens.validate("metrics.cached_tokens")?;
        self.prompt_evaluation_duration
            .validate("metrics.prompt_evaluation_duration")?;
        self.time_to_first_token
            .validate("metrics.time_to_first_token")?;
        self.cache_state.validate("metrics.cache_state")?;
        self.ram_usage.validate("metrics.ram_usage")?;
        self.vram_usage.validate("metrics.vram_usage")?;
        self.kv_usage.validate("metrics.kv_usage")?;
        self.restore_or_rebuild
            .validate("metrics.restore_or_rebuild")?;
        Ok(())
    }
}

impl CapabilityGroup for KvCacheCapabilities {
    fn validate(&self) -> Result<(), PrefixityError> {
        self.quantization_support
            .validate("kv_cache.quantization_support")?;
        Ok(())
    }
}

fn invalid(message: impl Into<String>) -> Result<(), PrefixityError> {
    Err(PrefixityError::validation("<in-memory>", message))
}

fn validate_version(name: &str, found: u32, expected: u32) -> Result<(), PrefixityError> {
    if found != expected {
        return invalid(format!(
            "{name} schema_version {found} is unsupported (expected {expected})"
        ));
    }
    Ok(())
}

fn validate_identifier(name: &str, value: &str) -> Result<(), PrefixityError> {
    if value.trim().is_empty() {
        return invalid(format!("{name} must not be empty"));
    }
    if value.len() > MAX_IDENTIFIER_BYTES {
        return invalid(format!("{name} exceeds {MAX_IDENTIFIER_BYTES} bytes"));
    }
    Ok(())
}

fn validate_observed_string(name: &str, value: &Observed<String>) -> Result<(), PrefixityError> {
    if let Observed::Known(value) = value {
        validate_identifier(name, value)?;
    }
    Ok(())
}

fn validate_token_count(name: &str, value: &Observed<TokenCount>) -> Result<(), PrefixityError> {
    if let Observed::Known(value) = value {
        for (dimension, identity) in [
            ("provider", &value.provider),
            ("model", &value.model),
            ("tokenizer", &value.tokenizer),
        ] {
            validate_observed_string(&format!("{name}.{dimension}"), identity)?;
        }
    }
    Ok(())
}

fn validate_token_accounting(value: &TokenAccounting) -> Result<(), PrefixityError> {
    for (name, count) in [
        (
            "accounting.transmitted_input_tokens",
            &value.transmitted_input_tokens,
        ),
        (
            "accounting.provider_cached_tokens",
            &value.provider_cached_tokens,
        ),
        (
            "accounting.fresh_prefill_tokens",
            &value.fresh_prefill_tokens,
        ),
        (
            "accounting.reconstructed_context_tokens",
            &value.reconstructed_context_tokens,
        ),
        ("accounting.output_tokens", &value.output_tokens),
    ] {
        validate_token_count(name, count)?;
    }
    Ok(())
}

fn validate_metadata(
    name: &str,
    values: &BTreeMap<String, serde_json::Value>,
) -> Result<(), PrefixityError> {
    if values.len() > MAX_RAW_FIELDS {
        return invalid(format!("{name} exceeds {MAX_RAW_FIELDS} fields"));
    }
    for key in values.keys() {
        validate_identifier(&format!("{name} key"), key)?;
    }
    let encoded = serde_json::to_vec(values)
        .map_err(|error| PrefixityError::validation("<in-memory>", error.to_string()))?;
    if encoded.len() > MAX_RAW_BYTES {
        return invalid(format!("{name} exceeds {MAX_RAW_BYTES} bytes"));
    }
    Ok(())
}
