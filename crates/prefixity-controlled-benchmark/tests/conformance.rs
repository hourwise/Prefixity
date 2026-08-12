use prefixity_controlled_benchmark::{
    CaseRelationship, ConformanceExperiment, ConformanceRequest, ExpectedObservationMetadata,
    ExpectedObservationState, JsonField, MockConformanceRunner, MutationClass, OrderedJsonObject,
    ReasoningSetting, RequestContext, RequestEnvelope, ResponseFormat, RuntimeProfileReference,
    ToolDefinition, CONFORMANCE_SCHEMA_ID, CONFORMANCE_SCHEMA_VERSION, MOCK_TRANSPORT_ID,
};
use prefixity_core::observation::{Observed, RuntimeIdentity, CACHE_OBSERVATION_SCHEMA_VERSION};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture_experiment() -> ConformanceExperiment {
    let value = std::fs::read(
        workspace_root().join("fixtures/conformance/coding-agent-cache-conformance-v1.json"),
    )
    .expect("conformance fixture should exist");
    serde_json::from_slice(&value).expect("conformance fixture should parse")
}

fn run_fixture() -> prefixity_controlled_benchmark::ConformanceResult {
    let experiment = fixture_experiment();
    let runtime = experiment.runtime_profile.identity.clone();
    let mut runner = MockConformanceRunner::new("2026-08-12T12:00:00Z", runtime);
    experiment.run(&mut runner).expect("fixture should run")
}

fn sample_request() -> ConformanceRequest {
    let parameters = OrderedJsonObject::new(vec![
        JsonField {
            name: "type".to_string(),
            value: json!("object"),
        },
        JsonField {
            name: "properties".to_string(),
            value: json!({"path": {"type": "string"}}),
        },
    ]);
    let tools = vec![
        ToolDefinition {
            name: "read_file".to_string(),
            description: "Read a source file for the current task.".to_string(),
            parameters: parameters.clone(),
        },
        ToolDefinition {
            name: "list_files".to_string(),
            description: "List files under a bounded workspace path.".to_string(),
            parameters: OrderedJsonObject::new(vec![JsonField {
                name: "type".to_string(),
                value: json!("object"),
            }]),
        },
        ToolDefinition {
            name: "run_tests".to_string(),
            description: "Run the focused offline test command.".to_string(),
            parameters: OrderedJsonObject::new(vec![JsonField {
                name: "type".to_string(),
                value: json!("object"),
            }]),
        },
    ];
    ConformanceRequest {
        context: RequestContext {
            system_instruction:
                "You are a careful coding agent. Preserve repository safety boundaries.".to_string(),
            artifacts: vec![prefixity_controlled_benchmark::ContextArtifactInput {
                artifact_id: "workspace-map:v1".to_string(),
                content: "src/lib.rs\nsrc/main.rs\ntests/\nCargo.toml".to_string(),
            }],
            user_content: "Inspect the parser and add a focused regression test.".to_string(),
            tools,
        },
        envelope: RequestEnvelope {
            model: "example-local-model".to_string(),
            reasoning: Some(ReasoningSetting::Disabled),
            response_format: Some(ResponseFormat::Text),
        },
    }
}

fn expected() -> ExpectedObservationMetadata {
    ExpectedObservationMetadata {
        cache_reuse: ExpectedObservationState::ToBeObserved,
        cache_write: ExpectedObservationState::ToBeObserved,
        notes: "No provider expectation is asserted; observe if a future transport exposes it."
            .to_string(),
    }
}

