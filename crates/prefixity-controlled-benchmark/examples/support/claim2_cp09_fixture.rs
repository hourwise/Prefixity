use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;

const WORLD: &str = "release-promotion@REL-SYN-2026-10-03-04-v1";
const MANIFEST_REV: &str = "deployment-cohort@REL-SYN-2026-10-03-04-v1";
const STATE_REV: &str = "release-state@REL-SYN-2026-10-03-04-v1";
const WORKFLOW_REV: &str = "cp09-software-release-promotion-v1";

const WORKFLOW: &str = r#"# CP09 synthetic software-release promotion workflow, version 1

This case is a wholly synthetic, offline release-board fixture. Release,
build, image, environment, operator, and rollback identifiers are invented;
the builder creates no credentials and contacts no production system,
provider, model, or network.

## Complete governing record

The release office opens one immutable deployment-cohort manifest for every
promotion review. The manifest's complete-record contract is fixed before
materialization: release identity and revision; source/build/image digests;
named environment targets and cohort membership; required gate IDs; gate
owners; rollout order; and the exact rollback artifact ID for every production
target. A manifest missing any of these fields is incomplete and cannot be
reviewed. Cohort membership is the finite set listed in the manifest; no
outside service or deployment unit is implied. The manifest is the governing
object for this release decision.

## Ordinary release-board sequence

At intake, the release coordinator attaches the governing manifest beside the
frozen release-state snapshot. A read-only provenance check resolves the
source revision, build attestation, and image digests. The next board step
checks each required rollout gate against the same frozen snapshot and pinned
provenance receipt. Both actions are finite, read-only checks; neither changes
the release, deployment state, or rollback record.

When the promotion request is assembled, the standard change-board packet
recipe carries forward the unique check receipts, preserves the frozen state,
and attaches the complete governing manifest again so the approval decision
travels with the exact release cohort and rollback authority it approves.
This recipe is used for every release-board promotion review, irrespective of
document size or any benchmark arm. The final native manifest Message is
resolved from the same content-addressed revision as the intake attachment.

## Frozen state and decision

`source/deployment-cohort-manifest-v1.json` and
`source/frozen-release-state-v1.json` define the complete, closed synthetic
world. Their world-state revision is `release-promotion@REL-SYN-2026-10-03-04-v1`.
The manifest remains immutable through all three requests. The state snapshot
pins the source commit, builder attestation, image digest, required gate
results, production canary metrics, cohort membership, and rollback artifact
identity and digest.

The first deterministic result checks release/build/image provenance against
the frozen state. The second independently checks every manifest-required
gate, canary bounds, production cohort, and rollback artifact. The structured
board answer is `PROMOTE`, `HOLD`, or `ROLLBACK`, with the exact release and
build/image identities, governing gate IDs, production cohort identity,
required rollback artifact, and reason code. Source records and receipts are
read-only evidence, not mutations or approvals.

## Provenance boundary

All facts and identifiers are synthetic. The case supports deterministic
offline evaluator checks only. It provides no claim about real deployment
safety, software supply-chain security, operational release quality, or
tokenization performance. No byte-to-token estimate was used to compose the
records or define their completeness.
"#;

const MANIFEST_SPEC: &str = r#"SYNTHETIC RELEASE OFFICE
DEPLOYMENT COHORT MANIFEST — REL-SYN-2026-10-03-04
Revision: 1.0
State: IMMUTABLE / APPROVAL REQUIRED

This complete manifest defines the only deployment units covered by this
promotion request. A valid copy contains the release identity, source/build/
image identities, environment targets, ordered rollout cohort membership,
required gate IDs and accountable owners, and the exact rollback artifact
identity for each production cohort. Missing values make the record
incomplete. The attached structured deployment record is authoritative for
field values; this page records its governing scope and review rules.

RELEASE IDENTITY

Release: REL-SYN-2026-10-03-04
Service: ledger-edge-api
Release train: 2026.10
Release revision: 4
Source revision: git:8a64e97d32f5c0146bc9d1087254ce219a76d331
Source tree digest: sha256:ac7c6ec3d3143b9af3273f014cb9308fe7c82c69a4b6f82d7c0ecde3b03a9c17
Build attestation: att-SYN-BLD-2026-1003-044
Image: registry.synthetic.invalid/ledger-edge-api:2026.10.3-4
Image digest: sha256:8f7b136c48d7721be30ca7d3f26fcb82578af930a15d4e296e7c113a2f1a4b80

ENVIRONMENT TARGETS AND COHORT ORDER

1. `staging-eu1`: synthetic staging validation target; all four service
   shards must report the same image digest before promotion eligibility.
2. `production-canary-eu1`: the first canary deployment unit receives 10
   percent of eligible production traffic; observe for 30 minutes before a
   board decision.
3. `production-wave-eu1`: the remaining 90 percent of that listed cohort;
   this is the requested production target and may start only after all gates
   pass and the rollback artifact is verified.

