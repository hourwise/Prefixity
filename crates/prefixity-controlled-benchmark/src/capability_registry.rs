//! Deterministic, evidence-aware registry and matrix for runtime capabilities.
//!
//! The registry consumes the neutral `RuntimeCacheCapabilities` contract from
//! `prefixity-core`.  It adds profile identity, bounded provenance, typed
//! queries, matrix cells, and research-gap counts without creating a second
//! capability vocabulary or treating unknown as unsupported.

use crate::error::BenchmarkError;
use crate::hashing::canonical_hash;
use prefixity_core::observation::{
    Capability, CapabilityEvidence, CapabilitySupport, Observed, RuntimeCacheCapabilities,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const CAPABILITY_REGISTRY_SCHEMA_ID: &str = "prefixity.capability-registry";
pub const CAPABILITY_REGISTRY_SCHEMA_VERSION: u32 = 1;
pub const MAX_REGISTRY_PROFILES: usize = 128;
pub const MAX_REGISTRY_PROVENANCE: usize = 16;
pub const MAX_REGISTRY_TEXT_BYTES: usize = 512;

pub const APPROVED_CAPABILITY_FIXTURE_PATHS: [&str; 8] = [
    "fixtures/capabilities/alibaba-model-studio.json",
    "fixtures/capabilities/deepseek.json",
    "fixtures/capabilities/llama-cpp-documented-v1.json",
    "fixtures/capabilities/llama-cpp.json",
    "fixtures/capabilities/meta.json",
    "fixtures/capabilities/mistral.json",
    "fixtures/capabilities/ollama.json",
    "fixtures/capabilities/z-ai-glm.json",
];

const CAPABILITY_KEYS: [CapabilityKey; 28] = [
    CapabilityKey::PrefixReuse,
    CapabilityKey::AutomaticCachePopulation,
    CapabilityKey::ExplicitCachePopulation,
    CapabilityKey::MinimumCacheablePrefixTokens,
    CapabilityKey::CacheUnit,
    CapabilityKey::MatchingConstraints,
    CapabilityKey::DeviceKvState,
    CapabilityKey::HostRamCaching,
    CapabilityKey::DiskPersistence,
    CapabilityKey::ExplicitSaveRestore,
    CapabilityKey::CheckpointSupport,
    CapabilityKey::IdleSlotSessionCaching,
    CapabilityKey::SlotsOrSessions,
    CapabilityKey::CacheAffinityRoutingHints,
    CapabilityKey::RetentionOrTtlControls,
    CapabilityKey::ConversationChaining,
    CapabilityKey::InputTokens,
    CapabilityKey::CachedTokens,
    CapabilityKey::PromptEvaluationDuration,
    CapabilityKey::TimeToFirstToken,
    CapabilityKey::CacheState,
    CapabilityKey::RamUsage,
    CapabilityKey::VramUsage,
    CapabilityKey::KvUsage,
    CapabilityKey::RestoreOrRebuild,
    CapabilityKey::PrecisionOptions,
    CapabilityKey::QuantizationSupport,
    CapabilityKey::QualityRegressionEvidence,
];

/// The provenance class of a registry profile, separate from each capability
/// claim's existing documented/observed/unverified evidence state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RegistryEvidenceOrigin {
    ProjectDocumentation,
    ExperimentalObservation,
    SyntheticFixture,
    #[default]
    UnknownUnverified,
}

/// One approved capability profile plus deterministic identity and bounded
/// ingestion provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityProfile {
    pub profile_id: String,
    pub capabilities: RuntimeCacheCapabilities,
    pub origin: RegistryEvidenceOrigin,
    #[serde(default)]
    pub provenance: std::collections::BTreeMap<String, String>,
}

impl CapabilityProfile {
    pub fn from_capabilities(
        capabilities: RuntimeCacheCapabilities,
        origin: RegistryEvidenceOrigin,
        provenance: std::collections::BTreeMap<String, String>,
    ) -> Result<Self, BenchmarkError> {
        capabilities
            .validate()
            .map_err(|error| BenchmarkError::validation(format!("capability profile: {error}")))?;
        let profile_id = profile_fingerprint(&capabilities)?;
        let profile = Self {
            profile_id,
            capabilities,
            origin,
            provenance,
        };
        profile.validate()?;
        Ok(profile)
    }

