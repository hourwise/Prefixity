use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;

const WORKFLOW: &str = r#"# CP10 synthetic observation publication workflow, version 1

This closed benchmark world contains twelve generated readings from two fictional
laboratory instruments. It contains no real people, facilities, or locations.
All source records are immutable and all actions are read-only.

## Governing specification and completeness

`source/field-dictionary-aggregation-v1.json` is the complete versioned
publication specification. It defines all input fields, units, null handling,
quality flags, timestamp normalization, UTC grouping, output columns,
integer rounding, and quality dispositions. It identifies the three and only
three source shards in this frozen publication collection. The specification
is complete because the declared schema accounts for every field in each
shard and the output recipe resolves every valid observation using only those
fields and rules. No additional source, inferred field, or narrative policy is
needed.

## Ordinary publication sequence

The publication desk first attaches the governing dictionary and the complete
source collection. A catalog inspection records all shard IDs, content hashes,
and row counts. The second desk step evaluates nulls, source quality flags,
timestamp offsets, and coverage against the same immutable collection. Neither
step changes any source record.

When the standard publication packet is assembled, its cover sheet carries
the catalog and quality receipts and attaches the effective field dictionary
again so that the publication record travels with the exact schema used to
interpret its rows. The packet recipe emits one native specification message
for every publication, regardless of document size, arm, or benchmark state.
The later occurrence resolves to the same content-addressed specification
revision as the original. It does not transform or summarize that object.

## Deterministic checks and publication

The finite first-slot action menu offers catalog inspection or a shard-count
cross-check. The second-slot menu offers quality/gap inspection or timezone
and unit verification. Each action has a fixed receipt bound to its input
hashes and the unchanged publication world revision.

The final rollup normalizes RFC3339 offsets to UTC, excludes a null measurement
under `QF-01`, excludes every non-`VALID` quality flag under `QF-02`, and groups
remaining integer tenths of Celsius by instrument and UTC date. Means use
round-half-up integer arithmetic. Excluded observations remain visible in the
flag list. The hidden evaluator checks the exact rows, flags, source IDs,
quality rules, publication status, and reason. Its key is stored outside every
prompt and planner-visible asset.

## Provenance limits

The generator uses fixed synthetic inputs and performs no network, tokenizer,
model, or inference operation. This workflow defines its bounded records and
ordinary packet behavior independently of any byte or token count. No size
derived admission claim is made.
"#;

const SYSTEM: &str = "You are preparing a deterministic publication from a closed synthetic observation collection. At request slots 1 and 2, return exactly one JSON object with the single string field action_id and choose from the displayed finite read-only menu. At request slot 3, return the exact requested structured rollup using only the unchanged source records and governing field dictionary. Do not invent observations, identifiers, measurements, rules, or locations.\n";
const TASK: &str = "CP10 synthetic observation-data publication. Normalize the frozen instrument readings, apply the versioned field dictionary and quality rules, and prepare the UTC daily publication. All source records are synthetic, immutable, and read-only.\n";
const REQUEST1: &str = "Inspect the complete publication source catalog against the governing specification. Choose one read-only action: {\"action_id\":\"inspect_observation_catalog\"} or {\"action_id\":\"verify_shard_inventory\"}.\n";
const REQUEST2: &str = "Using the unchanged collection and the prior catalog receipt, inspect quality flags, missing values, UTC date coverage, and eligible measurements. Choose one read-only action: {\"action_id\":\"inspect_quality_and_coverage\"} or {\"action_id\":\"verify_timezone_and_units\"}.\n";
const REQUEST3: &str = "Return exact JSON with publication_status, dataset_id, output_rows, flagged_observations, source_ids, quality_rule_ids, and publication_reason. Each output row must contain instrument_id, utc_date, unit, valid_observation_count, mean_value_celsius_tenths, minimum_value_celsius_tenths, maximum_value_celsius_tenths, and source_observation_ids. Each flagged observation must contain observation_id, normalized_utc, quality_flag, rule_id, and disposition. Preserve sorted source IDs and observation IDs. Means use integer round-half-up arithmetic.\n";

const WORLD_REVISION: &str = "observation-publication@OBS-SYN-CP10-v1";
const SPEC_REVISION: &str = "field-dictionary-aggregation@1.0.0";
const SPEC_LOCATOR: &str = "cp10/source/field-dictionary-aggregation-v1.json";
const SHARD_REVISIONS: [&str; 3] = [
    "observation-shard@OBS-SYN-CP10-S01-v1",
    "observation-shard@OBS-SYN-CP10-S02-v1",
    "observation-shard@OBS-SYN-CP10-S03-v1",
];
const SHARD_IDS: [&str; 3] = ["OBS-SYN-CP10-S01", "OBS-SYN-CP10-S02", "OBS-SYN-CP10-S03"];
const SHARD_SOURCE_PATHS: [&str; 3] = [
    "source/shard-OBS-SYN-CP10-S01-v1.json",
    "source/shard-OBS-SYN-CP10-S02-v1.json",
    "source/shard-OBS-SYN-CP10-S03-v1.json",
];
const SHARD_BODY_PATHS: [&str; 3] = [
    "bodies/shard-OBS-SYN-CP10-S01-v1.json",
    "bodies/shard-OBS-SYN-CP10-S02-v1.json",
    "bodies/shard-OBS-SYN-CP10-S03-v1.json",
];

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn json_bytes(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).expect("fixed fixture JSON serializes");
    bytes.push(b'\n');
    bytes
}

