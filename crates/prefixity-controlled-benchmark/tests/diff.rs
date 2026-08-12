use prefixity_controlled_benchmark::{
    envelope_diff, prefix_diff, request_diff, CacheImpactAssessment, ChangeCategory,
    ConformanceRequest, ContextArtifactInput, DiffState, EnvelopeField,
};
use prefixity_core::observation::Observed;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture_requests() -> Vec<ConformanceRequest> {
    let path = workspace_root().join("fixtures/conformance/coding-agent-cache-conformance-v1.json");
    let bytes = std::fs::read(path).expect("conformance fixture should exist");
    let value: Value = serde_json::from_slice(&bytes).expect("conformance fixture should parse");
    let experiment: prefixity_controlled_benchmark::ConformanceExperiment =
        serde_json::from_value(value).expect("conformance fixture should deserialize");
    experiment
        .cases
        .into_iter()
        .map(|case| case.request)
        .collect()
}

#[test]
fn identical_requests_have_empty_deterministic_diff_and_preserve_fingerprints() {
    let requests = fixture_requests();
    let left = &requests[0];
    let before = left.request_fingerprint().unwrap();
    let first = request_diff(left, &requests[1]).unwrap();
    let second = request_diff(left, &requests[1]).unwrap();
    assert_eq!(first, second);
    assert!(first.prefix_diff.identical);
    assert!(first.envelope_diff.identical);
    assert!(first.prefix_diff.changes.is_empty());
    assert!(first.envelope_diff.changes.is_empty());
    assert_eq!(first.interpretation.context, DiffState::Identical);
    assert_eq!(first.interpretation.envelope, DiffState::Identical);
    assert_eq!(first.cache_impact, CacheImpactAssessment::Unknown);
    assert_eq!(left.request_fingerprint().unwrap(), before);
    assert!(matches!(
        first.prefix_diff.common_prefix.token_units,
        Observed::NotObserved
    ));
    assert_eq!(
        serde_json::to_vec(&first).unwrap(),
        serde_json::to_vec(&second).unwrap()
    );
}

#[test]
fn beginning_and_ending_mutations_report_their_first_context_path() {
    let requests = fixture_requests();
    let beginning = prefix_diff(&requests[0], &requests[2]).unwrap();
    assert_eq!(
        beginning.first_divergence.as_ref().unwrap().path,
        "context.artifacts[0].content"
    );
    assert_eq!(
        beginning.first_divergence.as_ref().unwrap().category,
        ChangeCategory::ArtifactContentChanged
    );
    assert_eq!(beginning.common_prefix.structural_units, 1);

    let ending = prefix_diff(&requests[0], &requests[3]).unwrap();
    assert_eq!(
        ending.first_divergence.as_ref().unwrap().path,
        "context.current_user"
    );
    assert_eq!(ending.common_prefix.artifact_units, 1);
    assert!(ending
        .common_prefix
        .text
        .iter()
        .any(|prefix| { prefix.path == "context.current_user" && prefix.common_bytes > 0 }));
}

#[test]
fn whitespace_only_change_is_classified_without_normalizing_input() {
    let requests = fixture_requests();
    let diff = prefix_diff(&requests[0], &requests[4]).unwrap();
    let change = diff.first_divergence.as_ref().unwrap();
    assert_eq!(change.path, "context.current_user");
    assert_eq!(change.category, ChangeCategory::TextContentChanged);
    assert!(change.whitespace_only);
    assert_ne!(
        requests[0].context.user_content,
        requests[4].context.user_content
    );
}

#[test]
fn artifact_add_remove_and_reorder_remain_distinct() {
    let requests = fixture_requests();
    let mut added = requests[0].clone();
    added.context.artifacts.push(ContextArtifactInput {
        artifact_id: "new-artifact:v1".to_string(),
        content: "bounded extra context".to_string(),
    });
    let added_diff = prefix_diff(&requests[0], &added).unwrap();
    assert!(added_diff
        .changes
        .iter()
        .any(|change| change.category == ChangeCategory::ArtifactAdded));

    let mut removed = requests[0].clone();
    removed.context.artifacts.clear();
    let removed_diff = prefix_diff(&requests[0], &removed).unwrap();
    assert!(removed_diff
        .changes
        .iter()
        .any(|change| change.category == ChangeCategory::ArtifactRemoved));

    let mut reordered = requests[0].clone();
    reordered.context.artifacts.push(ContextArtifactInput {
        artifact_id: "second-artifact:v1".to_string(),
        content: "second".to_string(),
    });
    reordered.context.artifacts.reverse();
    let mut original = requests[0].clone();
    original.context.artifacts.push(ContextArtifactInput {
        artifact_id: "second-artifact:v1".to_string(),
        content: "second".to_string(),
    });
    let reordered_diff = prefix_diff(&original, &reordered).unwrap();
    assert_eq!(
        reordered_diff.first_divergence.as_ref().unwrap().category,
        ChangeCategory::ArtifactOrderChanged
    );
    assert!(!reordered_diff.changes.iter().any(|change| matches!(
        change.category,
        ChangeCategory::ArtifactAdded | ChangeCategory::ArtifactRemoved
    )));
}