    pub fn validate(&self) -> Result<(), BenchmarkError> {
        validate_text(&self.profile_id, "profile_id")?;
        if self.provenance.len() > MAX_REGISTRY_PROVENANCE {
            return Err(BenchmarkError::validation(
                "capability profile provenance exceeds its bound",
            ));
        }
        validate_provenance(&self.provenance, "capability profile provenance")?;
        self.capabilities
            .validate()
            .map_err(|error| BenchmarkError::validation(format!("capability profile: {error}")))?;
        validate_boolean_capabilities(&self.capabilities)?;
        let expected = profile_fingerprint(&self.capabilities)?;
        if self.profile_id != expected {
            return Err(BenchmarkError::validation(
                "capability profile_id does not match its semantic profile fingerprint",
            ));
        }
        Ok(())
    }

    pub fn capability(&self, key: CapabilityKey) -> CapabilityCell {
        capability_cell(&self.profile_id, key, &self.capabilities)
    }
}

/// Versioned registry of independently identifiable provider/model/protocol/
/// runtime capability profiles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityRegistry {
    pub schema_id: String,
    pub registry_version: u32,
    pub profiles: Vec<CapabilityProfile>,
    #[serde(default)]
    pub provenance: std::collections::BTreeMap<String, String>,
}

impl CapabilityRegistry {
    pub fn from_profiles(
        profiles: Vec<CapabilityProfile>,
        provenance: std::collections::BTreeMap<String, String>,
    ) -> Result<Self, BenchmarkError> {
        let mut registry = Self {
            schema_id: CAPABILITY_REGISTRY_SCHEMA_ID.to_string(),
            registry_version: CAPABILITY_REGISTRY_SCHEMA_VERSION,
            profiles,
            provenance,
        };
        registry
            .profiles
            .sort_by(|left, right| left.profile_id.cmp(&right.profile_id));
        registry.validate()?;
        Ok(registry)
    }

    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if self.schema_id != CAPABILITY_REGISTRY_SCHEMA_ID {
            return Err(BenchmarkError::validation(
                "capability registry schema_id is unsupported",
            ));
        }
        if self.registry_version != CAPABILITY_REGISTRY_SCHEMA_VERSION {
            return Err(BenchmarkError::validation(
                "capability registry version is unsupported",
            ));
        }
        if self.profiles.is_empty() || self.profiles.len() > MAX_REGISTRY_PROFILES {
            return Err(BenchmarkError::validation(
                "capability registry must contain a bounded non-empty profile list",
            ));
        }
        if self.provenance.len() > MAX_REGISTRY_PROVENANCE {
            return Err(BenchmarkError::validation(
                "capability registry provenance exceeds its bound",
            ));
        }
        validate_provenance(&self.provenance, "capability registry provenance")?;
        let mut ids = BTreeSet::new();
        let mut semantic_profiles = BTreeSet::new();
        for profile in &self.profiles {
            profile.validate()?;
            if !ids.insert(profile.profile_id.clone()) {
                return Err(BenchmarkError::validation(
                    "capability registry contains duplicate profile identities",
                ));
            }
            let semantic_id = profile_fingerprint(&profile.capabilities)?;
            if !semantic_profiles.insert(semantic_id) {
                return Err(BenchmarkError::validation(
                    "capability registry contains duplicate semantic capability profiles",
                ));
            }
        }
        Ok(())
    }

    pub fn all_profiles(&self) -> &[CapabilityProfile] {
        &self.profiles
    }

    pub fn query(&self, query: &CapabilityQuery) -> Vec<&CapabilityProfile> {
        self.profiles
            .iter()
            .filter(|profile| query.matches(profile))
            .collect()
    }

    pub fn matrix(
        &self,
        query: &CapabilityQuery,
        selected_capabilities: &[CapabilityKey],
    ) -> CapabilityMatrix {
        let profiles = self.query(query);
        let mut capabilities = selected_capabilities.to_vec();
        if capabilities.is_empty() {
            capabilities.extend(CAPABILITY_KEYS);
        }
        capabilities.sort();
        capabilities.dedup();
        let rows = capabilities
            .iter()
            .copied()
            .map(|capability| CapabilityMatrixRow {
                capability,
                cells: profiles
                    .iter()
                    .map(|profile| profile.capability(capability))
                    .collect(),
            })
            .collect();
        CapabilityMatrix {
            registry_version: self.registry_version,
            profile_ids: profiles
                .iter()
                .map(|profile| profile.profile_id.clone())
                .collect(),
            capabilities,
            rows,
        }
    }

    pub fn gap_report(&self) -> ResearchGapReport {
        let capability_gaps = CAPABILITY_KEYS
            .iter()
            .copied()
            .map(|capability| {
                let cells = self
                    .profiles
                    .iter()
                    .map(|profile| profile.capability(capability));
                let mut gap = CapabilityGap {
                    capability,
                    known_profiles: 0,
                    unknown_profiles: 0,
                    experimentally_observed_profiles: 0,
                };
                for cell in cells {
                    if cell.support == CapabilitySupport::Unknown {
                        gap.unknown_profiles += 1;
                    } else {
                        gap.known_profiles += 1;
                    }
                    if cell.evidence == CapabilityEvidence::ExperimentallyObserved {
                        gap.experimentally_observed_profiles += 1;
                    }
                }
                gap
            })
            .collect();
        let profile_gaps = self
            .profiles
            .iter()
            .map(|profile| {
                let cells = CAPABILITY_KEYS
                    .iter()
                    .copied()
                    .map(|capability| profile.capability(capability));
                let mut gap = ProfileGap {
                    profile_id: profile.profile_id.clone(),
                    known_capability_fields: 0,
                    unknown_capability_fields: 0,
                    experimentally_observed_fields: 0,
                };
                for cell in cells {
                    if cell.support == CapabilitySupport::Unknown {
                        gap.unknown_capability_fields += 1;
                    } else {
                        gap.known_capability_fields += 1;
                    }
                    if cell.evidence == CapabilityEvidence::ExperimentallyObserved {
                        gap.experimentally_observed_fields += 1;
                    }
                }
                gap
            })
            .collect();
        ResearchGapReport {
            capability_gaps,
            profile_gaps,
        }
    }
}