fn provenance(locator: &str, revision: &str, bytes: &[u8]) -> Value {
    json!({
        "source_kind":"self_authored",
        "classification":"CAPTURED_EXPLICIT",
        "source_locator":locator,
        "source_revision":revision,
        "content_hash":digest(bytes)
    })
}

fn asset(
    id: &str,
    path: &str,
    kind: &str,
    event: Option<&str>,
    revision: Option<&str>,
    body: &[u8],
) -> Value {
    let mut value = json!({"asset_id":id,"relative_path":path,"sha256":digest(body),"kind":kind});
    if let Some(event_id) = event {
        value["event_id"] = json!(event_id);
    }
    if let Some(revision_id) = revision {
        value["revision_id"] = json!(revision_id);
    }
    value
}

#[allow(clippy::too_many_arguments)]
fn message_event(
    id: &str,
    seq: u32,
    context_id: &str,
    body_hash: &str,
    source_revision: &str,
    world_revision: &str,
    refs: &[&str],
    locator: &str,
    source_hash: &str,
    workflow_hash: &str,
) -> Value {
    json!({
        "event_id":id,"sequence_index":seq,"event_type":"message","actor_role":"user",
        "parent_event_ids":[],"reference_event_ids":refs,"action":null,"result":null,
        "context_block_id":context_id,"world_state_revision":world_revision,
        "order":{"logical_tick":seq,"timestamp_origin":"derived_structural"},
        "content_hash":body_hash,
        "provenance":[
            {"source_kind":"self_authored","classification":"CAPTURED_EXPLICIT","source_locator":locator,"source_revision":source_revision,"content_hash":source_hash},
            {"source_kind":"self_authored","classification":"CAPTURED_EXPLICIT","source_locator":"cp10/workflow.md#ordinary-publication-packet-provenance","source_revision":"cp10-observation-publication-workflow-v1","content_hash":workflow_hash}
        ]
    })
}

#[allow(clippy::too_many_arguments)]
fn action_event(
    id: &str,
    seq: u32,
    action_id: &str,
    refs: &[&str],
    world_revision: &str,
    workflow_hash: &str,
    spec_hash: &str,
    collection_hash: &str,
) -> Value {
    let args = json!({"dataset_id":"OBS-SYN-CP10","specification_sha256":spec_hash,"collection_manifest_sha256":collection_hash});
    json!({
        "event_id":id,"sequence_index":seq,"event_type":"action","actor_role":"agent",
        "parent_event_ids":[],"reference_event_ids":refs,
        "action":{"action_id":action_id,"tool_name":action_id,"argument_hash":digest(&serde_json::to_vec(&args).expect("action arguments serialize"))},
        "result":null,"context_block_id":null,"world_state_revision":world_revision,
        "order":{"logical_tick":seq,"timestamp_origin":"derived_structural"},"content_hash":null,
        "provenance":[{"source_kind":"self_authored","classification":"CAPTURED_EXPLICIT","source_locator":"cp10/workflow.md#read-only-action-menu","source_revision":"cp10-observation-publication-workflow-v1","content_hash":workflow_hash}]
    })
}

#[allow(clippy::too_many_arguments)]
fn result_event(
    id: &str,
    seq: u32,
    result_id: &str,
    action_id: &str,
    receipt_hash: &str,
    refs: &[&str],
    world_revision: &str,
    receipt_path: &str,
    receipt_revision: &str,
) -> Value {
    json!({
        "event_id":id,"sequence_index":seq,"event_type":"result","actor_role":"tool",
        "parent_event_ids":[],"reference_event_ids":refs,"action":null,
        "result":{"result_id":result_id,"originating_action_id":action_id,"observation_hash":receipt_hash,"status":"success"},
        "context_block_id":null,"world_state_revision":world_revision,
        "order":{"logical_tick":seq,"timestamp_origin":"derived_structural"},"content_hash":null,
        "provenance":[{"source_kind":"self_authored","classification":"CAPTURED_EXPLICIT","source_locator":receipt_path,"source_revision":receipt_revision,"content_hash":receipt_hash}]
    })
}

