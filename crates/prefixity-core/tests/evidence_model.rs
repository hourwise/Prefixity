use prefixity_core::model::{
    ContextBlock, EvidenceOrigin, EvidenceProvenance, ProviderFieldState, ProviderResponseMetadata,
    RequestTrace, SourceLocator, TRACE_FORMAT_VERSION,
};
use std::collections::BTreeMap;

#[test]
fn historical_trace_v2_remains_compatible_with_evidence_extension() {
    let trace: RequestTrace = serde_json::from_value(serde_json::json!({
        "format_version": TRACE_FORMAT_VERSION,
        "request_id": "historical",
        "provider": "synthetic",
        "model": "synthetic-model",
        "blocks": [{
            "id": "block",
            "source": "user_request",
            "position": 0,
            "content_hash": "0000000000000000000000000000000000000000000000000000000000000000",
            "byte_count": 0
        }]
    }))
    .unwrap();

    assert_eq!(trace.evidence_schema_version, None);
    assert!(trace.provider_response.is_none());
    assert!(trace.provenance.is_empty());
    assert!(trace.blocks[0].timestamp.is_none());
}

#[test]
fn evidence_round_trip_keeps_origin_and_response_identity_separate_from_dependencies() {
    let locator = SourceLocator {
        trajectory_id: Some("trajectory".to_string()),
        source_file_sha256: Some("a".repeat(64)),
        source_event_index: Some(4),
        source_event_id: Some("message-0004".to_string()),
        upstream_field_path: Some("messages[4].timestamp".to_string()),
    };
    let provenance = EvidenceProvenance {
        origin: EvidenceOrigin::SourceExplicit,
        source_locator: Some(locator.clone()),
        derivation_rule: None,
        derivation_inputs: Vec::new(),
        evaluation_only: false,
    };
    let block = ContextBlock {
        id: "message-0004".to_string(),
        source: "conversation".to_string(),
        position: 0,
        content_hash: "0".repeat(64),
        token_count: Some(1),
        byte_count: 0,
        timestamp: Some(123.5),
        content: None,
        semantic_zone: Some("messages".to_string()),
        structural_path: Some("messages[4]".to_string()),
        role: Some("assistant".to_string()),
        sensitivity: None,
        dependencies: Vec::new(),
        lifetime: None,
        optional: false,
        required: false,
        stale: false,
        provenance: BTreeMap::from([("timestamp".to_string(), provenance.clone())]),
        metadata: BTreeMap::new(),
    };
    let response = ProviderResponseMetadata {
        id: "response-1".to_string(),
        model: "provider-model".to_string(),
        created: Some(123),
        object: Some("chat.completion".to_string()),
        choice_index: Some(0),
        finish_reason: Some("stop".to_string()),
        response_message_role: Some("assistant".to_string()),
        field_states: BTreeMap::from([("tool_calls".to_string(), ProviderFieldState::Null)]),
    };
    let trace = RequestTrace {
        format_version: TRACE_FORMAT_VERSION,
        request_id: "request".to_string(),
        session_id: None,
        timestamp: None,
        provider: "provider".to_string(),
        model: "model".to_string(),
        evidence_schema_version: Some(1),
        blocks: vec![block],
        usage: None,
        provider_response: Some(response),
        latency: None,
        provenance: BTreeMap::from([("provider_response".to_string(), provenance)]),
        metadata: BTreeMap::new(),
    };

    let encoded = serde_json::to_value(&trace).unwrap();
    assert_eq!(encoded["evidence_schema_version"], 1);
    assert_eq!(encoded["blocks"][0]["timestamp"], 123.5);
    assert_eq!(
        encoded["blocks"][0]["provenance"]["timestamp"]["origin"],
        "source_explicit"
    );
    assert_eq!(encoded["provider_response"]["id"], "response-1");
    assert_eq!(
        encoded["blocks"][0]["dependencies"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        encoded["provider_response"]["field_states"]["tool_calls"],
        "null"
    );
}