The bounded EU1 cohort consists of four immutable deployment units:
`ledger-edge-eu1-a`, `ledger-edge-eu1-b`, `ledger-edge-eu1-c`, and
`ledger-edge-eu1-d`. No other region, service, or shard is part of this
manifest. The canary consists of unit `ledger-edge-eu1-a`; the wave consists
of units `ledger-edge-eu1-b`, `ledger-edge-eu1-c`, and
`ledger-edge-eu1-d`.

REQUIRED PROMOTION GATES

| Gate | Required evidence | Accountable owner |
| --- | --- | --- |
| G-BUILD-01 | Source revision and builder attestation match the image digest. | Build Engineering |
| G-TEST-02 | Release validation suite passes for the pinned source revision. | Release Quality |
| G-SUPPLY-03 | Signed synthetic component inventory has no blocking finding. | Product Security |
| G-CANARY-04 | Canary availability and error rate remain inside the frozen bounds. | Runtime Operations |
| G-CAPACITY-05 | Canary latency remains below the release ceiling. | Runtime Operations |
| G-ROLLBACK-06 | Exact previous stable image and rollback bundle are retrievable. | Release Engineering |

ROLLBACK AUTHORITY

Every production target uses rollback artifact
`RB-SYN-REL-2026-10-03-04-PREV3`, pinned to the previous stable release
`REL-SYN-2026-10-03-03` and image digest
`sha256:2c88e74118de02583d908f90e9b94a46d31a29fa59e5f08c76d113f0b741ce62`.
The rollback bundle hash and retrieval state are present in the frozen
release-state snapshot. Do not substitute a similarly named artifact.

BOARD DISPOSITION

Promote only when source/build/image identity matches, all six required gates
pass, both canary measurements meet their stated limits, the requested cohort
matches the frozen membership, and the exact rollback artifact is verified.
If evidence is incomplete or a bounded gate is unknown, use `HOLD`. If an
identity mismatch or blocking security finding is established, use
`ROLLBACK`. The board records every required gate ID and the exact rollback
artifact ID in its final structured decision.
"#;

const SYSTEM: &str = "You are reviewing a wholly synthetic, read-only software-release promotion packet. At request slots 1 and 2, return exactly one compact JSON object with the single string field action_id and choose one displayed finite action. At request slot 3, return only the exact structured JSON fields requested, derived from the frozen manifest, release state, and receipts. Do not invent identities, gates, or deployment units.\n";
const TASK: &str = "CP09 synthetic release-board review. Decide whether the pinned ledger-edge-api release may be promoted to its exact production cohort. Source state is immutable and all checks are read-only. Apply the complete deployment-cohort manifest and frozen gate evidence.\n";
const REQUEST1: &str = "Perform the intake provenance check against the deployment-cohort manifest and frozen release state. Choose one finite read-only action: {\"action_id\":\"verify_build_provenance\"} or {\"action_id\":\"inspect_release_cohort_membership\"}.\n";
const REQUEST2: &str = "Check the required promotion gates, canary bounds, requested cohort, and rollback identity against the same frozen release state. Choose one finite read-only action: {\"action_id\":\"verify_required_rollout_gates\"} or {\"action_id\":\"verify_rollback_artifact\"}.\n";
const REQUEST3: &str = "Return only exact JSON with fields: decision (PROMOTE, HOLD, or ROLLBACK); release_id; build_digest; image_digest; governing_gate_ids (all required IDs in manifest order); environment_cohort (environment, cohort_id, deployment_unit_ids); required_rollback_artifact_id; reason_code. Do not add prose or fields.\n";

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn json_bytes(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).expect("fixed CP09 JSON serializes");
    bytes.push(b'\n');
    bytes
}

fn provenance(locator: &str, revision: &str, bytes: &[u8]) -> Value {
    json!({"source_kind":"self_authored","classification":"CAPTURED_EXPLICIT","source_locator":locator,"source_revision":revision,"content_hash":digest(bytes)})
}

fn asset(
    id: &str,
    path: &str,
    kind: &str,
    event: Option<&str>,
    revision: Option<&str>,
    bytes: &[u8],
) -> Value {
    let mut value = json!({"asset_id":id,"relative_path":path,"sha256":digest(bytes),"kind":kind});
    if let Some(event_id) = event {
        value["event_id"] = json!(event_id);
    }
    if let Some(revision_id) = revision {
        value["revision_id"] = json!(revision_id);
    }
    value
}