fn relation(
    id: &str,
    kind: &str,
    from: &str,
    to: &str,
    locator: &str,
    revision: &str,
    hash: &str,
) -> Value {
    json!({
        "relation_id":id,"relation_type":kind,"from_id":from,"to_id":to,"scope":"scenario_local",
        "semantics_version":"controlled-benchmark-relations-v1",
        "provenance":[{"source_kind":"self_authored","classification":"CAPTURED_EXPLICIT","source_locator":locator,"source_revision":revision,"content_hash":hash}]
    })
}

fn message(role: &str, parts: Vec<Value>) -> Value {
    json!({"role":role,"parts":parts})
}
fn asset_part(id: &str) -> Value {
    json!({"kind":"asset","asset_id":id})
}
fn event_part(id: &str) -> Value {
    json!({"kind":"event_body","event_id":id})
}

fn specification() -> Value {
    json!({
        "schema_id":"prefixity.claim2.synthetic-observation-field-dictionary-aggregation-specification",
        "schema_version":1,"specification_id":"FIELD-DICT-SYN-CP10-1.0.0","revision":SPEC_REVISION,
        "dataset_id":"OBS-SYN-CP10","synthetic_only":true,"world_state_revision":WORLD_REVISION,
        "collection":{"shard_ids":SHARD_IDS,"shard_count":3,"expected_observation_count":12,"closed_world":true},
        "fields":[
            {"field_id":"observation_id","type":"string","required":true,"constraints":"unique within dataset"},
            {"field_id":"instrument_id","type":"string","required":true,"allowed_values":["LAB-SYN-CHAMBER-A","LAB-SYN-CHAMBER-B"]},
            {"field_id":"observed_at","type":"RFC3339 timestamp","required":true,"timezone_rule":"offset is mandatory; normalize the represented instant to UTC before grouping or flag reporting"},
            {"field_id":"value_celsius_tenths","type":"integer or null","unit":"0.1 degree Celsius","required":true,"null_rule":"exclude and flag under QF-01"},
            {"field_id":"quality_flag","type":"enum","required":true,"allowed_values":["VALID","SUSPECT"]}
        ],
        "quality_rules":[
            {"rule_id":"QF-01","when":"value_celsius_tenths is null","disposition":"exclude; flag NULL_MEASUREMENT"},
            {"rule_id":"QF-02","when":"quality_flag is not VALID and value_celsius_tenths is not null","disposition":"exclude; flag NON_VALID_SOURCE_QUALITY"}
        ],
        "aggregation":{"group_by":["instrument_id","UTC calendar date of observed_at"],"eligible":"non-null value_celsius_tenths and quality_flag VALID","output_unit":"0.1 degree Celsius","mean":"sum integer tenths divided by count, rounded to nearest integer with exact halves away from zero (round-half-up for nonnegative observations)","minimum":"minimum eligible value_celsius_tenths","maximum":"maximum eligible value_celsius_tenths","source_observation_ids":"lexicographically sorted eligible observations in each group"},
        "output_schema":{"publication_status":["PUBLISHED","PUBLISHED_WITH_FLAGS","HELD_NO_ELIGIBLE_ROWS"],"row_fields":["instrument_id","utc_date","unit","valid_observation_count","mean_value_celsius_tenths","minimum_value_celsius_tenths","maximum_value_celsius_tenths","source_observation_ids"],"flag_fields":["observation_id","normalized_utc","quality_flag","rule_id","disposition"]},
        "rounding":{"representation":"integer Celsius tenths","aggregation_intermediate":"integer sum and count","mean_tie_rule":"round-half-up; no binary floating point"}
    })
}