#[test]
fn tool_order_schema_field_and_definition_changes_are_identified() {
    let requests = fixture_requests();
    let tool_order = prefix_diff(&requests[0], &requests[6]).unwrap();
    assert_eq!(
        tool_order.first_divergence.as_ref().unwrap().category,
        ChangeCategory::ToolOrderChanged
    );

    let field_order = prefix_diff(&requests[0], &requests[5]).unwrap();
    assert!(field_order
        .changes
        .iter()
        .any(|change| change.category == ChangeCategory::OrderedSchemaFieldChanged));

    let optional = prefix_diff(&requests[0], &requests[7]).unwrap();
    assert!(optional
        .changes
        .iter()
        .any(|change| change.category == ChangeCategory::OptionalSchemaFieldAdded));

    let mut optional_removed_request = requests[0].clone();
    optional_removed_request.context.tools[0]
        .parameters
        .fields
        .pop();
    let optional_removed = prefix_diff(&requests[0], &optional_removed_request).unwrap();
    assert!(optional_removed
        .changes
        .iter()
        .any(|change| change.category == ChangeCategory::OptionalSchemaFieldRemoved));

    let changed = prefix_diff(&requests[0], &requests[8]).unwrap();
    assert_eq!(
        changed.first_divergence.as_ref().unwrap().path,
        "context.tools[0].description"
    );
    assert_eq!(
        changed.first_divergence.as_ref().unwrap().category,
        ChangeCategory::ToolDefinitionChanged
    );
}

#[test]
fn envelope_only_mutations_do_not_enter_prefix_diff() {
    let requests = fixture_requests();
    for (index, field) in [
        (9, EnvelopeField::Model),
        (10, EnvelopeField::Reasoning),
        (11, EnvelopeField::ResponseFormat),
    ] {
        let prefix = prefix_diff(&requests[0], &requests[index]).unwrap();
        let envelope = envelope_diff(&requests[0], &requests[index]).unwrap();
        assert!(prefix.identical);
        assert!(!envelope.identical);
        assert_eq!(envelope.changes.len(), 1);
        assert_eq!(envelope.changes[0].field, field);
        assert_eq!(envelope.cache_impact, CacheImpactAssessment::Unknown);
    }
}

#[test]
fn combined_diff_reports_context_and_envelope_independently() {
    let requests = fixture_requests();
    let mut both = requests[3].clone();
    both.envelope.model = "another-model".to_string();
    let diff = request_diff(&requests[0], &both).unwrap();
    assert_eq!(diff.interpretation.context, DiffState::Changed);
    assert_eq!(diff.interpretation.envelope, DiffState::Changed);
    assert!(!diff.prefix_diff.identical);
    assert!(!diff.envelope_diff.identical);
    assert_eq!(diff.cache_impact, CacheImpactAssessment::Unknown);
}

#[test]
fn large_changed_content_is_bounded_and_invalid_pairs_fail_cleanly() {
    let requests = fixture_requests();
    let mut large = requests[0].clone();
    large.context.user_content = "x".repeat(10_000);
    let diff = prefix_diff(&requests[0], &large).unwrap();
    let change = diff.first_divergence.as_ref().unwrap();
    assert!(change.right.as_ref().unwrap().size_bytes >= 10_000);
    assert!(
        change
            .right
            .as_ref()
            .unwrap()
            .preview
            .as_ref()
            .unwrap()
            .len()
            < 100
    );
    assert!(!serde_json::to_vec(&diff)
        .unwrap()
        .windows(1_000)
        .any(|window| { window.iter().all(|byte| *byte == b'x') }));

    let mut invalid = requests[0].clone();
    invalid.context.artifacts[0].artifact_id = invalid.context.artifacts[0].artifact_id.clone();
    invalid
        .context
        .artifacts
        .push(invalid.context.artifacts[0].clone());
    assert!(prefix_diff(&requests[0], &invalid).is_err());
}