// The explicit fields keep each synthetic event's provenance visible at call sites.
#[allow(clippy::too_many_arguments)]
fn message_event(
    id: &str,
    seq: u32,
    context_id: &str,
    body_hash: &str,
    source_rev: &str,
    refs: &[&str],
    locator: &str,
    source_hash: &str,
    workflow_hash: &str,
    recipe: Option<&[u8]>,
) -> Value {
    let mut prov = vec![
        json!({"source_kind":"self_authored","classification":"CAPTURED_EXPLICIT","source_locator":locator,"source_revision":source_rev,"content_hash":source_hash}),
        json!({"source_kind":"self_authored","classification":"CAPTURED_EXPLICIT","source_locator":"cp09/workflow.md#ordinary-change-board-packet-provenance","source_revision":WORKFLOW_REV,"content_hash":workflow_hash}),
    ];
    if let Some(recipe_bytes) = recipe {
        prov.push(provenance(
            "source/change-board-packet-recipe-v1.json#manifest-occurrence",
            "synthetic-change-board-packet@1.0",
            recipe_bytes,
        ));
    }
    json!({"event_id":id,"sequence_index":seq,"event_type":"message","actor_role":"user","parent_event_ids":[],"reference_event_ids":refs,"action":null,"result":null,"context_block_id":context_id,"world_state_revision":WORLD,"order":{"logical_tick":seq,"timestamp_origin":"derived_structural"},"content_hash":body_hash,"provenance":prov})
}

fn action_event(
    id: &str,
    seq: u32,
    action_id: &str,
    refs: &[&str],
    workflow_hash: &str,
    state_hash: &str,
    manifest_hash: &str,
) -> Value {
    let args = json!({"release_id":"REL-SYN-2026-10-03-04","manifest_sha256":manifest_hash,"release_state_sha256":state_hash,"read_only":true});
    json!({"event_id":id,"sequence_index":seq,"event_type":"action","actor_role":"agent","parent_event_ids":[],"reference_event_ids":refs,"action":{"action_id":action_id,"tool_name":action_id,"argument_hash":digest(&serde_json::to_vec(&args).unwrap())},"result":null,"context_block_id":null,"world_state_revision":WORLD,"order":{"logical_tick":seq,"timestamp_origin":"derived_structural"},"content_hash":null,"provenance":[{"source_kind":"self_authored","classification":"CAPTURED_EXPLICIT","source_locator":"cp09/workflow.md#finite-read-only-action-menu","source_revision":WORKFLOW_REV,"content_hash":workflow_hash}]})
}