fn shards() -> [Value; 3] {
    [
        json!({
            "schema_id":"prefixity.claim2.synthetic-observation-shard","schema_version":1,
            "shard_id":SHARD_IDS[0],"dataset_id":"OBS-SYN-CP10","source_revision":SHARD_REVISIONS[0],"world_state_revision":WORLD_REVISION,"synthetic_only":true,"record_state":"immutable_frozen",
            "observations":[
                {"observation_id":"OBS-CP10-A-001","instrument_id":"LAB-SYN-CHAMBER-A","observed_at":"2026-01-14T08:00:00+01:00","value_celsius_tenths":214,"quality_flag":"VALID"},
                {"observation_id":"OBS-CP10-A-002","instrument_id":"LAB-SYN-CHAMBER-A","observed_at":"2026-01-14T08:15:00+01:00","value_celsius_tenths":216,"quality_flag":"VALID"},
                {"observation_id":"OBS-CP10-B-001","instrument_id":"LAB-SYN-CHAMBER-B","observed_at":"2026-01-14T08:00:00Z","value_celsius_tenths":198,"quality_flag":"VALID"},
                {"observation_id":"OBS-CP10-B-002","instrument_id":"LAB-SYN-CHAMBER-B","observed_at":"2026-01-14T08:15:00Z","value_celsius_tenths":204,"quality_flag":"VALID"}
            ]
        }),
        json!({
            "schema_id":"prefixity.claim2.synthetic-observation-shard","schema_version":1,
            "shard_id":SHARD_IDS[1],"dataset_id":"OBS-SYN-CP10","source_revision":SHARD_REVISIONS[1],"world_state_revision":WORLD_REVISION,"synthetic_only":true,"record_state":"immutable_frozen",
            "observations":[
                {"observation_id":"OBS-CP10-A-003","instrument_id":"LAB-SYN-CHAMBER-A","observed_at":"2026-01-14T08:30:00+01:00","value_celsius_tenths":299,"quality_flag":"SUSPECT"},
                {"observation_id":"OBS-CP10-A-004","instrument_id":"LAB-SYN-CHAMBER-A","observed_at":"2026-01-14T08:45:00+01:00","value_celsius_tenths":null,"quality_flag":"VALID"},
                {"observation_id":"OBS-CP10-B-003","instrument_id":"LAB-SYN-CHAMBER-B","observed_at":"2026-01-14T08:30:00Z","value_celsius_tenths":200,"quality_flag":"VALID"},
                {"observation_id":"OBS-CP10-B-004","instrument_id":"LAB-SYN-CHAMBER-B","observed_at":"2026-01-14T08:45:00Z","value_celsius_tenths":202,"quality_flag":"VALID"}
            ]
        }),
        json!({
            "schema_id":"prefixity.claim2.synthetic-observation-shard","schema_version":1,
            "shard_id":SHARD_IDS[2],"dataset_id":"OBS-SYN-CP10","source_revision":SHARD_REVISIONS[2],"world_state_revision":WORLD_REVISION,"synthetic_only":true,"record_state":"immutable_frozen",
            "observations":[
                {"observation_id":"OBS-CP10-A-005","instrument_id":"LAB-SYN-CHAMBER-A","observed_at":"2026-01-14T09:00:00+01:00","value_celsius_tenths":218,"quality_flag":"VALID"},
                {"observation_id":"OBS-CP10-A-006","instrument_id":"LAB-SYN-CHAMBER-A","observed_at":"2026-01-14T09:15:00+01:00","value_celsius_tenths":212,"quality_flag":"VALID"},
                {"observation_id":"OBS-CP10-B-005","instrument_id":"LAB-SYN-CHAMBER-B","observed_at":"2026-01-14T09:00:00Z","value_celsius_tenths":196,"quality_flag":"VALID"},
                {"observation_id":"OBS-CP10-B-006","instrument_id":"LAB-SYN-CHAMBER-B","observed_at":"2026-01-14T09:15:00Z","value_celsius_tenths":206,"quality_flag":"VALID"}
            ]
        }),
    ]
}

