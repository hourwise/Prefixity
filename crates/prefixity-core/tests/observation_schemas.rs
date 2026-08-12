//! Validation tests and representative fixtures for the neutral observation
//! and runtime-capability contracts.

use prefixity_core::observation::{
    ArtifactLifecycle, ArtifactStability, ArtifactType, CacheObservation, CapabilityEvidence,
    CapabilitySupport, ContextArtifact, Observed, RuntimeCacheCapabilities,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture_value(relative: &str) -> Value {
    let path = workspace_root().join(relative);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|error| panic!("failed to parse {path:?}: {error}"))
}

fn fixture<T: DeserializeOwned>(relative: &str) -> T {
    serde_json::from_value(fixture_value(relative))
        .unwrap_or_else(|error| panic!("failed to deserialize {relative}: {error}"))
}

#[test]
fn valid_representative_context_artifact_fixture_loads() {
    let artifact: ContextArtifact =
        fixture("fixtures/observations/context-artifact-representative.json");
    artifact.validate().unwrap();
    assert_eq!(artifact.stability, ArtifactStability::Stable);
    assert_eq!(artifact.lifecycle, ArtifactLifecycle::PersistentVersioned);
    assert_ne!(
        artifact.stability,
        ArtifactStability::Volatile,
        "stability is not lifecycle"
    );

    let mut unknown_type = artifact;
    unknown_type.artifact_type = ArtifactType::Unknown;
    unknown_type.validate().unwrap();
}

#[test]
fn valid_representative_cache_observation_fixture_loads() {
    let observation: CacheObservation =
        fixture("fixtures/observations/cache-observation-representative.json");
    observation.validate().unwrap();

    let transmitted = match observation.accounting.transmitted_input_tokens {
        Observed::Known(value) => value.count,
        _ => panic!("fixture should include transmitted input tokens"),
    };
    let cached = match observation.accounting.provider_cached_tokens {
        Observed::Known(value) => value.count,
        _ => panic!("fixture should include provider cached tokens"),
    };
    let fresh = match observation.accounting.fresh_prefill_tokens {
        Observed::Known(value) => value.count,
        _ => panic!("fixture should include fresh prefill tokens"),
    };
    assert_eq!((transmitted, cached, fresh), (2048, 1024, 700));
    assert_ne!(cached, transmitted - fresh);
    assert!(matches!(
        observation.accounting.reconstructed_context_tokens,
        Observed::Unknown
    ));
}

#[test]
fn minimal_capability_fixture_defaults_to_unknown_unverified() {
    let capabilities: RuntimeCacheCapabilities = fixture("fixtures/capabilities/llama-cpp.json");
    capabilities.validate().unwrap();
    assert_eq!(
        capabilities.residency.device_kv_state.support,
        CapabilitySupport::Unknown
    );
    assert_eq!(
        capabilities.residency.device_kv_state.evidence,
        CapabilityEvidence::Unverified
    );
    assert!(matches!(
        capabilities.residency.device_kv_state.details,
        Observed::NotObserved
    ));
}

#[test]
fn all_local_and_cloud_capability_fixtures_are_unknown_examples() {
    let fixtures = [
        ("local", "fixtures/capabilities/llama-cpp.json"),
        ("local", "fixtures/capabilities/ollama.json"),
        ("DeepSeek", "fixtures/capabilities/deepseek.json"),
        ("Meta", "fixtures/capabilities/meta.json"),
        ("Mistral", "fixtures/capabilities/mistral.json"),
        (
            "Alibaba Model Studio",
            "fixtures/capabilities/alibaba-model-studio.json",
        ),
        ("Z.AI / GLM", "fixtures/capabilities/z-ai-glm.json"),
    ];

    for (expected_provider, path) in fixtures {
        let capabilities: RuntimeCacheCapabilities = fixture(path);
        capabilities.validate().unwrap();
        assert_eq!(
            capabilities.identity.provider,
            Observed::Known(expected_provider.to_string())
        );
        assert_eq!(
            capabilities.prefix_cache.prefix_reuse.support,
            CapabilitySupport::Unknown
        );
        assert_eq!(
            capabilities.prefix_cache.prefix_reuse.evidence,
            CapabilityEvidence::Unverified
        );
    }
}