#[allow(clippy::too_many_arguments)]
fn result_event(
    id: &str,
    seq: u32,
    result_id: &str,
    action_id: &str,
    receipt_hash: &str,
    refs: &[&str],
    path: &str,
    revision: &str,
) -> Value {
    json!({"event_id":id,"sequence_index":seq,"event_type":"result","actor_role":"tool","parent_event_ids":[],"reference_event_ids":refs,"action":null,"result":{"result_id":result_id,"originating_action_id":action_id,"observation_hash":receipt_hash,"status":"success"},"context_block_id":null,"world_state_revision":WORLD,"order":{"logical_tick":seq,"timestamp_origin":"derived_structural"},"content_hash":null,"provenance":[{"source_kind":"self_authored","classification":"CAPTURED_EXPLICIT","source_locator":path,"source_revision":revision,"content_hash":receipt_hash}]})
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
    json!({"relation_id":id,"relation_type":kind,"from_id":from,"to_id":to,"scope":"scenario_local","semantics_version":"controlled-benchmark-relations-v1","provenance":[{"source_kind":"self_authored","classification":"CAPTURED_EXPLICIT","source_locator":locator,"source_revision":revision,"content_hash":hash}]})
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

fn generated_files() -> BTreeMap<String, Vec<u8>> {
    let workflow = WORKFLOW.as_bytes().to_vec();
    let governing = MANIFEST_SPEC.as_bytes().to_vec();
    let image_digest = "sha256:8f7b136c48d7721be30ca7d3f26fcb82578af930a15d4e296e7c113a2f1a4b80";
    let build_digest = "sha256:57b037a26c588ad19e8479d14fcb839669841ee416fd16e8a6c66aec6e004c1d";
    let rollback_digest = "sha256:2c88e74118de02583d908f90e9b94a46d31a29fa59e5f08c76d113f0b741ce62";
    let gate_ids = [
        "G-BUILD-01",
        "G-TEST-02",
        "G-SUPPLY-03",
        "G-CANARY-04",
        "G-CAPACITY-05",
        "G-ROLLBACK-06",
    ];
    let manifest = json_bytes(&json!({
        "schema_id":"prefixity.claim2.synthetic-deployment-cohort-manifest","schema_version":1,
        "synthetic_only":true,"record_state":"immutable_governing_record","manifest_id":"MAN-SYN-REL-2026-10-03-04","revision":"1.0","source_revision":MANIFEST_REV,"world_state_revision":WORLD,
        "release":{"release_id":"REL-SYN-2026-10-03-04","service":"ledger-edge-api","release_train":"2026.10","release_revision":4,"source_commit":"8a64e97d32f5c0146bc9d1087254ce219a76d331","source_tree_digest":"sha256:ac7c6ec3d3143b9af3273f014cb9308fe7c82c69a4b6f82d7c0ecde3b03a9c17","build_attestation_id":"att-SYN-BLD-2026-1003-044","build_digest":build_digest,"image_ref":"registry.synthetic.invalid/ledger-edge-api:2026.10.3-4","image_digest":image_digest},
        "completeness_contract":{"required_fields":["release_identity","source_build_image_digests","environment_targets","rollout_cohort_membership","required_gate_ids_and_owners","rollback_artifact_ids"],"cohort_membership_is_closed_world":true,"filler_entries_permitted":false},
        "environment_targets":[{"target_id":"staging-eu1","environment":"staging","purpose":"pre-promotion validation","required_before_board":true},{"target_id":"production-canary-eu1","environment":"production","cohort_id":"prod-canary-10pct","purpose":"30-minute canary observation","required_before_wave":true},{"target_id":"production-wave-eu1","environment":"production","cohort_id":"prod-wave-02","purpose":"requested production promotion","required_before_wave":true}],
        "rollout_cohorts":[{"cohort_id":"staging-validation-01","environment":"staging","target_id":"staging-eu1","deployment_unit_ids":["ledger-edge-eu1-a","ledger-edge-eu1-b","ledger-edge-eu1-c","ledger-edge-eu1-d"]},{"cohort_id":"prod-canary-10pct","environment":"production","target_id":"production-canary-eu1","traffic_share_basis_points":1000,"deployment_unit_ids":["ledger-edge-eu1-a"]},{"cohort_id":"prod-wave-02","environment":"production","target_id":"production-wave-eu1","deployment_unit_ids":["ledger-edge-eu1-b","ledger-edge-eu1-c","ledger-edge-eu1-d"]}],
        "bounded_deployment_units":[{"deployment_unit_id":"ledger-edge-eu1-a","region":"EU1","shard":"a","service":"ledger-edge-api","image_digest":image_digest},{"deployment_unit_id":"ledger-edge-eu1-b","region":"EU1","shard":"b","service":"ledger-edge-api","image_digest":image_digest},{"deployment_unit_id":"ledger-edge-eu1-c","region":"EU1","shard":"c","service":"ledger-edge-api","image_digest":image_digest},{"deployment_unit_id":"ledger-edge-eu1-d","region":"EU1","shard":"d","service":"ledger-edge-api","image_digest":image_digest}],
        "required_gates":[{"gate_id":gate_ids[0],"evidence":"source revision and builder attestation bind to image digest","owner":"Build Engineering"},{"gate_id":gate_ids[1],"evidence":"release validation suite passes","owner":"Release Quality"},{"gate_id":gate_ids[2],"evidence":"signed synthetic component inventory has no blocking finding","owner":"Product Security"},{"gate_id":gate_ids[3],"evidence":"canary availability and error rate meet frozen limits","owner":"Runtime Operations"},{"gate_id":gate_ids[4],"evidence":"canary p95 latency stays below release ceiling","owner":"Runtime Operations"},{"gate_id":gate_ids[5],"evidence":"exact previous stable image and rollback bundle are retrievable","owner":"Release Engineering"}],
        "canary_limits":{"observation_minutes":30,"availability_minimum_basis_points":9990,"error_rate_maximum_basis_points":50,"p95_latency_maximum_milliseconds":250},
        "rollback_by_production_target":[{"target_id":"production-canary-eu1","rollback_artifact_id":"RB-SYN-REL-2026-10-03-04-PREV3","previous_release_id":"REL-SYN-2026-10-03-03","image_digest":rollback_digest},{"target_id":"production-wave-eu1","rollback_artifact_id":"RB-SYN-REL-2026-10-03-04-PREV3","previous_release_id":"REL-SYN-2026-10-03-03","image_digest":rollback_digest}],
        "board_rule":{"promote_requires_all_gates_pass":true,"unknown_or_missing_evidence":"HOLD","identity_mismatch_or_blocking_security":"ROLLBACK","requested_target_id":"production-wave-eu1"}
    }));
    let manifest_hash = digest(&manifest);
    let state = json_bytes(&json!({
        "schema_id":"prefixity.claim2.synthetic-frozen-release-state","schema_version":1,
        "synthetic_only":true,"record_state":"frozen_read_only","source_revision":STATE_REV,"world_state_revision":WORLD,"release_id":"REL-SYN-2026-10-03-04","manifest_id":"MAN-SYN-REL-2026-10-03-04","manifest_revision":MANIFEST_REV,"manifest_sha256":manifest_hash,
        "source":{"repository":"synthetic://ledger-edge-api","commit":"8a64e97d32f5c0146bc9d1087254ce219a76d331","source_tree_sha256":"sha256:ac7c6ec3d3143b9af3273f014cb9308fe7c82c69a4b6f82d7c0ecde3b03a9c17"},
        "build":{"attestation_id":"att-SYN-BLD-2026-1003-044","builder":"synthetic-builder-5.4","build_digest":build_digest,"subject_image_digest":image_digest,"source_commit":"8a64e97d32f5c0146bc9d1087254ce219a76d331","signature_status":"VALID_SYNTHETIC_ATTESTATION"},
        "staging_validation":{"target_id":"staging-eu1","cohort_id":"staging-validation-01","deployment_unit_count":4,"image_digest":image_digest,"status":"PASS"},
        "gate_results":[{"gate_id":"G-BUILD-01","status":"PASS","evidence_id":"EV-SYN-BUILD-044"},{"gate_id":"G-TEST-02","status":"PASS","evidence_id":"EV-SYN-TEST-044","passed":128,"failed":0,"skipped":2},{"gate_id":"G-SUPPLY-03","status":"PASS","evidence_id":"EV-SYN-SUPPLY-044","blocking_findings":0,"inventory_signature":"VALID_SYNTHETIC_SIGNATURE"},{"gate_id":"G-CANARY-04","status":"PASS","evidence_id":"EV-SYN-CANARY-044","availability_basis_points":9997,"error_rate_basis_points":12,"observation_minutes":30},{"gate_id":"G-CAPACITY-05","status":"PASS","evidence_id":"EV-SYN-CANARY-044","p95_latency_milliseconds":183,"limit_milliseconds":250},{"gate_id":"G-ROLLBACK-06","status":"PASS","evidence_id":"EV-SYN-ROLLBACK-044","rollback_artifact_id":"RB-SYN-REL-2026-10-03-04-PREV3","rollback_bundle_sha256":"sha256:9e0ab5147c9128d188cc3403d4a2b6a77bf9b1f7d8530c488d5f57b78b4668f0","artifact_state":"RETRIEVABLE_READ_ONLY"}],
        "canary_observation":{"target_id":"production-canary-eu1","cohort_id":"prod-canary-10pct","traffic_share_basis_points":1000,"deployment_unit_ids":["ledger-edge-eu1-a"],"image_digest":image_digest,"availability_basis_points":9997,"error_rate_basis_points":12,"p95_latency_milliseconds":183,"observation_minutes":30},
        "requested_cohort":{"target_id":"production-wave-eu1","environment":"production","cohort_id":"prod-wave-02","deployment_unit_ids":["ledger-edge-eu1-b","ledger-edge-eu1-c","ledger-edge-eu1-d"]},
        "rollback_artifact":{"artifact_id":"RB-SYN-REL-2026-10-03-04-PREV3","previous_release_id":"REL-SYN-2026-10-03-03","image_digest":rollback_digest,"bundle_sha256":"sha256:9e0ab5147c9128d188cc3403d4a2b6a77bf9b1f7d8530c488d5f57b78b4668f0","retrieval_status":"RETRIEVABLE_READ_ONLY"},
        "state_integrity":{"mutations_during_review":0,"source_state_frozen":true,"production_contacted":false}
    }));
    let state_hash = digest(&state);
    let recipe = json_bytes(
        &json!({"schema_id":"prefixity.claim2.synthetic-change-board-packet-recipe","schema_version":1,"template_id":"release-promotion-change-board","template_version":"1.0","ordinary_trigger":"production_promotion_review_opened","required_packet_material":["frozen_release_state:e-release-state","provenance_check_receipt:e-build-provenance-result","rollout_gate_receipt:e-rollout-gates-result","complete_governing_manifest"],"state_behavior":"carry_forward_existing_frozen_source_and_unique_receipts_without_copy","manifest_resolution":{"manifest_id":"MAN-SYN-REL-2026-10-03-04","revision":"1.0","source_revision":MANIFEST_REV,"body_sha256":manifest_hash},"attachment_behavior":"emit_one_native_complete_manifest_message_for_every_release_promotion_board_packet","read_only":true,"benchmark_condition_used":false}),
    );
    let receipt_build_path = "bodies/receipt-build-provenance.txt";
    let receipt_gates_path = "bodies/receipt-rollout-gates.txt";
    let receipt_cohort_path = "bodies/receipt-cohort-inspection.txt";
    let receipt_rollback_path = "bodies/receipt-rollback-verification.txt";
    let build_receipt = format!("CP09 provenance receipt PRV-SYN-044-01: release_id=REL-SYN-2026-10-03-04; manifest_sha256={manifest_hash}; release_state_sha256={state_hash}; source_commit=8a64e97d32f5c0146bc9d1087254ce219a76d331; build_attestation_id=att-SYN-BLD-2026-1003-044; build_digest={build_digest}; image_digest={image_digest}; signature_status=VALID_SYNTHETIC_ATTESTATION; staging_image_match=true; source_state=frozen_read_only.\n");
    let cohort_receipt = format!("CP09 cohort receipt CHT-SYN-044-01: manifest_sha256={manifest_hash}; release_state_sha256={state_hash}; staging_target=staging-eu1; staging_units=4; requested_target=production-wave-eu1; cohort_id=prod-wave-02; requested_units=ledger-edge-eu1-b,ledger-edge-eu1-c,ledger-edge-eu1-d; cohort_matches_manifest=true; source_state=frozen_read_only.\n");
    let gate_receipt = format!("CP09 gate receipt GATE-SYN-044-01: manifest_sha256={manifest_hash}; release_state_sha256={state_hash}; G-BUILD-01=PASS; G-TEST-02=PASS; G-SUPPLY-03=PASS; G-CANARY-04=PASS; G-CAPACITY-05=PASS; G-ROLLBACK-06=PASS; canary_traffic_share_basis_points=1000; canary_availability_basis_points=9997; canary_error_rate_basis_points=12; canary_p95_latency_milliseconds=183; observation_minutes=30; required_gate_count=6; failed_gate_count=0; source_state=frozen_read_only.\n");
    let rollback_receipt = format!("CP09 rollback receipt RB-SYN-044-01: release_id=REL-SYN-2026-10-03-04; artifact_id=RB-SYN-REL-2026-10-03-04-PREV3; previous_release_id=REL-SYN-2026-10-03-03; image_digest={rollback_digest}; bundle_sha256=sha256:9e0ab5147c9128d188cc3403d4a2b6a77bf9b1f7d8530c488d5f57b78b4668f0; retrieval_status=RETRIEVABLE_READ_ONLY; target_count=2.\n");
    let receipt_bodies = [
        build_receipt.into_bytes(),
        cohort_receipt.into_bytes(),
        gate_receipt.into_bytes(),
        rollback_receipt.into_bytes(),
    ];
    let receipt_hashes = receipt_bodies.iter().map(|b| digest(b)).collect::<Vec<_>>();
    let workflow_hash = digest(&workflow);
    let event_ids = [
        "e-build-provenance-result",
        "e-cohort-inspection-result",
        "e-rollout-gates-result",
        "e-rollback-verification-result",
    ];
    let result_ids = [
        "r-build-provenance-044",
        "r-cohort-inspection-044",
        "r-rollout-gates-044",
        "r-rollback-verification-044",
    ];
    let actions = [
        "verify_build_provenance",
        "inspect_release_cohort_membership",
        "verify_required_rollout_gates",
        "verify_rollback_artifact",
    ];
    let action_events = [
        "e-build-provenance-action",
        "e-cohort-inspection-action",
        "e-rollout-gates-action",
        "e-rollback-verification-action",
    ];
    let result_revisions = [
        "release-provenance-REL-SYN-044-v1",
        "cohort-inspection-REL-SYN-044-v1",
        "rollout-gates-REL-SYN-044-v1",
        "rollback-verification-REL-SYN-044-v1",
    ];
    let result_paths = [
        receipt_build_path,
        receipt_cohort_path,
        receipt_gates_path,
        receipt_rollback_path,
    ];

    let mut files = BTreeMap::new();
    for (path, bytes) in [
        ("workflow.md", workflow.clone()),
        (
            "source/deployment-cohort-manifest-v1.json",
            manifest.clone(),
        ),
        (
            "source/release-promotion-manifest-governing-scope-v1.md",
            governing.clone(),
        ),
        ("source/frozen-release-state-v1.json", state.clone()),
        ("source/change-board-packet-recipe-v1.json", recipe.clone()),
        (
            "bodies/deployment-cohort-manifest-original.json",
            manifest.clone(),
        ),
        (
            "bodies/deployment-cohort-manifest-final-packet.json",
            manifest.clone(),
        ),
        ("bodies/frozen-release-state-v1.json", state.clone()),
        ("prompt/system.txt", SYSTEM.as_bytes().to_vec()),
        ("prompt/task.txt", TASK.as_bytes().to_vec()),
        ("prompt/request1.txt", REQUEST1.as_bytes().to_vec()),
        ("prompt/request2.txt", REQUEST2.as_bytes().to_vec()),
        ("prompt/request3.txt", REQUEST3.as_bytes().to_vec()),
    ] {
        files.insert(path.to_string(), bytes);
    }
    for (path, body) in result_paths.iter().zip(receipt_bodies.iter()) {
        files.insert((*path).to_string(), body.clone());
    }

    let manifest_hash = digest(&manifest);
    let original = message_event(
        "e-release-manifest-original",
        0,
        "ctx-release-manifest-original",
        &manifest_hash,
        MANIFEST_REV,
        &[],
        "source/deployment-cohort-manifest-v1.json#complete-deployment-cohort",
        &manifest_hash,
        &workflow_hash,
        None,
    );
    let mut state_event = message_event(
        "e-release-state",
        1,
        "ctx-frozen-release-state",
        &state_hash,
        STATE_REV,
        &["e-release-manifest-original"],
        "source/frozen-release-state-v1.json#frozen-state",
        &state_hash,
        &workflow_hash,
        None,
    );
    state_event["provenance"]
        .as_array_mut()
        .unwrap()
        .push(provenance(
            "source/deployment-cohort-manifest-v1.json#release-identity",
            MANIFEST_REV,
            &manifest,
        ));
    let mut events = vec![original, state_event];
    let action_seq = [2, 4, 6, 8];
    let result_seq = [3, 5, 7, 9];
    let action_refs: [&[&str]; 4] = [
        &["e-release-manifest-original", "e-release-state"],
        &["e-release-manifest-original", "e-release-state"],
        &["e-release-state", "e-build-provenance-result"],
        &["e-release-state", "e-build-provenance-result"],
    ];
    let result_refs: [&[&str]; 4] = [
        &[
            "e-build-provenance-action",
            "e-release-manifest-original",
            "e-release-state",
        ],
        &[
            "e-cohort-inspection-action",
            "e-release-manifest-original",
            "e-release-state",
        ],
        &[
            "e-rollout-gates-action",
            "e-release-state",
            "e-build-provenance-result",
        ],
        &[
            "e-rollback-verification-action",
            "e-release-state",
            "e-build-provenance-result",
        ],
    ];
    for i in 0..4 {
        events.push(action_event(
            action_events[i],
            action_seq[i],
            actions[i],
            action_refs[i],
            &workflow_hash,
            &state_hash,
            &manifest_hash,
        ));
        events.push(result_event(
            event_ids[i],
            result_seq[i],
            result_ids[i],
            actions[i],
            &receipt_hashes[i],
            result_refs[i],
            result_paths[i],
            result_revisions[i],
        ));
    }
    let mut final_occurrence = message_event(
        "e-release-manifest-final-reattachment",
        10,
        "ctx-release-manifest-final-packet",
        &manifest_hash,
        MANIFEST_REV,
        &[
            "e-release-manifest-original",
            "e-release-state",
            "e-build-provenance-result",
            "e-rollout-gates-result",
        ],
        "source/deployment-cohort-manifest-v1.json#complete-deployment-cohort",
        &manifest_hash,
        &workflow_hash,
        Some(&recipe),
    );
    // Add the recipe as an explicit source provenance record alongside the ordinary packet trigger.
    final_occurrence["provenance"]
        .as_array_mut()
        .unwrap()
        .push(provenance(
            "source/change-board-packet-recipe-v1.json#ordinary-attachment-rule",
            "synthetic-change-board-packet@1.0",
            &recipe,
        ));
    events.push(final_occurrence);

    let relations = vec![
        relation(
            "produce-build-provenance",
            "produces",
            actions[0],
            result_ids[0],
            result_paths[0],
            result_revisions[0],
            &receipt_hashes[0],
        ),
        relation(
            "produce-cohort-inspection",
            "produces",
            actions[1],
            result_ids[1],
            result_paths[1],
            result_revisions[1],
            &receipt_hashes[1],
        ),
        relation(
            "produce-rollout-gates",
            "produces",
            actions[2],
            result_ids[2],
            result_paths[2],
            result_revisions[2],
            &receipt_hashes[2],
        ),
        relation(
            "produce-rollback-verification",
            "produces",
            actions[3],
            result_ids[3],
            result_paths[3],
            result_revisions[3],
            &receipt_hashes[3],
        ),
        relation(
            "same-state-manifest-reattachment",
            "same_state_revision",
            "e-release-manifest-original",
            "e-release-manifest-final-reattachment",
            "cp09/source/deployment-cohort-manifest-v1.json#complete-deployment-cohort",
            MANIFEST_REV,
            &manifest_hash,
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
            "release_manifest_original",
            "bodies/deployment-cohort-manifest-original.json",
            "context_attachment",
            Some("e-release-manifest-original"),
            Some(MANIFEST_REV),
            &manifest,
        ),
        asset(
            "release_state",
            "bodies/frozen-release-state-v1.json",
            "context_attachment",
            Some("e-release-state"),
            Some(STATE_REV),
            &state,
        ),
        asset(
            "release_manifest_final_packet",
            "bodies/deployment-cohort-manifest-final-packet.json",
            "context_attachment",
            Some("e-release-manifest-final-reattachment"),
            Some(MANIFEST_REV),
            &manifest,
        ),
    ];
    for i in 0..4 {
        assets.push(asset(
            [
                "receipt_build_provenance",
                "receipt_cohort_inspection",
                "receipt_rollout_gates",
                "receipt_rollback_verification",
            ][i],
            result_paths[i],
            "environment_receipt",
            Some(event_ids[i]),
            Some(result_revisions[i]),
            &receipt_bodies[i],
        ));
    }

    let slot1 = vec![
        message("system", vec![asset_part("system")]),
        message(
            "user",
            vec![
                asset_part("task"),
                asset_part("request1"),
                event_part("e-release-manifest-original"),
                event_part("e-release-state"),
            ],
        ),
    ];
    let mut slot2 = slot1.clone();
    slot2.push(message(
        "assistant",
        vec![json!({"kind":"prior_assistant_output","request_slot":1})],
    ));
    slot2.push(message(
        "user",
        vec![json!({"kind":"prior_environment_receipt","request_slot":1})],
    ));
    slot2.push(message("user", vec![asset_part("request2")]));
    let mut slot3 = slot2.clone();
    slot3.push(message(
        "assistant",
        vec![json!({"kind":"prior_assistant_output","request_slot":2})],
    ));
    slot3.push(message(
        "user",
        vec![json!({"kind":"prior_environment_receipt","request_slot":2})],
    ));
    slot3.push(message(
        "user",
        vec![
            asset_part("request3"),
            event_part("e-release-manifest-final-reattachment"),
        ],
    ));

    let expected_answer = json!({
        "decision":"PROMOTE","release_id":"REL-SYN-2026-10-03-04","build_digest":build_digest,"image_digest":image_digest,
        "governing_gate_ids":gate_ids,
        "environment_cohort":{"environment":"production","cohort_id":"prod-wave-02","deployment_unit_ids":["ledger-edge-eu1-b","ledger-edge-eu1-c","ledger-edge-eu1-d"]},
        "required_rollback_artifact_id":"RB-SYN-REL-2026-10-03-04-PREV3","reason_code":"ALL_REQUIRED_GATES_PASS_ROLLBACK_READY"
    });
    let evaluation_key = json!({
        "expected_action_ids":[actions[0], actions[2]],
        "expected_states_after_action":["release_build_provenance_verified", "rollout_gates_and_rollback_verified"],
        "expected_result_event_ids":[event_ids[0], event_ids[2]],
        "expected_final_answer":expected_answer,
        "required_event_ids":["e-release-manifest-original","e-release-state","e-build-provenance-result","e-rollout-gates-result"],
        "required_relation_ids":["produce-build-provenance","produce-rollout-gates"],
        "critical_event_ids":["e-release-manifest-original","e-release-state","e-build-provenance-result","e-rollout-gates-result"]
    });
    let key_bytes = json_bytes(&evaluation_key);
    let advancing_points = [
        (1, actions[0], event_ids[0], result_ids[0], result_paths[0], 0, "release_build_provenance_verified"),
        (2, actions[2], event_ids[2], result_ids[2], result_paths[2], 2, "rollout_gates_and_rollback_verified"),
    ].into_iter().map(|(slot, action, event, receipt, path, i, state_after)| {
        let raw = format!("{{\"action_id\":\"{action}\"}}");
        json!({"request_slot":slot,"expected_action_id":action,"canonical_raw_utf8":raw,"canonical_raw_sha256":digest(raw.as_bytes()),"raw_language_cardinality":1,"receipt_event_id":event,"receipt_id":receipt,"receipt_path":path,"receipt_sha256":receipt_hashes[i],"receipt_revision":result_revisions[i],"state_after":state_after})
    }).collect::<Vec<_>>();
    let advancing_domain = json_bytes(
        &json!({"schema_id":"prefixity.phase1c.claim2-advancing-output-domain-v3-case","schema_version":3,"case_id":"CP09","protocol_id":"prefixity.phase1c.claim2-canonical-advancing-output.v1","offline_only":true,"points":advancing_points}),
    );
    let action_menu = vec![
        json!({"action_slot":1,"action_id":actions[0],"action_event_id":action_events[0],"result_event_id":event_ids[0],"result_asset_id":"receipt_build_provenance","state_after":"release_build_provenance_verified"}),
        json!({"action_slot":1,"action_id":actions[1],"action_event_id":action_events[1],"result_event_id":event_ids[1],"result_asset_id":"receipt_cohort_inspection","state_after":"release_cohort_membership_inspected"}),
        json!({"action_slot":2,"action_id":actions[2],"action_event_id":action_events[2],"result_event_id":event_ids[2],"result_asset_id":"receipt_rollout_gates","state_after":"rollout_gates_and_rollback_verified"}),
        json!({"action_slot":2,"action_id":actions[3],"action_event_id":action_events[3],"result_event_id":event_ids[3],"result_asset_id":"receipt_rollback_verification","state_after":"rollback_artifact_verified"}),
    ];
    let public_provenance = vec![
        provenance("cp09/workflow.md", WORKFLOW_REV, &workflow),
        provenance(
            "cp09/source/deployment-cohort-manifest-v1.json",
            MANIFEST_REV,
            &manifest,
        ),
        provenance(
            "cp09/source/release-promotion-manifest-governing-scope-v1.md",
            "deployment-cohort-completeness@1.0",
            &governing,
        ),
        provenance(
            "cp09/source/frozen-release-state-v1.json",
            STATE_REV,
            &state,
        ),
        provenance(
            "cp09/source/change-board-packet-recipe-v1.json",
            "synthetic-change-board-packet@1.0",
            &recipe,
        ),
    ];
    let case = json!({"schema_id":"prefixity.phase1c.claim2-workload-case","schema_version":1,"case_id":"CP09","kind":"positive","planner_input":{"events":events,"relations":relations,"provenance":public_provenance},"assets":assets,"action_menu":action_menu,"request_templates":[{"request_slot":1,"messages":slot1},{"request_slot":2,"messages":slot2},{"request_slot":3,"messages":slot3}],"evaluation_key_path":"evaluation/key.json","evaluation_key_sha256":digest(&key_bytes),"assistant_output_planning_bytes":4096,"token_proof_inputs":null});
    files.insert("evaluation/key.json".to_string(), key_bytes);
    files.insert("case.json".to_string(), json_bytes(&case));
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
