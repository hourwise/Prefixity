//! Provider/runtime-neutral structural request diagnostics.
//!
//! These diagnostics describe differences between neutral conformance
//! requests. They do not predict cache hits, misses, invalidation, or runtime
//! performance.

use crate::conformance::{
    ConformanceRequest, ContextArtifactInput, OrderedJsonObject, ToolDefinition,
};
use crate::error::BenchmarkError;
use crate::hashing::{canonical_hash, hash_text};
use prefixity_core::observation::Observed;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

pub const PREFIX_DIFF_SCHEMA_ID: &str = "prefixity.prefix-diff";
pub const PREFIX_DIFF_SCHEMA_VERSION: u32 = 1;
pub const ENVELOPE_DIFF_SCHEMA_ID: &str = "prefixity.request-envelope-diff";
pub const ENVELOPE_DIFF_SCHEMA_VERSION: u32 = 1;
pub const REQUEST_DIFF_SCHEMA_ID: &str = "prefixity.request-diff";
pub const REQUEST_DIFF_SCHEMA_VERSION: u32 = 1;

const MAX_CHANGES: usize = 256;
const MAX_PREVIEW_BYTES: usize = 64;

/// A deliberately conservative cache interpretation for a structural diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CacheImpactAssessment {
    /// No runtime/provider evidence is attached to this diagnostic.
    #[default]
    Unknown,
    /// The compared dimension does not apply to the requested diagnostic.
    NotApplicable,
    /// Reserved for a later evidence-attached result; P0-L7 does not produce it.
    EvidenceSupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffState {
    Identical,
    Changed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeCategory {
    TextContentChanged,
    ContentAdded,
    ContentRemoved,
    ArtifactAdded,
    ArtifactRemoved,
    ArtifactOrderChanged,
    ArtifactContentChanged,
    ToolAdded,
    ToolRemoved,
    ToolOrderChanged,
    ToolDefinitionChanged,
    OptionalSchemaFieldAdded,
    OptionalSchemaFieldRemoved,
    OrderedSchemaFieldChanged,
    JsonStructureChanged,
    ValueChanged,
    PresenceChanged,
}

/// A bounded description of a value involved in a difference. Full content is
/// intentionally absent; the fingerprint and size are sufficient for audit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValueSummary {
    pub value_type: String,
    pub fingerprint: String,
    pub size_bytes: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiffChange {
    pub path: String,
    pub category: ChangeCategory,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left: Option<ValueSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right: Option<ValueSummary>,
    pub order_changed: bool,
    pub content_changed: bool,
    pub presence_changed: bool,
    pub whitespace_only: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sequence_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextCommonPrefix {
    pub path: String,
    pub left_bytes: usize,
    pub right_bytes: usize,
    pub common_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommonPrefixMeasurement {
    pub structural_units: usize,
    pub artifact_units: usize,
    pub tool_units: usize,
    pub text: Vec<TextCommonPrefix>,
    /// Token-level prefix measurement is not available without a tokenizer.
    pub token_units: Observed<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrefixDiff {
    pub schema_id: String,
    pub schema_version: u32,
    pub left_context_fingerprint: String,
    pub right_context_fingerprint: String,
    pub identical: bool,
    pub common_prefix: CommonPrefixMeasurement,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_divergence: Option<DiffChange>,
    pub changes: Vec<DiffChange>,
    pub provenance: BTreeMap<String, String>,
    pub cache_impact: CacheImpactAssessment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvelopeField {
    Model,
    Reasoning,
    ResponseFormat,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvelopeChange {
    pub path: String,
    pub field: EnvelopeField,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left: Option<ValueSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right: Option<ValueSummary>,
    pub presence_changed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvelopeDiff {
    pub schema_id: String,
    pub schema_version: u32,
    pub left_request_fingerprint: String,
    pub right_request_fingerprint: String,
    pub identical: bool,
    pub changes: Vec<EnvelopeChange>,
    pub cache_impact: CacheImpactAssessment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestDiffInterpretation {
    pub context: DiffState,
    pub envelope: DiffState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestDiff {
    pub schema_id: String,
    pub schema_version: u32,
    pub left_request_fingerprint: String,
    pub right_request_fingerprint: String,
    pub prefix_diff: PrefixDiff,
    pub envelope_diff: EnvelopeDiff,
    pub interpretation: RequestDiffInterpretation,
    pub cache_impact: CacheImpactAssessment,
}

/// Compare model-visible context only.
pub fn prefix_diff(
    left: &ConformanceRequest,
    right: &ConformanceRequest,
) -> Result<PrefixDiff, BenchmarkError> {
    left.validate()?;
    right.validate()?;
    let left_fingerprint = left.context_fingerprint()?;
    let right_fingerprint = right.context_fingerprint()?;
    let mut changes = Vec::new();
    let mut text = Vec::new();

    compare_text(
        "context.system_instruction",
        &left.context.system_instruction,
        &right.context.system_instruction,
        ChangeCategory::TextContentChanged,
        None,
        &mut changes,
        &mut text,
    );
    compare_artifacts(
        &left.context.artifacts,
        &right.context.artifacts,
        &mut changes,
        &mut text,
    );
    compare_text(
        "context.current_user",
        &left.context.user_content,
        &right.context.user_content,
        ChangeCategory::TextContentChanged,
        None,
        &mut changes,
        &mut text,
    );
    compare_tools(
        &left.context.tools,
        &right.context.tools,
        &mut changes,
        &mut text,
    );
    ensure_change_bound(&changes)?;

    let left_units = context_units(left);
    let right_units = context_units(right);
    let structural_units = common_len(&left_units, &right_units);
    let artifact_units = common_artifact_units(&left.context.artifacts, &right.context.artifacts);
    let tool_units = common_tool_units(&left.context.tools, &right.context.tools);
    let identical = left_fingerprint == right_fingerprint;

    Ok(PrefixDiff {
        schema_id: PREFIX_DIFF_SCHEMA_ID.to_string(),
        schema_version: PREFIX_DIFF_SCHEMA_VERSION,
        left_context_fingerprint: left_fingerprint,
        right_context_fingerprint: right_fingerprint,
        identical,
        common_prefix: CommonPrefixMeasurement {
            structural_units,
            artifact_units,
            tool_units,
            text,
            token_units: Observed::NotObserved,
        },
        first_divergence: changes.first().cloned(),
        changes,
        provenance: BTreeMap::from([
            (
                "source".to_string(),
                "neutral-request-structure".to_string(),
            ),
            ("cache_semantics".to_string(), "not_observed".to_string()),
        ]),
        cache_impact: CacheImpactAssessment::Unknown,
    })
}

/// Compare non-context request settings only.
pub fn envelope_diff(
    left: &ConformanceRequest,
    right: &ConformanceRequest,
) -> Result<EnvelopeDiff, BenchmarkError> {
    left.validate()?;
    right.validate()?;
    let left_fingerprint = left.request_fingerprint()?;
    let right_fingerprint = right.request_fingerprint()?;
    let fields = [
        (
            EnvelopeField::Model,
            "envelope.model",
            Some(Value::String(left.envelope.model.clone())),
            Some(Value::String(right.envelope.model.clone())),
        ),
        (
            EnvelopeField::Reasoning,
            "envelope.reasoning",
            left.envelope
                .reasoning
                .as_ref()
                .map(|value| serde_json::to_value(value).expect("reasoning serializes")),
            right
                .envelope
                .reasoning
                .as_ref()
                .map(|value| serde_json::to_value(value).expect("reasoning serializes")),
        ),
        (
            EnvelopeField::ResponseFormat,
            "envelope.response_format",
            left.envelope
                .response_format
                .as_ref()
                .map(|value| serde_json::to_value(value).expect("response format serializes")),
            right
                .envelope
                .response_format
                .as_ref()
                .map(|value| serde_json::to_value(value).expect("response format serializes")),
        ),
    ];
    let changes = fields
        .into_iter()
        .filter_map(|(field, path, left_value, right_value)| {
            if left_value == right_value {
                None
            } else {
                Some(EnvelopeChange {
                    path: path.to_string(),
                    field,
                    left: left_value.as_ref().map(value_summary),
                    right: right_value.as_ref().map(value_summary),
                    presence_changed: left_value.is_none() != right_value.is_none(),
                })
            }
        })
        .collect::<Vec<_>>();
    Ok(EnvelopeDiff {
        schema_id: ENVELOPE_DIFF_SCHEMA_ID.to_string(),
        schema_version: ENVELOPE_DIFF_SCHEMA_VERSION,
        left_request_fingerprint: left_fingerprint,
        right_request_fingerprint: right_fingerprint,
        identical: changes.is_empty(),
        changes,
        cache_impact: CacheImpactAssessment::Unknown,
    })
}

/// Compare context and envelope independently for one request pair.
pub fn request_diff(
    left: &ConformanceRequest,
    right: &ConformanceRequest,
) -> Result<RequestDiff, BenchmarkError> {
    let prefix = prefix_diff(left, right)?;
    let envelope = envelope_diff(left, right)?;
    let context = if prefix.identical {
        DiffState::Identical
    } else {
        DiffState::Changed
    };
    let envelope_state = if envelope.identical {
        DiffState::Identical
    } else {
        DiffState::Changed
    };
    let left_request_fingerprint = left.request_fingerprint()?;
    let right_request_fingerprint = right.request_fingerprint()?;
    Ok(RequestDiff {
        schema_id: REQUEST_DIFF_SCHEMA_ID.to_string(),
        schema_version: REQUEST_DIFF_SCHEMA_VERSION,
        left_request_fingerprint,
        right_request_fingerprint,
        prefix_diff: prefix,
        envelope_diff: envelope,
        interpretation: RequestDiffInterpretation {
            context,
            envelope: envelope_state,
        },
        cache_impact: CacheImpactAssessment::Unknown,
    })
}

fn compare_artifacts(
    left: &[ContextArtifactInput],
    right: &[ContextArtifactInput],
    changes: &mut Vec<DiffChange>,
    text: &mut Vec<TextCommonPrefix>,
) {
    let left_ids = left
        .iter()
        .map(|item| item.artifact_id.as_str())
        .collect::<Vec<_>>();
    let right_ids = right
        .iter()
        .map(|item| item.artifact_id.as_str())
        .collect::<Vec<_>>();
    let left_set = left_ids.iter().copied().collect::<BTreeSet<_>>();
    let right_set = right_ids.iter().copied().collect::<BTreeSet<_>>();
    if left_set == right_set && left_ids != right_ids {
        let index = first_difference_index(&left_ids, &right_ids);
        push_change(
            changes,
            DiffChange {
                path: "context.artifacts".to_string(),
                category: ChangeCategory::ArtifactOrderChanged,
                left: Some(sequence_summary(&left_ids)),
                right: Some(sequence_summary(&right_ids)),
                order_changed: true,
                content_changed: false,
                presence_changed: false,
                whitespace_only: false,
                sequence_index: index,
            },
        );
    }
    for (index, artifact) in left.iter().enumerate() {
        if !right_set.contains(artifact.artifact_id.as_str()) {
            push_change(
                changes,
                presence_change(
                    format!("context.artifacts[{index}]"),
                    ChangeCategory::ArtifactRemoved,
                    Some(summary_text(&artifact.content)),
                    None,
                    Some(index),
                ),
            );
        }
    }
    for (index, artifact) in right.iter().enumerate() {
        if !left_set.contains(artifact.artifact_id.as_str()) {
            push_change(
                changes,
                presence_change(
                    format!("context.artifacts[{index}]"),
                    ChangeCategory::ArtifactAdded,
                    None,
                    Some(summary_text(&artifact.content)),
                    Some(index),
                ),
            );
        }
    }
    let right_by_id = right
        .iter()
        .enumerate()
        .map(|(index, item)| (item.artifact_id.as_str(), (index, item)))
        .collect::<BTreeMap<_, _>>();
    for artifact in left {
        if let Some((right_index, other)) = right_by_id.get(artifact.artifact_id.as_str()) {
            compare_text(
                format!("context.artifacts[{right_index}].content"),
                &artifact.content,
                &other.content,
                ChangeCategory::ArtifactContentChanged,
                Some(*right_index),
                changes,
                text,
            );
        }
    }
}

fn compare_tools(
    left: &[ToolDefinition],
    right: &[ToolDefinition],
    changes: &mut Vec<DiffChange>,
    text: &mut Vec<TextCommonPrefix>,
) {
    let left_names = left
        .iter()
        .map(|item| item.name.as_str())
        .collect::<Vec<_>>();
    let right_names = right
        .iter()
        .map(|item| item.name.as_str())
        .collect::<Vec<_>>();
    let left_set = left_names.iter().copied().collect::<BTreeSet<_>>();
    let right_set = right_names.iter().copied().collect::<BTreeSet<_>>();
    if left_set == right_set && left_names != right_names {
        let index = first_difference_index(&left_names, &right_names);
        push_change(
            changes,
            DiffChange {
                path: "context.tools".to_string(),
                category: ChangeCategory::ToolOrderChanged,
                left: Some(sequence_summary(&left_names)),
                right: Some(sequence_summary(&right_names)),
                order_changed: true,
                content_changed: false,
                presence_changed: false,
                whitespace_only: false,
                sequence_index: index,
            },
        );
    }
    for (index, tool) in left.iter().enumerate() {
        if !right_set.contains(tool.name.as_str()) {
            push_change(
                changes,
                presence_change(
                    format!("context.tools[{index}]"),
                    ChangeCategory::ToolRemoved,
                    Some(summary_text(&tool.description)),
                    None,
                    Some(index),
                ),
            );
        }
    }
    for (index, tool) in right.iter().enumerate() {
        if !left_set.contains(tool.name.as_str()) {
            push_change(
                changes,
                presence_change(
                    format!("context.tools[{index}]"),
                    ChangeCategory::ToolAdded,
                    None,
                    Some(summary_text(&tool.description)),
                    Some(index),
                ),
            );
        }
    }
    let right_by_name = right
        .iter()
        .enumerate()
        .map(|(index, item)| (item.name.as_str(), (index, item)))
        .collect::<BTreeMap<_, _>>();
    for tool in left {
        if let Some((right_index, other)) = right_by_name.get(tool.name.as_str()) {
            compare_text(
                format!("context.tools[{right_index}].description"),
                &tool.description,
                &other.description,
                ChangeCategory::ToolDefinitionChanged,
                Some(*right_index),
                changes,
                text,
            );
            compare_ordered_fields(
                &format!("context.tools[{right_index}].parameters.fields"),
                &tool.parameters,
                &other.parameters,
                changes,
            );
        }
    }
}

fn compare_ordered_fields(
    path: &str,
    left: &OrderedJsonObject,
    right: &OrderedJsonObject,
    changes: &mut Vec<DiffChange>,
) {
    let left_names = left
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<Vec<_>>();
    let right_names = right
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<Vec<_>>();
    let left_set = left_names.iter().copied().collect::<BTreeSet<_>>();
    let right_set = right_names.iter().copied().collect::<BTreeSet<_>>();
    if left_set == right_set && left_names != right_names {
        push_change(
            changes,
            DiffChange {
                path: path.to_string(),
                category: ChangeCategory::OrderedSchemaFieldChanged,
                left: Some(sequence_summary(&left_names)),
                right: Some(sequence_summary(&right_names)),
                order_changed: true,
                content_changed: false,
                presence_changed: false,
                whitespace_only: false,
                sequence_index: first_difference_index(&left_names, &right_names),
            },
        );
    }
    for (index, field) in left.fields.iter().enumerate() {
        if !right_set.contains(field.name.as_str()) {
            push_change(
                changes,
                presence_change(
                    format!("{path}[{index}]"),
                    ChangeCategory::OptionalSchemaFieldRemoved,
                    Some(value_summary(&field.value)),
                    None,
                    Some(index),
                ),
            );
        }
    }
    for (index, field) in right.fields.iter().enumerate() {
        if !left_set.contains(field.name.as_str()) {
            push_change(
                changes,
                presence_change(
                    format!("{path}[{index}]"),
                    ChangeCategory::OptionalSchemaFieldAdded,
                    None,
                    Some(value_summary(&field.value)),
                    Some(index),
                ),
            );
        }
    }
    let left_by_name = left
        .fields
        .iter()
        .map(|field| (field.name.as_str(), field))
        .collect::<BTreeMap<_, _>>();
    for (index, field) in right.fields.iter().enumerate() {
        if let Some(previous) = left_by_name.get(field.name.as_str()) {
            if previous.value != field.value {
                push_change(
                    changes,
                    value_change(
                        format!("{path}[{index}].value"),
                        if json_shape(&previous.value) != json_shape(&field.value) {
                            ChangeCategory::JsonStructureChanged
                        } else {
                            ChangeCategory::ValueChanged
                        },
                        Some(value_summary(&previous.value)),
                        Some(value_summary(&field.value)),
                        Some(index),
                    ),
                );
            }
        }
    }
}

fn compare_text(
    path: impl Into<String>,
    left: &str,
    right: &str,
    category: ChangeCategory,
    sequence_index: Option<usize>,
    changes: &mut Vec<DiffChange>,
    text: &mut Vec<TextCommonPrefix>,
) {
    let path = path.into();
    let common_bytes = common_byte_prefix(left, right);
    text.push(TextCommonPrefix {
        path: path.clone(),
        left_bytes: left.len(),
        right_bytes: right.len(),
        common_bytes,
    });
    if left != right {
        push_change(
            changes,
            DiffChange {
                path,
                category,
                left: Some(summary_text(left)),
                right: Some(summary_text(right)),
                order_changed: false,
                content_changed: true,
                presence_changed: false,
                whitespace_only: whitespace_only_change(left, right),
                sequence_index,
            },
        );
    }
}

fn context_units(request: &ConformanceRequest) -> Vec<String> {
    let mut units = vec![format!(
        "system:{}",
        hash_text(&request.context.system_instruction)
    )];
    units.extend(request.context.artifacts.iter().map(|artifact| {
        format!(
            "artifact:{}:{}",
            artifact.artifact_id,
            hash_text(&artifact.content)
        )
    }));
    units.push(format!("user:{}", hash_text(&request.context.user_content)));
    units.extend(request.context.tools.iter().map(|tool| {
        format!(
            "tool:{}:{}",
            tool.name,
            canonical_hash(tool).unwrap_or_else(|_| "invalid".to_string())
        )
    }));
    units
}

fn common_artifact_units(left: &[ContextArtifactInput], right: &[ContextArtifactInput]) -> usize {
    left.iter().zip(right).take_while(|(a, b)| a == b).count()
}

fn common_tool_units(left: &[ToolDefinition], right: &[ToolDefinition]) -> usize {
    left.iter().zip(right).take_while(|(a, b)| a == b).count()
}

fn common_len(left: &[String], right: &[String]) -> usize {
    left.iter().zip(right).take_while(|(a, b)| a == b).count()
}

fn common_byte_prefix(left: &str, right: &str) -> usize {
    left.as_bytes()
        .iter()
        .zip(right.as_bytes())
        .take_while(|(a, b)| a == b)
        .count()
}

fn first_difference_index(left: &[&str], right: &[&str]) -> Option<usize> {
    left.iter()
        .zip(right)
        .position(|(a, b)| a != b)
        .or_else(|| (left.len() != right.len()).then_some(left.len().min(right.len())))
}

fn sequence_summary<T: Serialize>(value: &T) -> ValueSummary {
    let value = serde_json::to_value(value).unwrap_or(Value::Null);
    value_summary(&value)
}

fn summary_text(value: &str) -> ValueSummary {
    ValueSummary {
        value_type: "string".to_string(),
        fingerprint: hash_text(value),
        size_bytes: value.len(),
        preview: bounded_preview(value),
    }
}

fn value_summary(value: &Value) -> ValueSummary {
    let serialized = serde_json::to_vec(value).unwrap_or_default();
    ValueSummary {
        value_type: value_type(value).to_string(),
        fingerprint: canonical_hash(value).unwrap_or_else(|_| hash_text("<invalid-json>")),
        size_bytes: serialized.len(),
        preview: None,
    }
}

fn bounded_preview(value: &str) -> Option<String> {
    if value.len() <= MAX_PREVIEW_BYTES {
        Some(value.to_string())
    } else {
        let mut end = MAX_PREVIEW_BYTES;
        while !value.is_char_boundary(end) {
            end -= 1;
        }
        Some(format!("{}…[truncated]", &value[..end]))
    }
}

fn value_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn json_shape(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut shape = Map::new();
            for (key, value) in object {
                shape.insert(key.clone(), json_shape(value));
            }
            Value::Object(shape)
        }
        Value::Array(values) => Value::Array(values.iter().map(json_shape).collect()),
        scalar => Value::String(value_type(scalar).to_string()),
    }
}

fn whitespace_only_change(left: &str, right: &str) -> bool {
    left.chars()
        .filter(|character| !character.is_whitespace())
        .eq(right.chars().filter(|character| !character.is_whitespace()))
}

fn value_change(
    path: String,
    category: ChangeCategory,
    left: Option<ValueSummary>,
    right: Option<ValueSummary>,
    sequence_index: Option<usize>,
) -> DiffChange {
    DiffChange {
        path,
        category,
        left,
        right,
        order_changed: false,
        content_changed: true,
        presence_changed: false,
        whitespace_only: false,
        sequence_index,
    }
}

fn presence_change(
    path: String,
    category: ChangeCategory,
    left: Option<ValueSummary>,
    right: Option<ValueSummary>,
    sequence_index: Option<usize>,
) -> DiffChange {
    DiffChange {
        path,
        category,
        left,
        right,
        order_changed: false,
        content_changed: false,
        presence_changed: true,
        whitespace_only: false,
        sequence_index,
    }
}

fn push_change(changes: &mut Vec<DiffChange>, change: DiffChange) {
    if changes.len() <= MAX_CHANGES {
        changes.push(change);
    }
}

fn ensure_change_bound(changes: &[DiffChange]) -> Result<(), BenchmarkError> {
    if changes.len() > MAX_CHANGES {
        return Err(BenchmarkError::validation(
            "request diff exceeds its bounded change limit",
        ));
    }
    Ok(())
}