#[test]
fn provider_model_protocol_and_runtime_dimensions_remain_distinct() {
    let local: RuntimeCacheCapabilities = fixture("fixtures/capabilities/llama-cpp.json");
    let ollama: RuntimeCacheCapabilities = fixture("fixtures/capabilities/ollama.json");
    let cloud: RuntimeCacheCapabilities = fixture("fixtures/capabilities/deepseek.json");

    assert_ne!(local.identity.backend, ollama.identity.backend);
    assert_ne!(local.identity.protocol, ollama.identity.protocol);
    assert_ne!(local.identity.provider, cloud.identity.provider);
    assert_ne!(local.identity.runtime, cloud.identity.runtime);
    assert_ne!(local.identity.model, cloud.identity.model);
}

#[test]
fn unknown_fields_are_ignored_and_raw_telemetry_is_preserved() {
    let mut value = fixture_value("fixtures/observations/cache-observation-representative.json");
    let object = value
        .as_object_mut()
        .expect("observation fixture must be an object");
    object.insert("future_backend_field".to_string(), json!({"v": 1}));
    object
        .get_mut("raw_telemetry")
        .and_then(Value::as_object_mut)
        .expect("fixture must carry raw telemetry")
        .insert("provider_native_cache_counter".to_string(), json!(17));

    let observation: CacheObservation = serde_json::from_value(value).unwrap();
    observation.validate().unwrap();
    assert_eq!(
        observation.raw_telemetry["provider_native_cache_counter"],
        json!(17)
    );
}

#[test]
fn optional_telemetry_is_truly_optional() {
    let mut value = fixture_value("fixtures/observations/cache-observation-representative.json");
    value
        .as_object_mut()
        .expect("observation fixture must be an object")
        .remove("raw_telemetry");
    let observation: CacheObservation = serde_json::from_value(value).unwrap();
    observation.validate().unwrap();
    assert!(observation.raw_telemetry.is_empty());
}

#[test]
fn malformed_artifact_data_is_rejected() {
    let mut value = fixture_value("fixtures/observations/context-artifact-representative.json");
    value["schema_version"] = json!(99);
    let artifact: ContextArtifact = serde_json::from_value(value).unwrap();
    assert!(artifact.validate().is_err());

    let mut value = fixture_value("fixtures/observations/context-artifact-representative.json");
    value["content_hash"]["value"] = json!("not-a-sha256");
    let artifact: ContextArtifact = serde_json::from_value(value).unwrap();
    assert!(artifact.validate().is_err());

    let mut value = fixture_value("fixtures/observations/context-artifact-representative.json");
    value
        .as_object_mut()
        .expect("artifact fixture must be an object")
        .remove("artifact_id");
    assert!(serde_json::from_value::<ContextArtifact>(value).is_err());
}

#[test]
fn unsupported_claim_without_evidence_is_rejected() {
    let mut capabilities: RuntimeCacheCapabilities = fixture("fixtures/capabilities/deepseek.json");
    capabilities.prefix_cache.prefix_reuse.support = CapabilitySupport::Supported;
    capabilities.prefix_cache.prefix_reuse.evidence = CapabilityEvidence::Unverified;
    assert!(capabilities.validate().is_err());
}

#[test]
fn store_conversation_persistence_disk_cache_and_kv_are_not_one_mechanism() {
    let mut capabilities: RuntimeCacheCapabilities = fixture("fixtures/capabilities/ollama.json");
    capabilities
        .raw_capabilities
        .insert("store".to_string(), json!(false));
    capabilities.sessions.conversation_chaining.support = CapabilitySupport::Supported;
    capabilities.sessions.conversation_chaining.evidence = CapabilityEvidence::Documented;
    capabilities.residency.disk_persistence.support = CapabilitySupport::Unsupported;
    capabilities.residency.disk_persistence.evidence = CapabilityEvidence::Documented;
    capabilities.residency.device_kv_state.support = CapabilitySupport::Unknown;
    capabilities.residency.device_kv_state.evidence = CapabilityEvidence::Unverified;
    capabilities.validate().unwrap();

    assert_eq!(capabilities.raw_capabilities["store"], json!(false));
    assert_eq!(
        capabilities.sessions.conversation_chaining.support,
        CapabilitySupport::Supported
    );
    assert_eq!(
        capabilities.residency.disk_persistence.support,
        CapabilitySupport::Unsupported
    );
    assert_eq!(
        capabilities.residency.device_kv_state.support,
        CapabilitySupport::Unknown
    );
}

#[test]
fn raw_telemetry_is_bounded() {
    let mut observation: CacheObservation =
        fixture("fixtures/observations/cache-observation-representative.json");
    for index in 0..65 {
        observation
            .raw_telemetry
            .insert(format!("field-{index}"), json!(index));
    }
    assert!(observation.validate().is_err());
}