fn programmatic_experiment() -> ConformanceExperiment {
    let baseline = sample_request();
    let exact = baseline.clone();
    let mut beginning = baseline.clone();
    beginning.context.artifacts[0].content =
        "src/changed.rs\nsrc/main.rs\ntests/\nCargo.toml".to_string();
    let mut ending = baseline.clone();
    ending
        .context
        .user_content
        .push_str(" Keep the change minimal.");
    let mut whitespace = baseline.clone();
    whitespace.context.user_content = whitespace.context.user_content.replace("parser", "parser ");
    let mut field_order = baseline.clone();
    field_order.context.tools[0].parameters = field_order.context.tools[0].parameters.reordered();
    let mut tool_order = baseline.clone();
    tool_order.context.tools.reverse();
    let mut optional_field = baseline.clone();
    optional_field.context.tools[0].parameters = optional_field.context.tools[0]
        .parameters
        .with_field("additionalProperties", json!(false))
        .unwrap();
    let mut tool_change = baseline.clone();
    tool_change.context.tools[0]
        .description
        .push_str(" Return UTF-8 text.");
    let mut model = baseline.clone();
    model.envelope.model = "example-local-model-v2".to_string();
    let mut reasoning = baseline.clone();
    reasoning.envelope.reasoning = Some(ReasoningSetting::Enabled);
    let mut response = baseline.clone();
    response.envelope.response_format = Some(ResponseFormat::JsonObject);
    let case = |case_id: &str,
                mutation: MutationClass,
                request: ConformanceRequest,
                relationship: CaseRelationship| {
        prefixity_controlled_benchmark::ConformanceCase {
            case_id: case_id.to_string(),
            mutation,
            request,
            relationship,
            expected_observation: expected(),
        }
    };
    ConformanceExperiment {
        schema_id: CONFORMANCE_SCHEMA_ID.to_string(),
        schema_version: CONFORMANCE_SCHEMA_VERSION,
        experiment_id: "coding-agent-cache-conformance-v1".to_string(),
        baseline_request: baseline.clone(),
        cases: vec![
            case(
                "baseline",
                MutationClass::Baseline,
                baseline,
                CaseRelationship::Baseline,
            ),
            case(
                "exact-repeat",
                MutationClass::ExactRepeat,
                exact,
                CaseRelationship::ExactRepeatOf("baseline".to_string()),
            ),
            case(
                "content-beginning",
                MutationClass::StableContentBeginning,
                beginning,
                CaseRelationship::MutationOf("baseline".to_string()),
            ),
            case(
                "content-end",
                MutationClass::CurrentContentEnd,
                ending,
                CaseRelationship::MutationOf("baseline".to_string()),
            ),
            case(
                "whitespace-only",
                MutationClass::WhitespaceOnly,
                whitespace,
                CaseRelationship::MutationOf("baseline".to_string()),
            ),
            case(
                "json-field-order",
                MutationClass::JsonFieldOrder,
                field_order,
                CaseRelationship::MutationOf("baseline".to_string()),
            ),
            case(
                "tool-order",
                MutationClass::ToolDefinitionOrder,
                tool_order,
                CaseRelationship::MutationOf("baseline".to_string()),
            ),
            case(
                "optional-tool-field",
                MutationClass::OptionalToolField,
                optional_field,
                CaseRelationship::MutationOf("baseline".to_string()),
            ),
            case(
                "tool-change",
                MutationClass::ToolDefinitionChange,
                tool_change,
                CaseRelationship::MutationOf("baseline".to_string()),
            ),
            case(
                "model-change",
                MutationClass::ModelIdentifier,
                model,
                CaseRelationship::MutationOf("baseline".to_string()),
            ),
            case(
                "reasoning-change",
                MutationClass::ReasoningSetting,
                reasoning,
                CaseRelationship::MutationOf("baseline".to_string()),
            ),
            case(
                "response-format-change",
                MutationClass::ResponseFormat,
                response,
                CaseRelationship::MutationOf("baseline".to_string()),
            ),
        ],
        runtime_profile: RuntimeProfileReference {
            profile_id: "synthetic-mock-profile-v1".to_string(),
            identity: RuntimeIdentity {
                backend: MOCK_TRANSPORT_ID.to_string(),
                provider: Observed::Known("synthetic-test".to_string()),
                model: Observed::Known("example-local-model".to_string()),
                protocol: Observed::Known("conformance-mock-v1".to_string()),
                runtime: Observed::Known("in-process".to_string()),
                runtime_version: Observed::Known("1".to_string()),
                ..RuntimeIdentity::default()
            },
        },
        metadata: BTreeMap::from([
            ("workload".to_string(), "coding-agent-style".to_string()),
            ("evidence".to_string(), "synthetic-fixture-only".to_string()),
        ]),
    }
}

#[test]
fn complete_fixture_has_expected_shape_and_unknown_expectations() {
    let experiment = fixture_experiment();
    experiment.validate().unwrap();
    assert_eq!(experiment.cases.len(), 12);
    assert!(experiment.cases.iter().all(
        |case| case.expected_observation.cache_reuse == ExpectedObservationState::ToBeObserved
    ));
    assert!(experiment.baseline_request.context.tools.len() >= 3);
    assert_eq!(experiment.baseline_request.context.artifacts.len(), 1);
}

#[test]
fn baseline_and_exact_repeat_have_distinct_case_ids_but_same_request_identity() {
    let experiment = fixture_experiment();
    let baseline = &experiment.cases[0];
    let repeat = &experiment.cases[1];
    assert_ne!(baseline.case_id, repeat.case_id);
    assert_eq!(
        baseline.request.request_fingerprint().unwrap(),
        repeat.request.request_fingerprint().unwrap()
    );
}