/// Typed filters. A missing identity dimension never acts as a wildcard for
/// a known query value; only an explicit known value can match.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityQuery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability: Option<CapabilityKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support: Option<CapabilitySupport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<CapabilityEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<RegistryEvidenceOrigin>,
}

impl CapabilityQuery {
    fn matches(&self, profile: &CapabilityProfile) -> bool {
        let identity = &profile.capabilities.identity;
        matches_string(&identity.provider, &self.provider)
            && matches_string(&identity.model, &self.model)
            && matches_string(&identity.protocol, &self.protocol)
            && matches_string(&identity.runtime, &self.runtime)
            && matches_string(&identity.runtime_version, &self.runtime_version)
            && self
                .origin
                .as_ref()
                .is_none_or(|origin| profile.origin == *origin)
            && self.matches_capability(profile)
    }

    fn matches_capability(&self, profile: &CapabilityProfile) -> bool {
        if self.capability.is_none() && self.support.is_none() && self.evidence.is_none() {
            return true;
        }
        CAPABILITY_KEYS.iter().copied().any(|key| {
            if self.capability.is_some_and(|expected| expected != key) {
                return false;
            }
            let cell = profile.capability(key);
            self.support
                .as_ref()
                .is_none_or(|support| *support == cell.support)
                && self
                    .evidence
                    .as_ref()
                    .is_none_or(|evidence| *evidence == cell.evidence)
        })
    }
}

/// Every normalized capability dimension already represented by P0-L2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityKey {
    PrefixReuse,
    AutomaticCachePopulation,
    ExplicitCachePopulation,
    MinimumCacheablePrefixTokens,
    CacheUnit,
    MatchingConstraints,
    DeviceKvState,
    HostRamCaching,
    DiskPersistence,
    ExplicitSaveRestore,
    CheckpointSupport,
    IdleSlotSessionCaching,
    SlotsOrSessions,
    CacheAffinityRoutingHints,
    RetentionOrTtlControls,
    ConversationChaining,
    InputTokens,
    CachedTokens,
    PromptEvaluationDuration,
    TimeToFirstToken,
    CacheState,
    RamUsage,
    VramUsage,
    KvUsage,
    RestoreOrRebuild,
    PrecisionOptions,
    QuantizationSupport,
    QualityRegressionEvidence,
}