fn generated_files() -> BTreeMap<String, Vec<u8>> {
    let workflow = WORKFLOW.as_bytes().to_vec();
    let spec = json_bytes(&specification());
    let spec_hash = digest(&spec);
    let shard_values = shards();
    let shard_bytes = shard_values.each_ref().map(json_bytes);
    let shard_hashes = shard_bytes.each_ref().map(|bytes| digest(bytes));
    let collection_manifest = json_bytes(&json!({
        "schema_id":"prefixity.claim2.synthetic-observation-collection-manifest","schema_version":1,
        "dataset_id":"OBS-SYN-CP10","world_state_revision":WORLD_REVISION,"synthetic_only":true,"closed_world":true,
        "specification_sha256":spec_hash,
        "shards":(0..3).map(|i| json!({"shard_id":SHARD_IDS[i],"source_revision":SHARD_REVISIONS[i],"relative_path":SHARD_SOURCE_PATHS[i],"sha256":shard_hashes[i],"observation_count":4})).collect::<Vec<_>>(),
        "total_observations":12,"collection_state":"immutable_frozen"
    }));
    let collection_hash = digest(&collection_manifest);
    let recipe = json_bytes(&json!({
        "schema_id":"prefixity.claim2.synthetic-observation-publication-packet-recipe","schema_version":1,
        "recipe_id":"observation-publication-packet@1.0","ordinary_trigger":"publication_packet_opened_for_final_rollup",
        "benchmark_condition_used":false,
        "attachment_behavior":"attach_the_effective_field_dictionary_and_aggregation_specification_as_one_native_message_to_every_final_publication_packet",
        "specification_resolution":{"source_locator":SPEC_LOCATOR,"source_revision":SPEC_REVISION,"sha256":spec_hash,"transformation":"none"},
        "packet_contents":["source_catalog_result","quality_gap_result","their_pinned_receipts","final_publication_task","effective_field_dictionary_and_aggregation_specification"]
    }));

    let catalog_receipt = json_bytes(&json!({
        "receipt_id":"r-catalog-OBS-SYN-CP10","action_id":"inspect_observation_catalog","status":"success",
        "dataset_id":"OBS-SYN-CP10","world_state_revision":WORLD_REVISION,"specification_sha256":spec_hash,"collection_manifest_sha256":collection_hash,
        "shard_ids":SHARD_IDS,"shard_sha256":shard_hashes,"shard_observation_counts":[4,4,4],"total_observation_count":12,
        "catalog_state":"complete_closed_and_hash_verified"
    }));
    let quality_receipt = json_bytes(&json!({
        "receipt_id":"r-quality-OBS-SYN-CP10","action_id":"inspect_quality_and_coverage","status":"success",
        "dataset_id":"OBS-SYN-CP10","world_state_revision":WORLD_REVISION,"specification_sha256":spec_hash,"collection_manifest_sha256":collection_hash,
        "normalized_observation_count":12,"eligible_observation_ids":["OBS-CP10-A-001","OBS-CP10-A-002","OBS-CP10-A-005","OBS-CP10-A-006","OBS-CP10-B-001","OBS-CP10-B-002","OBS-CP10-B-003","OBS-CP10-B-004","OBS-CP10-B-005","OBS-CP10-B-006"],
        "flagged_observations":[
            {"observation_id":"OBS-CP10-A-003","normalized_utc":"2026-01-14T07:30:00Z","quality_flag":"SUSPECT","rule_id":"QF-02","disposition":"EXCLUDED_NON_VALID_SOURCE_QUALITY"},
            {"observation_id":"OBS-CP10-A-004","normalized_utc":"2026-01-14T07:45:00Z","quality_flag":"VALID","rule_id":"QF-01","disposition":"EXCLUDED_NULL_MEASUREMENT"}
        ],
        "utc_date_coverage":{"LAB-SYN-CHAMBER-A":["2026-01-14"],"LAB-SYN-CHAMBER-B":["2026-01-14"]},
        "quality_state":"ten_eligible_two_flagged_no_unresolved_gap"
    }));
    let inventory_receipt = json_bytes(&json!({
        "receipt_id":"r-inventory-OBS-SYN-CP10","action_id":"verify_shard_inventory","status":"success",
        "dataset_id":"OBS-SYN-CP10","world_state_revision":WORLD_REVISION,"collection_manifest_sha256":collection_hash,
        "present_shards":SHARD_IDS,"expected_shards":SHARD_IDS,"observation_count":6,"result":"INVENTORY_MATCH"
    }));
    let timezone_receipt = json_bytes(&json!({
        "receipt_id":"r-timezone-OBS-SYN-CP10","action_id":"verify_timezone_and_units","status":"success",
        "dataset_id":"OBS-SYN-CP10","world_state_revision":WORLD_REVISION,"specification_sha256":spec_hash,
        "all_timestamps_have_explicit_offset":true,"all_values_use_integer_celsius_tenths_or_null":true,"result":"TIMEZONE_AND_UNITS_VALID"
    }));
    let receipt_bodies = [
        catalog_receipt,
        inventory_receipt,
        quality_receipt,
        timezone_receipt,
    ];
    let receipt_hashes = receipt_bodies.each_ref().map(|bytes| digest(bytes));
    let receipt_paths = [
        "bodies/receipt-source-catalog.json",
        "bodies/receipt-shard-inventory.json",
        "bodies/receipt-quality-gaps.json",
        "bodies/receipt-timezone-units.json",
    ];
    let receipt_revisions = [
        "observation-catalog-check@OBS-SYN-CP10-v1",
        "observation-inventory-check@OBS-SYN-CP10-v1",
        "observation-quality-check@OBS-SYN-CP10-v1",
        "observation-timezone-check@OBS-SYN-CP10-v1",
    ];
    let action_ids = [
        "inspect_observation_catalog",
        "verify_shard_inventory",
        "inspect_quality_and_coverage",
        "verify_timezone_and_units",
    ];
    let action_events = [
        "e-action-catalog",
        "e-action-inventory",
        "e-action-quality",
        "e-action-timezone",
    ];
    let result_events = [
        "e-catalog-result",
        "e-inventory-result",
        "e-quality-result",
        "e-timezone-result",
    ];
    let result_ids = [
        "r-catalog-OBS-SYN-CP10",
        "r-inventory-OBS-SYN-CP10",
        "r-quality-OBS-SYN-CP10",
        "r-timezone-OBS-SYN-CP10",
    ];
    let spec_event_hash = spec_hash.clone();
    let workflow_hash = digest(&workflow);
    let mut files = BTreeMap::new();
    for (path, bytes) in [
        ("workflow.md", workflow.clone()),
        ("source/field-dictionary-aggregation-v1.json", spec.clone()),
        (
            "source/collection-manifest-v1.json",
            collection_manifest.clone(),
        ),
        ("source/publication-packet-recipe-v1.json", recipe.clone()),
        ("bodies/field-dictionary-original.json", spec.clone()),
        ("bodies/field-dictionary-final-packet.json", spec.clone()),
        ("prompt/system.txt", SYSTEM.as_bytes().to_vec()),
        ("prompt/task.txt", TASK.as_bytes().to_vec()),
        ("prompt/request1.txt", REQUEST1.as_bytes().to_vec()),
        ("prompt/request2.txt", REQUEST2.as_bytes().to_vec()),
        ("prompt/request3.txt", REQUEST3.as_bytes().to_vec()),
    ] {
        files.insert(path.to_string(), bytes);
    }
    for index in 0..3 {
        files.insert(
            SHARD_SOURCE_PATHS[index].to_string(),
            shard_bytes[index].clone(),
        );
        files.insert(
            SHARD_BODY_PATHS[index].to_string(),
            shard_bytes[index].clone(),
        );
    }
    for index in 0..4 {
        files.insert(
            receipt_paths[index].to_string(),
            receipt_bodies[index].clone(),
        );
    }

    let mut events = vec![message_event(
        "e-spec-original",
        0,
        "ctx-spec-original",
        &spec_event_hash,
        SPEC_REVISION,
        WORLD_REVISION,
        &[],
        SPEC_LOCATOR,
        &spec_hash,
        &workflow_hash,
    )];
    for index in 0..3 {
        events.push(message_event(
            ["e-shard-01", "e-shard-02", "e-shard-03"][index],
            (index + 1) as u32,
            ["ctx-shard-01", "ctx-shard-02", "ctx-shard-03"][index],
            &shard_hashes[index],
            SHARD_REVISIONS[index],
            WORLD_REVISION,
            &[],
            SHARD_SOURCE_PATHS[index],
            &shard_hashes[index],
            &workflow_hash,
        ));
    }
    let all_source_event_ids = ["e-spec-original", "e-shard-01", "e-shard-02", "e-shard-03"];
    for index in 0..4 {
        let refs = if index < 2 {
            all_source_event_ids.as_slice()
        } else {
            [
                "e-spec-original",
                "e-shard-01",
                "e-shard-02",
                "e-shard-03",
                "e-catalog-result",
            ]
            .as_slice()
        };
        let mut action = action_event(
            action_events[index],
            (4 + index * 2) as u32,
            action_ids[index],
            refs,
            WORLD_REVISION,
            &workflow_hash,
            &spec_hash,
            &collection_hash,
        );
        // The action receipt's actual action identity is the finite menu action.
        action["action"]["action_id"] = json!(action_ids[index]);
        events.push(action);
        let mut result_refs = vec![action_events[index]];
        result_refs.extend_from_slice(&[
            "e-spec-original",
            "e-shard-01",
            "e-shard-02",
            "e-shard-03",
        ]);
        if index == 2 {
            result_refs.push("e-catalog-result");
        }
        events.push(result_event(
            result_events[index],
            (5 + index * 2) as u32,
            result_ids[index],
            action_ids[index],
            &receipt_hashes[index],
            &result_refs,
            WORLD_REVISION,
            receipt_paths[index],
            receipt_revisions[index],
        ));
    }
    let mut final_occurrence = message_event(
        "e-spec-final-reattachment",
        12,
        "ctx-spec-final-publication",
        &spec_hash,
        SPEC_REVISION,
        WORLD_REVISION,
        &[
            "e-spec-original",
            "e-shard-01",
            "e-shard-02",
            "e-shard-03",
            "e-catalog-result",
            "e-quality-result",
        ],
        SPEC_LOCATOR,
        &spec_hash,
        &workflow_hash,
    );
    final_occurrence["provenance"]
        .as_array_mut()
        .unwrap()
        .push(provenance(
            "cp10/source/publication-packet-recipe-v1.json#specification-occurrence",
            "observation-publication-packet@1.0",
            &recipe,
        ));
    events.push(final_occurrence);

    let relations = vec![
        relation(
            "produce-source-catalog",
            "produces",
            action_ids[0],
            result_ids[0],
            receipt_paths[0],
            receipt_revisions[0],
            &receipt_hashes[0],
        ),
        relation(
            "produce-shard-inventory",
            "produces",
            action_ids[1],
            result_ids[1],
            receipt_paths[1],
            receipt_revisions[1],
            &receipt_hashes[1],
        ),
        relation(
            "produce-quality-gaps",
            "produces",
            action_ids[2],
            result_ids[2],
            receipt_paths[2],
            receipt_revisions[2],
            &receipt_hashes[2],
        ),
        relation(
            "produce-timezone-units",
            "produces",
            action_ids[3],
            result_ids[3],
            receipt_paths[3],
            receipt_revisions[3],
            &receipt_hashes[3],
        ),
        relation(
            "same-state-spec-reattachment",
            "same_state_revision",
            "e-spec-original",
            "e-spec-final-reattachment",
            SPEC_LOCATOR,
            SPEC_REVISION,
            &spec_hash,
        ),
    ];

    let mut assets = vec![
        asset(
            "system",
            "prompt/system.txt",
            "prompt_text",
            None,
            None,
            SYSTEM.as_bytes(),
        ),
        asset(
            "task",
            "prompt/task.txt",
            "prompt_text",
            None,
            None,
            TASK.as_bytes(),
        ),
        asset(
            "request1",
            "prompt/request1.txt",
            "prompt_text",
            None,
            None,
            REQUEST1.as_bytes(),
        ),
        asset(
            "request2",
            "prompt/request2.txt",
            "prompt_text",
            None,
            None,
            REQUEST2.as_bytes(),
        ),
        asset(
            "request3",
            "prompt/request3.txt",
            "prompt_text",
            None,
            None,
            REQUEST3.as_bytes(),
        ),
        asset(
            "spec_original",
            "bodies/field-dictionary-original.json",
            "context_attachment",
            Some("e-spec-original"),
            Some(SPEC_REVISION),
            &spec,
        ),
        asset(
            "spec_final_packet",
            "bodies/field-dictionary-final-packet.json",
            "context_attachment",
            Some("e-spec-final-reattachment"),
            Some(SPEC_REVISION),
            &spec,
        ),
    ];
    for index in 0..3 {
        assets.push(asset(
            ["shard_01", "shard_02", "shard_03"][index],
            SHARD_BODY_PATHS[index],
            "context_attachment",
            Some(["e-shard-01", "e-shard-02", "e-shard-03"][index]),
            Some(SHARD_REVISIONS[index]),
            &shard_bytes[index],
        ));
    }
    for index in 0..4 {
        assets.push(asset(
            [
                "receipt_catalog",
                "receipt_inventory",
                "receipt_quality",
                "receipt_timezone",
            ][index],
            receipt_paths[index],
            "environment_receipt",
            Some(result_events[index]),
            Some(receipt_revisions[index]),
            &receipt_bodies[index],
        ));
    }
    let slot1_messages = vec![
        message("system", vec![asset_part("system")]),
        message(
            "user",
            [
                vec![asset_part("task"), asset_part("request1")],
                vec![
                    event_part("e-spec-original"),
                    event_part("e-shard-01"),
                    event_part("e-shard-02"),
                    event_part("e-shard-03"),
                ],
            ]
            .concat(),
        ),
    ];
    let mut slot2_messages = slot1_messages.clone();
    slot2_messages.push(message(
        "assistant",
        vec![json!({"kind":"prior_assistant_output","request_slot":1})],
    ));
    slot2_messages.push(message(
        "user",
        vec![json!({"kind":"prior_environment_receipt","request_slot":1})],
    ));
    slot2_messages.push(message("user", vec![asset_part("request2")]));
    let mut slot3_messages = slot2_messages.clone();
    slot3_messages.push(message(
        "assistant",
        vec![json!({"kind":"prior_assistant_output","request_slot":2})],
    ));
    slot3_messages.push(message(
        "user",
        vec![json!({"kind":"prior_environment_receipt","request_slot":2})],
    ));
    slot3_messages.push(message("user", vec![asset_part("request3")]));
    slot3_messages.push(message(
        "user",
        vec![event_part("e-spec-final-reattachment")],
    ));

    let expected_final_answer = json!({
        "publication_status":"PUBLISHED_WITH_FLAGS","dataset_id":"OBS-SYN-CP10",
        "output_rows":[
            {"instrument_id":"LAB-SYN-CHAMBER-A","utc_date":"2026-01-14","unit":"0.1 degree Celsius","valid_observation_count":4,"mean_value_celsius_tenths":215,"minimum_value_celsius_tenths":212,"maximum_value_celsius_tenths":218,"source_observation_ids":["OBS-CP10-A-001","OBS-CP10-A-002","OBS-CP10-A-005","OBS-CP10-A-006"]},
            {"instrument_id":"LAB-SYN-CHAMBER-B","utc_date":"2026-01-14","unit":"0.1 degree Celsius","valid_observation_count":6,"mean_value_celsius_tenths":201,"minimum_value_celsius_tenths":196,"maximum_value_celsius_tenths":206,"source_observation_ids":["OBS-CP10-B-001","OBS-CP10-B-002","OBS-CP10-B-003","OBS-CP10-B-004","OBS-CP10-B-005","OBS-CP10-B-006"]}
        ],
        "flagged_observations":[
            {"observation_id":"OBS-CP10-A-003","normalized_utc":"2026-01-14T07:30:00Z","quality_flag":"SUSPECT","rule_id":"QF-02","disposition":"EXCLUDED_NON_VALID_SOURCE_QUALITY"},
            {"observation_id":"OBS-CP10-A-004","normalized_utc":"2026-01-14T07:45:00Z","quality_flag":"VALID","rule_id":"QF-01","disposition":"EXCLUDED_NULL_MEASUREMENT"}
        ],
        "source_ids":SHARD_IDS,"quality_rule_ids":["QF-01","QF-02"],
        "publication_reason":"Publish ten valid observations in two instrument-day rows. Exclude the one SUSPECT measurement under QF-02 and report the one null measurement under QF-01; preserve both flags in the publication record."
    });
    let evaluation_key = json!({
        "expected_action_ids":[action_ids[0],action_ids[2]],
        "expected_states_after_action":["source_catalog_verified","quality_gaps_verified"],
        "expected_result_event_ids":[result_events[0],result_events[2]],
        "expected_final_answer":expected_final_answer,
        "required_event_ids":["e-spec-original","e-shard-01","e-shard-02","e-shard-03","e-catalog-result","e-quality-result"],
        "required_relation_ids":["produce-source-catalog","produce-quality-gaps"],
        "critical_event_ids":["e-spec-original","e-shard-01","e-shard-02","e-shard-03","e-quality-result"]
    });
    let evaluation_bytes = json_bytes(&evaluation_key);
    let action_menu = vec![
        json!({"action_slot":1,"action_id":action_ids[0],"action_event_id":action_events[0],"result_event_id":result_events[0],"result_asset_id":"receipt_catalog","state_after":"source_catalog_verified"}),
        json!({"action_slot":1,"action_id":action_ids[1],"action_event_id":action_events[1],"result_event_id":result_events[1],"result_asset_id":"receipt_inventory","state_after":"shard_inventory_verified"}),
        json!({"action_slot":2,"action_id":action_ids[2],"action_event_id":action_events[2],"result_event_id":result_events[2],"result_asset_id":"receipt_quality","state_after":"quality_gaps_verified"}),
        json!({"action_slot":2,"action_id":action_ids[3],"action_event_id":action_events[3],"result_event_id":result_events[3],"result_asset_id":"receipt_timezone","state_after":"timezone_and_units_verified"}),
    ];
    let advancing_points = (0..2).map(|index| {
        let action_index = [0,2][index];
        let raw = format!("{{\"action_id\":\"{}\"}}",action_ids[action_index]);
        let state_after = if index == 0 { "source_catalog_verified" } else { "quality_gaps_verified" };
        json!({
            "request_slot":index+1,"expected_action_id":action_ids[action_index],"canonical_raw_utf8":raw,
            "canonical_raw_sha256":digest(raw.as_bytes()),"raw_language_cardinality":1,
            "receipt_event_id":result_events[action_index],"receipt_id":result_ids[action_index],
            "receipt_path":receipt_paths[action_index],"receipt_sha256":receipt_hashes[action_index],
            "receipt_revision":receipt_revisions[action_index],"state_after":state_after
        })
    }).collect::<Vec<_>>();
    let advancing_domain = json_bytes(&json!({
        "schema_id":"prefixity.phase1c.claim2-advancing-output-domain-v3-case","schema_version":3,
        "case_id":"CP10","protocol_id":"prefixity.phase1c.claim2-canonical-advancing-output.v1","offline_only":true,"points":advancing_points
    }));
    let manifest = json!({
        "schema_id":"prefixity.phase1c.claim2-workload-case","schema_version":1,"case_id":"CP10","kind":"positive",
        "planner_input":{"events":events,"relations":relations,"provenance":[
            provenance("cp10/workflow.md","cp10-observation-publication-workflow-v1",&workflow),
            provenance(SPEC_LOCATOR,SPEC_REVISION,&spec),
            provenance("cp10/source/collection-manifest-v1.json","observation-collection-manifest@OBS-SYN-CP10-v1",&collection_manifest),
            provenance("cp10/source/publication-packet-recipe-v1.json","observation-publication-packet@1.0",&recipe),
            provenance("cp10/source/shard-OBS-SYN-CP10-S01-v1.json",SHARD_REVISIONS[0],&shard_bytes[0]),
            provenance("cp10/source/shard-OBS-SYN-CP10-S02-v1.json",SHARD_REVISIONS[1],&shard_bytes[1]),
            provenance("cp10/source/shard-OBS-SYN-CP10-S03-v1.json",SHARD_REVISIONS[2],&shard_bytes[2])
        ]},
        "assets":assets,"action_menu":action_menu,
        "request_templates":[{"request_slot":1,"messages":slot1_messages},{"request_slot":2,"messages":slot2_messages},{"request_slot":3,"messages":slot3_messages}],
        "evaluation_key_path":"evaluation/key.json","evaluation_key_sha256":digest(&evaluation_bytes),
        "assistant_output_planning_bytes":4096,"token_proof_inputs":null
    });
    files.insert("evaluation/key.json".to_string(), evaluation_bytes);
    files.insert("case.json".to_string(), json_bytes(&manifest));
    files.insert(
        "advancing-output-domain-v3.json".to_string(),
        advancing_domain,
    );
    files
}

#[allow(dead_code)]
pub fn write_fixture(directory: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(directory)?;
    for (relative, bytes) in generated_files() {
        let path = directory.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, bytes)?;
    }
    Ok(())
}

#[allow(dead_code)]
pub fn generated_files_for_test() -> BTreeMap<String, Vec<u8>> {
    generated_files()
}