#[test]
fn content_mutations_change_fingerprints_deterministically() {
    let experiment = fixture_experiment();
    let baseline = &experiment.cases[0].request;
    let beginning = &experiment.cases[2].request;
    let ending = &experiment.cases[3].request;
    assert_ne!(
        baseline.request_fingerprint().unwrap(),
        beginning.request_fingerprint().unwrap()
    );
    assert_ne!(
        baseline.request_fingerprint().unwrap(),
        ending.request_fingerprint().unwrap()
    );
    assert_eq!(
        beginning.request_fingerprint().unwrap(),
        beginning.request_fingerprint().unwrap()
    );
}

#[test]
fn whitespace_is_an_intentional_request_mutation() {
    let experiment = fixture_experiment();
    let baseline = &experiment.cases[0].request;
    let whitespace = &experiment.cases[4].request;
    assert_ne!(
        baseline.context.user_content,
        whitespace.context.user_content
    );
    assert_ne!(
        baseline.request_fingerprint().unwrap(),
        whitespace.request_fingerprint().unwrap()
    );
}

#[test]
fn structured_and_envelope_mutations_remain_distinguishable() {
    let experiment = fixture_experiment();
    let baseline = &experiment.cases[0].request;
    assert_ne!(
        baseline.request_fingerprint().unwrap(),
        experiment.cases[5].request.request_fingerprint().unwrap()
    );
    assert_ne!(
        baseline.request_fingerprint().unwrap(),
        experiment.cases[6].request.request_fingerprint().unwrap()
    );
    assert_ne!(
        baseline.request_fingerprint().unwrap(),
        experiment.cases[7].request.request_fingerprint().unwrap()
    );
    assert_ne!(
        baseline.request_fingerprint().unwrap(),
        experiment.cases[8].request.request_fingerprint().unwrap()
    );
    assert_eq!(
        baseline.context_fingerprint().unwrap(),
        experiment.cases[9].request.context_fingerprint().unwrap()
    );
    assert_eq!(
        baseline.context_fingerprint().unwrap(),
        experiment.cases[10].request.context_fingerprint().unwrap()
    );
    assert_eq!(
        baseline.context_fingerprint().unwrap(),
        experiment.cases[11].request.context_fingerprint().unwrap()
    );
}

#[test]
fn execution_is_ordered_traceable_and_does_not_fabricate_cache_values() {
    let result = run_fixture();
    result.validate().unwrap();
    assert_eq!(result.cases[0].case_id, "baseline");
    assert_eq!(result.cases[1].case_id, "exact-repeat");
    for case in &result.cases {
        assert_eq!(
            case.observation.schema_version,
            CACHE_OBSERVATION_SCHEMA_VERSION
        );
        assert!(matches!(
            case.observation.accounting.provider_cached_tokens,
            Observed::NotObserved
        ));
        assert!(matches!(
            case.observation.cache.cache_hit,
            Observed::NotObserved
        ));
        assert_eq!(
            case.observation.raw_telemetry["transport"],
            json!(MOCK_TRANSPORT_ID)
        );
        assert_eq!(
            case.observation.context.serialized_request_identity,
            Observed::Known(case.request_fingerprint.clone())
        );
    }
}

#[test]
fn result_serialization_is_deterministic_across_repeated_runs() {
    let first = run_fixture().canonical_json().unwrap();
    let second = run_fixture().canonical_json().unwrap();
    assert_eq!(first, second);
}

#[test]
fn malformed_experiments_fail_cleanly() {
    let mut duplicate = programmatic_experiment();
    duplicate.cases[1].case_id = duplicate.cases[0].case_id.clone();
    assert!(duplicate.validate().is_err());

    let mut missing_target = programmatic_experiment();
    missing_target.cases[1].relationship = CaseRelationship::ExactRepeatOf("missing".to_string());
    assert!(missing_target.validate().is_err());

    let mut malformed = programmatic_experiment();
    malformed.schema_version = 99;
    assert!(malformed.validate().is_err());
}

#[test]
fn ordered_json_rejects_duplicate_fields_without_map_order_dependence() {
    let mut request = sample_request();
    request.context.tools[0].parameters.fields.push(JsonField {
        name: "type".to_string(),
        value: json!("object"),
    });
    assert!(request.validate().is_err());
}

#[test]
fn capability_evidence_is_not_created_by_the_mock_runner() {
    let result = run_fixture();
    assert_eq!(result.provenance["cache_metrics"], "not_observed");
    assert!(result.cases.iter().all(|case| {
        matches!(
            case.observation.accounting.transmitted_input_tokens,
            Observed::NotObserved
        )
    }));
}
