use prefixity_controlled_benchmark::{
    load_approved_capability_registry, CapabilityKey, CapabilityProfile, CapabilityQuery,
    CapabilityRegistry, CapabilityState, RegistryEvidenceOrigin,
};
use prefixity_core::observation::{CapabilityEvidence, CapabilitySupport, Observed};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn registry() -> CapabilityRegistry {
    load_approved_capability_registry(&workspace_root()).expect("approved fixtures should load")
}

fn documented_llama_profile() -> CapabilityProfile {
    registry()
        .query(&CapabilityQuery {
            protocol: Some("llama.cpp-openai-chat-v1".to_string()),
            ..CapabilityQuery::default()
        })
        .into_iter()
        .next()
        .expect("documented llama profile")
        .clone()
}

#[test]
fn approved_fixture_profiles_load_and_validate() {
    let registry = registry();
    assert_eq!(registry.profiles.len(), 8);
    registry.validate().unwrap();
    assert!(
        registry
            .query(&CapabilityQuery {
                runtime: Some("llama.cpp".to_string()),
                ..CapabilityQuery::default()
            })
            .len()
            >= 2
    );
}

#[test]
fn profile_identity_is_deterministic_and_semantic() {
    let first = registry();
    let second = registry();
    assert_eq!(
        first
            .profiles
            .iter()
            .map(|profile| &profile.profile_id)
            .collect::<Vec<_>>(),
        second
            .profiles
            .iter()
            .map(|profile| &profile.profile_id)
            .collect::<Vec<_>>()
    );
    let mut changed = documented_llama_profile();
    changed.capabilities.prefix_cache.prefix_reuse.details = Observed::Known(false);
    changed.capabilities.prefix_cache.prefix_reuse.support = CapabilitySupport::Unsupported;
    changed.capabilities.prefix_cache.prefix_reuse.evidence = CapabilityEvidence::Documented;
    let changed = CapabilityProfile::from_capabilities(
        changed.capabilities,
        RegistryEvidenceOrigin::ProjectDocumentation,
        Default::default(),
    )
    .unwrap();
    assert_ne!(documented_llama_profile().profile_id, changed.profile_id);
}

#[test]
fn duplicate_profile_identity_and_semantics_fail_cleanly() {
    let profile = documented_llama_profile();
    let error =
        CapabilityRegistry::from_profiles(vec![profile.clone(), profile], Default::default())
            .unwrap_err()
            .to_string();
    assert!(error.contains("duplicate profile identities"));
}

#[test]
fn malformed_capability_profile_fails_cleanly() {
    let mut capabilities = documented_llama_profile().capabilities;
    capabilities.prefix_cache.prefix_reuse.support = CapabilitySupport::Supported;
    capabilities.prefix_cache.prefix_reuse.evidence = CapabilityEvidence::Unverified;
    assert!(CapabilityProfile::from_capabilities(
        capabilities,
        RegistryEvidenceOrigin::SyntheticFixture,
        Default::default(),
    )
    .is_err());
}

#[test]
fn documented_observed_unknown_and_unsupported_states_remain_distinct() {
    let documented = documented_llama_profile();
    let documented_cell = documented.capability(CapabilityKey::PrefixReuse);
    assert_eq!(documented_cell.state, CapabilityState::SupportedDocumented);

    let mut observed_capabilities = documented.capabilities.clone();
    observed_capabilities.prefix_cache.prefix_reuse.evidence =
        CapabilityEvidence::ExperimentallyObserved;
    let observed = CapabilityProfile::from_capabilities(
        observed_capabilities,
        RegistryEvidenceOrigin::ExperimentalObservation,
        Default::default(),
    )
    .unwrap();
    assert_eq!(
        observed.capability(CapabilityKey::PrefixReuse).state,
        CapabilityState::SupportedObserved
    );

    let unknown = registry()
        .query(&CapabilityQuery {
            protocol: Some("llama.cpp-http".to_string()),
            ..CapabilityQuery::default()
        })
        .into_iter()
        .next()
        .unwrap()
        .capability(CapabilityKey::PrefixReuse);
    assert_eq!(unknown.state, CapabilityState::UnknownUnverified);

    let mut unsupported_capabilities = documented.capabilities;
    unsupported_capabilities.prefix_cache.prefix_reuse.support = CapabilitySupport::Unsupported;
    unsupported_capabilities.prefix_cache.prefix_reuse.details = Observed::Known(false);
    let unsupported = CapabilityProfile::from_capabilities(
        unsupported_capabilities,
        RegistryEvidenceOrigin::ProjectDocumentation,
        Default::default(),
    )
    .unwrap();
    assert_eq!(
        unsupported.capability(CapabilityKey::PrefixReuse).state,
        CapabilityState::UnsupportedDocumented
    );
}

#[test]
fn synthetic_fixture_origin_does_not_promote_documented_capabilities_to_observed() {
    let registry = registry();
    assert!(registry
        .profiles
        .iter()
        .all(|profile| profile.origin == RegistryEvidenceOrigin::SyntheticFixture));
    let documented = documented_llama_profile();
    let cell = documented.capability(CapabilityKey::CachedTokens);
    assert_eq!(cell.evidence, CapabilityEvidence::Documented);
    assert_ne!(cell.evidence, CapabilityEvidence::ExperimentallyObserved);
}