impl CapabilityKey {
    pub fn all() -> &'static [CapabilityKey; 28] {
        &CAPABILITY_KEYS
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::PrefixReuse => "prefix_reuse",
            Self::AutomaticCachePopulation => "automatic_cache_population",
            Self::ExplicitCachePopulation => "explicit_cache_population",
            Self::MinimumCacheablePrefixTokens => "minimum_cacheable_prefix_tokens",
            Self::CacheUnit => "cache_unit",
            Self::MatchingConstraints => "matching_constraints",
            Self::DeviceKvState => "device_kv_state",
            Self::HostRamCaching => "host_ram_caching",
            Self::DiskPersistence => "disk_persistence",
            Self::ExplicitSaveRestore => "explicit_save_restore",
            Self::CheckpointSupport => "checkpoint_support",
            Self::IdleSlotSessionCaching => "idle_slot_session_caching",
            Self::SlotsOrSessions => "slots_or_sessions",
            Self::CacheAffinityRoutingHints => "cache_affinity_routing_hints",
            Self::RetentionOrTtlControls => "retention_or_ttl_controls",
            Self::ConversationChaining => "conversation_chaining",
            Self::InputTokens => "input_tokens",
            Self::CachedTokens => "cached_tokens",
            Self::PromptEvaluationDuration => "prompt_evaluation_duration",
            Self::TimeToFirstToken => "time_to_first_token",
            Self::CacheState => "cache_state",
            Self::RamUsage => "ram_usage",
            Self::VramUsage => "vram_usage",
            Self::KvUsage => "kv_usage",
            Self::RestoreOrRebuild => "restore_or_rebuild",
            Self::PrecisionOptions => "precision_options",
            Self::QuantizationSupport => "quantization_support",
            Self::QualityRegressionEvidence => "quality_regression_evidence",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityState {
    SupportedDocumented,
    SupportedObserved,
    UnsupportedDocumented,
    UnsupportedObserved,
    UnknownDocumented,
    UnknownUnverified,
}

impl CapabilityState {
    fn from_parts(support: CapabilitySupport, evidence: CapabilityEvidence) -> Self {
        match (support, evidence) {
            (CapabilitySupport::Supported, CapabilityEvidence::Documented) => {
                Self::SupportedDocumented
            }
            (CapabilitySupport::Supported, CapabilityEvidence::ExperimentallyObserved) => {
                Self::SupportedObserved
            }
            (CapabilitySupport::Unsupported, CapabilityEvidence::Documented) => {
                Self::UnsupportedDocumented
            }
            (CapabilitySupport::Unsupported, CapabilityEvidence::ExperimentallyObserved) => {
                Self::UnsupportedObserved
            }
            (CapabilitySupport::Unknown, CapabilityEvidence::Documented) => Self::UnknownDocumented,
            (CapabilitySupport::Unknown, CapabilityEvidence::Unverified)
            | (CapabilitySupport::Supported, CapabilityEvidence::Unverified)
            | (CapabilitySupport::Unsupported, CapabilityEvidence::Unverified)
            | (CapabilitySupport::Unknown, CapabilityEvidence::ExperimentallyObserved) => {
                Self::UnknownUnverified
            }
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::SupportedDocumented => "supported_documented",
            Self::SupportedObserved => "supported_observed",
            Self::UnsupportedDocumented => "unsupported_documented",
            Self::UnsupportedObserved => "unsupported_observed",
            Self::UnknownDocumented => "unknown_documented",
            Self::UnknownUnverified => "unknown_unverified",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityCell {
    pub profile_id: String,
    pub capability: CapabilityKey,
    pub state: CapabilityState,
    pub support: CapabilitySupport,
    pub evidence: CapabilityEvidence,
    pub details: Observed<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityMatrix {
    pub registry_version: u32,
    pub profile_ids: Vec<String>,
    pub capabilities: Vec<CapabilityKey>,
    pub rows: Vec<CapabilityMatrixRow>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityMatrixRow {
    pub capability: CapabilityKey,
    pub cells: Vec<CapabilityCell>,
}

impl CapabilityMatrix {
    /// Render only structured state labels; no ranking, score, or terminal
    /// width is involved.
    pub fn render_markdown(&self) -> String {
        let mut output = String::from("| capability |");
        for profile_id in &self.profile_ids {
            output.push(' ');
            output.push_str(profile_id);
            output.push_str(" |");
        }
        output.push('\n');
        output.push_str("| --- |");
        for _ in &self.profile_ids {
            output.push_str(" --- |");
        }
        output.push('\n');
        for row in &self.rows {
            output.push('|');
            output.push_str(row.capability.label());
            output.push('|');
            for cell in &row.cells {
                output.push(' ');
                output.push_str(cell.state.label());
                output.push_str(" |");
            }
            output.push('\n');
        }
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityGap {
    pub capability: CapabilityKey,
    pub known_profiles: usize,
    pub unknown_profiles: usize,
    pub experimentally_observed_profiles: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileGap {
    pub profile_id: String,
    pub known_capability_fields: usize,
    pub unknown_capability_fields: usize,
    pub experimentally_observed_fields: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchGapReport {
    pub capability_gaps: Vec<CapabilityGap>,
    pub profile_gaps: Vec<ProfileGap>,
}

pub fn load_approved_capability_registry(
    repository_root: &Path,
) -> Result<CapabilityRegistry, BenchmarkError> {
    let profiles = APPROVED_CAPABILITY_FIXTURE_PATHS
        .iter()
        .map(|relative| {
            load_capability_profile_from_path(
                &repository_root.join(relative),
                relative,
                RegistryEvidenceOrigin::SyntheticFixture,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    CapabilityRegistry::from_profiles(
        profiles,
        std::collections::BTreeMap::from([
            (
                "source".to_string(),
                "approved-prefixity-fixtures".to_string(),
            ),
            ("network_access".to_string(), "none".to_string()),
        ]),
    )
}

pub fn load_capability_registry_from_paths(
    paths: &[PathBuf],
) -> Result<CapabilityRegistry, BenchmarkError> {
    let profiles = paths
        .iter()
        .map(|path| {
            load_capability_profile_from_path(
                path,
                &path.display().to_string(),
                RegistryEvidenceOrigin::SyntheticFixture,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    CapabilityRegistry::from_profiles(profiles, std::collections::BTreeMap::new())
}

fn load_capability_profile_from_path(
    path: &Path,
    source_name: &str,
    origin: RegistryEvidenceOrigin,
) -> Result<CapabilityProfile, BenchmarkError> {
    let bytes = std::fs::read(path).map_err(|source| BenchmarkError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if bytes.len() > 16 * 1024 {
        return Err(BenchmarkError::validation(format!(
            "capability fixture {source_name} exceeds 16 KiB"
        )));
    }
    let capabilities: RuntimeCacheCapabilities =
        serde_json::from_slice(&bytes).map_err(|source| BenchmarkError::InvalidJson {
            path: path.to_path_buf(),
            source,
        })?;
    CapabilityProfile::from_capabilities(
        capabilities,
        origin,
        std::collections::BTreeMap::from([("fixture_path".to_string(), source_name.to_string())]),
    )
}

fn profile_fingerprint(capabilities: &RuntimeCacheCapabilities) -> Result<String, BenchmarkError> {
    canonical_hash(capabilities).map_err(|error| BenchmarkError::validation(error.to_string()))
}

fn validate_text(value: &str, field: &str) -> Result<(), BenchmarkError> {
    if value.trim().is_empty() || value.len() > MAX_REGISTRY_TEXT_BYTES {
        return Err(BenchmarkError::validation(format!(
            "{field} must be non-empty and at most {MAX_REGISTRY_TEXT_BYTES} bytes"
        )));
    }
    Ok(())
}

fn validate_provenance(
    provenance: &std::collections::BTreeMap<String, String>,
    field: &str,
) -> Result<(), BenchmarkError> {
    for (key, value) in provenance {
        validate_text(key, &format!("{field} key"))?;
        validate_text(value, &format!("{field} value"))?;
    }
    Ok(())
}

fn validate_boolean_capabilities(
    capabilities: &RuntimeCacheCapabilities,
) -> Result<(), BenchmarkError> {
    let values = [
        (
            "prefix_cache.prefix_reuse",
            &capabilities.prefix_cache.prefix_reuse,
        ),
        (
            "prefix_cache.automatic_cache_population",
            &capabilities.prefix_cache.automatic_cache_population,
        ),
        (
            "prefix_cache.explicit_cache_population",
            &capabilities.prefix_cache.explicit_cache_population,
        ),
        (
            "residency.device_kv_state",
            &capabilities.residency.device_kv_state,
        ),
        (
            "residency.host_ram_caching",
            &capabilities.residency.host_ram_caching,
        ),
        (
            "residency.disk_persistence",
            &capabilities.residency.disk_persistence,
        ),
        (
            "residency.explicit_save_restore",
            &capabilities.residency.explicit_save_restore,
        ),
        (
            "residency.checkpoint_support",
            &capabilities.residency.checkpoint_support,
        ),
        (
            "residency.idle_slot_session_caching",
            &capabilities.residency.idle_slot_session_caching,
        ),
        (
            "sessions.slots_or_sessions",
            &capabilities.sessions.slots_or_sessions,
        ),
        (
            "sessions.cache_affinity_routing_hints",
            &capabilities.sessions.cache_affinity_routing_hints,
        ),
        (
            "sessions.retention_or_ttl_controls",
            &capabilities.sessions.retention_or_ttl_controls,
        ),
        (
            "sessions.conversation_chaining",
            &capabilities.sessions.conversation_chaining,
        ),
        ("metrics.input_tokens", &capabilities.metrics.input_tokens),
        ("metrics.cached_tokens", &capabilities.metrics.cached_tokens),
        (
            "metrics.prompt_evaluation_duration",
            &capabilities.metrics.prompt_evaluation_duration,
        ),
        (
            "metrics.time_to_first_token",
            &capabilities.metrics.time_to_first_token,
        ),
        ("metrics.cache_state", &capabilities.metrics.cache_state),
        ("metrics.ram_usage", &capabilities.metrics.ram_usage),
        ("metrics.vram_usage", &capabilities.metrics.vram_usage),
        ("metrics.kv_usage", &capabilities.metrics.kv_usage),
        (
            "metrics.restore_or_rebuild",
            &capabilities.metrics.restore_or_rebuild,
        ),
        (
            "kv_cache.quantization_support",
            &capabilities.kv_cache.quantization_support,
        ),
    ];
    for (field, capability) in values {
        if let Observed::Known(value) = capability.details {
            let contradictory = match capability.support {
                CapabilitySupport::Supported => !value,
                CapabilitySupport::Unsupported => value,
                CapabilitySupport::Unknown => true,
            };
            if contradictory {
                return Err(BenchmarkError::validation(format!(
                    "{field} contains contradictory support and details"
                )));
            }
        }
    }
    Ok(())
}

fn matches_string(value: &Observed<String>, expected: &Option<String>) -> bool {
    match expected {
        None => true,
        Some(expected) => matches!(value, Observed::Known(value) if value == expected),
    }
}

fn capability_cell(
    profile_id: &str,
    key: CapabilityKey,
    capabilities: &RuntimeCacheCapabilities,
) -> CapabilityCell {
    match key {
        CapabilityKey::PrefixReuse => {
            typed_cell(profile_id, key, &capabilities.prefix_cache.prefix_reuse)
        }
        CapabilityKey::AutomaticCachePopulation => typed_cell(
            profile_id,
            key,
            &capabilities.prefix_cache.automatic_cache_population,
        ),
        CapabilityKey::ExplicitCachePopulation => typed_cell(
            profile_id,
            key,
            &capabilities.prefix_cache.explicit_cache_population,
        ),
        CapabilityKey::MinimumCacheablePrefixTokens => typed_cell(
            profile_id,
            key,
            &capabilities.prefix_cache.minimum_cacheable_prefix_tokens,
        ),
        CapabilityKey::CacheUnit => observed_cell(
            profile_id,
            key,
            &capabilities.prefix_cache.cache_unit,
            CapabilitySupport::Unknown,
            CapabilityEvidence::Unverified,
        ),
        CapabilityKey::MatchingConstraints => observed_cell(
            profile_id,
            key,
            &capabilities.prefix_cache.matching_constraints,
            CapabilitySupport::Unknown,
            CapabilityEvidence::Unverified,
        ),
        CapabilityKey::DeviceKvState => {
            typed_cell(profile_id, key, &capabilities.residency.device_kv_state)
        }
        CapabilityKey::HostRamCaching => {
            typed_cell(profile_id, key, &capabilities.residency.host_ram_caching)
        }
        CapabilityKey::DiskPersistence => {
            typed_cell(profile_id, key, &capabilities.residency.disk_persistence)
        }
        CapabilityKey::ExplicitSaveRestore => typed_cell(
            profile_id,
            key,
            &capabilities.residency.explicit_save_restore,
        ),
        CapabilityKey::CheckpointSupport => {
            typed_cell(profile_id, key, &capabilities.residency.checkpoint_support)
        }
        CapabilityKey::IdleSlotSessionCaching => typed_cell(
            profile_id,
            key,
            &capabilities.residency.idle_slot_session_caching,
        ),
        CapabilityKey::SlotsOrSessions => {
            typed_cell(profile_id, key, &capabilities.sessions.slots_or_sessions)
        }
        CapabilityKey::CacheAffinityRoutingHints => typed_cell(
            profile_id,
            key,
            &capabilities.sessions.cache_affinity_routing_hints,
        ),
        CapabilityKey::RetentionOrTtlControls => typed_cell(
            profile_id,
            key,
            &capabilities.sessions.retention_or_ttl_controls,
        ),
        CapabilityKey::ConversationChaining => typed_cell(
            profile_id,
            key,
            &capabilities.sessions.conversation_chaining,
        ),
        CapabilityKey::InputTokens => {
            typed_cell(profile_id, key, &capabilities.metrics.input_tokens)
        }
        CapabilityKey::CachedTokens => {
            typed_cell(profile_id, key, &capabilities.metrics.cached_tokens)
        }
        CapabilityKey::PromptEvaluationDuration => typed_cell(
            profile_id,
            key,
            &capabilities.metrics.prompt_evaluation_duration,
        ),
        CapabilityKey::TimeToFirstToken => {
            typed_cell(profile_id, key, &capabilities.metrics.time_to_first_token)
        }
        CapabilityKey::CacheState => typed_cell(profile_id, key, &capabilities.metrics.cache_state),
        CapabilityKey::RamUsage => typed_cell(profile_id, key, &capabilities.metrics.ram_usage),
        CapabilityKey::VramUsage => typed_cell(profile_id, key, &capabilities.metrics.vram_usage),
        CapabilityKey::KvUsage => typed_cell(profile_id, key, &capabilities.metrics.kv_usage),
        CapabilityKey::RestoreOrRebuild => {
            typed_cell(profile_id, key, &capabilities.metrics.restore_or_rebuild)
        }
        CapabilityKey::PrecisionOptions => observed_cell(
            profile_id,
            key,
            &capabilities.kv_cache.precision_options,
            CapabilitySupport::Unknown,
            CapabilityEvidence::Unverified,
        ),
        CapabilityKey::QuantizationSupport => {
            typed_cell(profile_id, key, &capabilities.kv_cache.quantization_support)
        }
        CapabilityKey::QualityRegressionEvidence => observed_cell(
            profile_id,
            key,
            &capabilities.kv_cache.quality_regression_evidence,
            CapabilitySupport::Unknown,
            CapabilityEvidence::Unverified,
        ),
    }
}

fn typed_cell<T: Serialize>(
    profile_id: &str,
    key: CapabilityKey,
    capability: &Capability<T>,
) -> CapabilityCell {
    observed_cell(
        profile_id,
        key,
        &capability.details,
        capability.support.clone(),
        capability.evidence.clone(),
    )
}

fn observed_cell<T: Serialize>(
    profile_id: &str,
    key: CapabilityKey,
    details: &Observed<T>,
    support: CapabilitySupport,
    evidence: CapabilityEvidence,
) -> CapabilityCell {
    let details = match details {
        Observed::Known(value) => {
            Observed::Known(serde_json::to_value(value).expect("capability details serialize"))
        }
        Observed::Unknown => Observed::Unknown,
        Observed::NotObserved => Observed::NotObserved,
    };
    CapabilityCell {
        profile_id: profile_id.to_string(),
        capability: key,
        state: CapabilityState::from_parts(support.clone(), evidence.clone()),
        support,
        evidence,
        details,
    }
}