#[test]
fn identity_scope_preserves_provider_protocol_model_and_runtime_version() {
    let registry = registry();
    let local = registry.query(&CapabilityQuery {
        provider: Some("local".to_string()),
        ..CapabilityQuery::default()
    });
    assert_eq!(local.len(), 3);
    assert_eq!(
        registry
            .query(&CapabilityQuery {
                protocol: Some("llama.cpp-openai-chat-v1".to_string()),
                ..CapabilityQuery::default()
            })
            .len(),
        1
    );
    assert_eq!(
        registry
            .query(&CapabilityQuery {
                model: Some("example-local-model".to_string()),
                ..CapabilityQuery::default()
            })
            .len(),
        2
    );
    assert_eq!(
        registry
            .query(&CapabilityQuery {
                runtime_version: Some("1".to_string()),
                ..CapabilityQuery::default()
            })
            .len(),
        0
    );
}

#[test]
fn query_filters_are_typed_and_stably_ordered() {
    let registry = registry();
    let first = registry
        .query(&CapabilityQuery {
            provider: Some("Mistral".to_string()),
            ..CapabilityQuery::default()
        })
        .iter()
        .map(|profile| profile.profile_id.clone())
        .collect::<Vec<_>>();
    let second = registry
        .query(&CapabilityQuery {
            provider: Some("Mistral".to_string()),
            ..CapabilityQuery::default()
        })
        .iter()
        .map(|profile| profile.profile_id.clone())
        .collect::<Vec<_>>();
    assert_eq!(first, second);
    assert_eq!(first.len(), 1);
}

#[test]
fn capability_and_evidence_queries_keep_unknown_profiles() {
    let registry = registry();
    let documented_prefix = registry.query(&CapabilityQuery {
        capability: Some(CapabilityKey::PrefixReuse),
        support: Some(CapabilitySupport::Supported),
        evidence: Some(CapabilityEvidence::Documented),
        ..CapabilityQuery::default()
    });
    assert_eq!(documented_prefix.len(), 1);

    let unknown_cached = registry.query(&CapabilityQuery {
        capability: Some(CapabilityKey::CachedTokens),
        support: Some(CapabilitySupport::Unknown),
        ..CapabilityQuery::default()
    });
    assert_eq!(unknown_cached.len(), 7);
    assert!(registry
        .query(&CapabilityQuery {
            evidence: Some(CapabilityEvidence::ExperimentallyObserved),
            ..CapabilityQuery::default()
        })
        .is_empty());
}

#[test]
fn matrix_is_generated_with_deterministic_profile_and_capability_order() {
    let registry = registry();
    let query = CapabilityQuery {
        provider: Some("local".to_string()),
        ..CapabilityQuery::default()
    };
    let selected = [CapabilityKey::CachedTokens, CapabilityKey::PrefixReuse];
    let first = registry.matrix(&query, &selected);
    let second = registry.matrix(&query, &selected);
    assert_eq!(first, second);
    assert_eq!(
        first.capabilities,
        vec![CapabilityKey::PrefixReuse, CapabilityKey::CachedTokens]
    );
    assert_eq!(first.profile_ids.len(), 3);
    assert_eq!(first.render_markdown(), second.render_markdown());
    assert!(first.render_markdown().contains("supported_documented"));
    assert!(first.render_markdown().contains("unknown_unverified"));
}

#[test]
fn matrix_cells_preserve_unknown_details_and_evidence() {
    let matrix = registry().matrix(
        &CapabilityQuery {
            protocol: Some("llama.cpp-http".to_string()),
            ..CapabilityQuery::default()
        },
        &[CapabilityKey::CachedTokens],
    );
    let cell = &matrix.rows[0].cells[0];
    assert_eq!(cell.support, CapabilitySupport::Unknown);
    assert_eq!(cell.evidence, CapabilityEvidence::Unverified);
    assert!(matches!(
        cell.details,
        Observed::Unknown | Observed::NotObserved
    ));
}

#[test]
fn gap_report_counts_unknown_known_and_observed_without_validation_claims() {
    let registry = registry();
    let report = registry.gap_report();
    let prefix = report
        .capability_gaps
        .iter()
        .find(|gap| gap.capability == CapabilityKey::PrefixReuse)
        .unwrap();
    assert_eq!(prefix.known_profiles, 1);
    assert_eq!(prefix.unknown_profiles, 7);
    assert_eq!(prefix.experimentally_observed_profiles, 0);
    assert!(report
        .profile_gaps
        .iter()
        .all(|gap| gap.experimentally_observed_fields == 0));
}

#[test]
fn repeated_registry_serialization_is_identical() {
    let registry = registry();
    assert_eq!(
        serde_json::to_vec(&registry).unwrap(),
        serde_json::to_vec(&registry).unwrap()
    );
    let value: Value = serde_json::to_value(&registry).unwrap();
    assert_eq!(value["registry_version"], 1);
}
