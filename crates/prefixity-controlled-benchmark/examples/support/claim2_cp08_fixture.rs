use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;

const WORKFLOW: &str = r#"# CP08 synthetic cold-chain lot disposition workflow, version 1

This fixture is wholly synthetic. `LOT-SYN-CC08-117`, `PRD-SYN-RX-44`,
`LOGGER-SYN-14`, and every supplier, site, certificate, and operator reference
are fictional. The deterministic builder creates a frozen receiving record,
logger intervals, calibration record, ordinary quality-review packet recipe,
receipts, trace, and isolated evaluation key. It performs no network, provider,
or model operation.

## Ordinary receiving and disposition sequence

At receipt of a temperature-controlled lot, the receiving technician opens
the currently effective cold-chain SOP beside the lot record, logger export,
and calibration certificate. The technician checks the lot identity and
continuous logger record, then verifies calibration and calculates total
excursion exposure from the unchanged records. Those checks are read-only and
do not edit, repair, or reclassify any source record.

When a disposition review is opened, the routine controlled-record packet
contains the frozen lot, logger, and calibration evidence plus a copy of the
effective SOP. Quality reviewers receive the governing procedure with the
decision record so that each hold, release, or escalation is traceable to the
rules in force. The packet recipe attaches one native SOP message for every
review; it does not inspect document size or use a benchmark condition. The
final SOP occurrence is generated from the same content-addressed procedure
revision as the receiving occurrence.

## Frozen source state

The SOP is `CC-SOP-04`, revision `3.2.1`, effective 2026-01-01. The frozen lot
record is `source/lot-snapshot-LOT-SYN-CC08-117-v1.json`, revision
`cold-chain-lot@LOT-SYN-CC08-117-v1`. It names the logger and calibration
record and freezes the receiving interval. The logger export is
`source/logger-record-LOGGER-SYN-14-v1.json`, revision
`cold-chain-logger@LOGGER-SYN-14-v1`; the certificate is
`source/calibration-CAL-SYN-14-2026-v1.json`, revision
`logger-calibration@CAL-SYN-14-2026-v1`. All three remain read-only under
world-state revision `cold-chain-disposition@LOT-SYN-CC08-117-v1`.

The original procedure occurrence is `e-sop-original`; the final packet
occurrence is `e-sop-final-reattachment`. Their UTF-8 body bytes, SOP source
revision, and world-state revision are identical. The original remains in the
chronological input. The later native user Message has no consumers and no
protected relation. The ordinary packet recipe records the unconditional
attachment rule and the procedure hash.

## Deterministic checks

Request 1 offers `inspect_lot_logger_state` and
`verify_receiving_record_completeness`. The first checks lot identity, the
logger identity, interval coverage, observed extrema, and the frozen source
hashes. Request 2 offers `calculate_calibration_and_exposure` and
`verify_calibration_certificate`. The expected calculation checks the
calibration correction, inclusive storage band, excursion interval duration,
and cumulative temperature-degree-minutes using the published SOP method.
Each action creates a distinct deterministic receipt tied to its action and
source revision; no receipt mutates the lot, logger, certificate, or SOP.

The final answer is exact JSON: disposition, rule IDs, corrected interval
extrema, interval and cumulative excursion arithmetic, calibration validity,
and the required hold or escalation reason. Its evaluation key is stored
separately and is never a prompt asset or planner-visible event. The public
source documents and check receipts define the answer independently.

## Provenance limits

The records describe a closed synthetic quality system, not a real facility,
product, shipment, logger, certificate, or release decision. The repeated SOP
is part of the ordinary final quality packet defined above. No size-derived
criterion or tokenizer result was used to compose, select, or repeat it.
"#;

const SOP: &str = r#"SYNTHETIC QUALITY SYSTEM
CONTROLLED COLD-CHAIN RECEIPT, EXCURSION REVIEW, AND LOT DISPOSITION

Document: CC-SOP-04
Revision: 3.2.1
Effective date: 2026-01-01
Supersedes: CC-SOP-04 revision 3.2.0
Owner: Synthetic Quality Operations
Approval record: QA-SYN-APPROVAL-2025-118
Status: current controlled procedure

1. PURPOSE AND SCOPE

This procedure defines the receiving record, temperature evidence review,
calibration check, cumulative excursion calculation, disposition, escalation,
and retention requirements for synthetic demonstration lots designated for
refrigerated storage. It applies from receipt at the controlled dock through
the signed quality disposition. It covers the complete rules needed for the
lot in this fixture. No external product specification or unwritten
temperature rule is required. It does not authorize a technician to alter a
logger export, change a certificate, waive an excursion, or release a lot
outside the decision table in section 8.

2. TERMS AND CONTROLLED RECORDS

2.1 `lot record` means the immutable receipt row containing the lot ID,
product revision, supplier seal, shipment identity, receipt time, storage
unit, assigned logger, required range, and references to the logger export and
calibration certificate. `Source revision` identifies those captured bytes.

2.2 `logger interval` means a consecutive start/end interval represented by
one export row. The row includes its minimum and maximum indicated
temperatures for the whole interval. Rows must abut: the next start must equal
the prior end. The required interval is 15 minutes. The period from first
start through final end is the receipt exposure period.

2.3 `calibration correction` is the signed value `reference temperature minus
indicated temperature` in the as-found certificate. Apply the certificate's
single approved correction to each reported logger minimum and maximum by
addition. Store corrected values in tenths of a degree Celsius; do not round
or convert through Fahrenheit.

2.4 `excursion interval` means a logger interval whose corrected minimum is
below 2.0 C or corrected maximum is above 8.0 C. The band is inclusive at
2.0 C and 8.0 C. A row inside the band is compliant. A row outside the band
is counted once even if both extrema are outside.

2.5 `excursion minutes` is the sum of full interval durations for excursion
rows. `Degree-minutes` is calculated separately for each row using its most
extreme corrected value: for a high row, `(corrected maximum - 8.0 C) x
interval minutes`; for a low row, `(2.0 C - corrected minimum) x interval
minutes`. Convert the temperature difference to tenths before multiplying,
so the retained integer unit is one tenth of a C-minute. Add the row values
to obtain cumulative degree-minutes. Do not cancel high and low exposure.

2.6 `initial hold` means physical segregation and an electronic HOLD status
until the disposition record is signed. It is distinct from an escalation to
the quality lead. A technician never releases a lot while a required record
or calculation is missing.

3. ROLES AND SEQUENCE

3.1 Receiving records the lot identity, intact seal, receipt interval, and
assigned logger. The technician compares those fields with the shipping
record and does not infer missing times.

3.2 The receiving technician checks the logger export against the lot record
and confirms each interval is present and consecutive. If an interval is
missing, duplicated, reversed, or longer than 15 minutes, set the lot status
to HOLD and send the evidence to the quality lead under rule CC-09.

3.3 Before computing corrected temperatures, the technician verifies that
the identified logger's calibration certificate was issued before the first
receipt interval and remains within its validity dates. Calibration evidence
is read as-found; a later adjustment cannot replace a failed as-found result.

3.4 A disposition request uses the ordinary controlled-record packet
template. It carries the frozen lot record and check receipts and attaches
one complete copy of the effective procedure so a reviewer can resolve each
rule reference against the signed disposition. The attachment is required
for every lot review, regardless of whether a temperature excursion exists.

4. REQUIRED SOURCE IDENTITY

4.1 The lot ID, product ID, logger serial, calibration ID, logger export
hash, and SOP document revision must agree across their source records. A
hash mismatch, unknown alias, or inconsistent revision is an identity
failure; do not choose whichever record appears newest.

4.2 Source files are read-only during review. Corrections to source records
require a separately controlled amendment and a new source revision. The
disposition must state the hashes and revisions it evaluated.

4.3 Each generated calculation receipt identifies the action, input record
hashes, world-state revision, interval count, and computed values. A receipt
records the check; it cannot revise its source or substitute for a required
source document.

5. CALIBRATION VALIDITY — RULE CC-04

5.1 Use the logger identified by the frozen lot record. The certificate must
name that exact serial number, include an as-found reference and indicated
value, precede the first logger interval, and remain current on the final
interval date.

5.2 Calculate the signed correction as reference minus indicated reading.
The as-found absolute error is the absolute value of that correction. A
certificate is valid only if the certificate is in date and the absolute
error is at most 0.5 C (5 tenths). The certificate must state that no repair
or adjustment occurred before the as-found reading was recorded.

5.3 For a valid single-point certificate, apply its signed correction to the
minimum and maximum values in every interval in the identified logger export.
Preserve both indicated and corrected extrema in the calculation. If the
logger identity differs, the as-found value exceeds tolerance, or the
certificate is absent, future-dated, expired, or post-adjustment only, mark
calibration invalid and escalate under CC-09. Do not treat a missing
correction as zero.

6. LOGGER REVIEW AND STORAGE BAND — RULE CC-05

6.1 The approved refrigerated storage range for this procedure is 2.0 C
through 8.0 C inclusive. Apply the correction in section 5 before testing the
range. A corrected minimum below 2.0 C or corrected maximum above 8.0 C is
outside the band.

6.2 Verify every row between the frozen first and last timestamps. Rows must
be ordered, non-overlapping, contiguous, and exactly 15 minutes each. The
start of the first row and end of the last row must match the lot's recorded
receipt interval. Missing duration cannot be treated as compliant exposure.

6.3 Report the number of total rows, compliant rows, high rows, low rows,
first and last excursion timestamps, and the corrected extrema for every
row. If a single row has both high and low extrema, count one excursion
interval and calculate both high and low degree-minute terms for that row.

7. CUMULATIVE EXPOSURE — RULE CC-06

7.1 Sum the full row duration of each excursion interval. This deliberately
uses the interval extrema and does not estimate an average or interpolate
between readings. Retain separate high minutes, low minutes, and total
excursion minutes.

7.2 For each high interval, multiply the number of tenths of a degree above
80 tenths by the full interval minutes. For each low interval, multiply the
number of tenths below 20 tenths by the full interval minutes. Sum without
rounding and report in tenths of C-minute. These values are reproducible
integer arithmetic from the logger rows and the signed correction.

7.3 An interval at exactly 20 or 80 tenths is inside the permitted band and
has zero excursion minutes and zero degree-minutes. A corrected maximum of
85 tenths therefore contributes 5 tenths x 15 minutes = 75 tenths of
C-minute for one 15-minute row.

8. DISPOSITION RULES

8.1 RELEASE — rule CC-07. Release is permitted only when identity checks
pass, the certificate is valid, logger coverage is complete, every corrected
interval is inside the inclusive band, and both cumulative excursion
measures are zero. Record `RELEASE` and the evaluated revision set.

8.2 HOLD — rule CC-08. If identity, calibration, logger coverage, and
interval continuity pass, but one or more temperature excursion intervals
are present with total excursion time no greater than 30 minutes and
cumulative degree-minutes no greater than 150 tenths of C-minute, place the
lot on `HOLD`. State the first excursion, total minutes, cumulative
degree-minutes, and that quality review is required before use. These limits
are review boundaries, not automatic release limits. No operator may
convert a qualifying excursion to release from arithmetic alone.

8.3 ESCALATE — rule CC-09. Escalate to the quality lead and retain the lot on
HOLD if identity is inconsistent, calibration is invalid, logger coverage
or interval continuity is incomplete, total excursion time exceeds 30
minutes, cumulative degree-minutes exceed 150 tenths of C-minute, or either
exposure quantity cannot be calculated. Record every triggering reason.

8.4 Apply the rules in this order: source identity and calibration; logger
coverage and correction; excursion calculation; then disposition. An
escalation condition overrides the ordinary hold pathway. A valid, complete
record with a nonzero excursion within both review boundaries is HOLD. Only
zero exposure under fully valid evidence is RELEASE.

9. REVIEW RECORD AND AUDIT TRAIL

9.1 The final record identifies the lot and product revisions, logger and
calibration identities, SOP revision, source hashes, storage limits, every
corrected interval, high and low minutes, cumulative degree-minutes, and
disposition. The reason must identify each applicable CC rule.

9.2 Keep the original logger export, calibration certificate, signed
disposition, and final review packet together. Preserve earlier native
source occurrences and their order. A final packet contains the effective
procedure body as well as the lot evidence; it must not replace or relabel
the original source, a calculation receipt, an action, or an assistant
response.

9.3 Any change to source evidence after review creates a new revision and
requires a new review. A past receipt or disposition does not update the
current source state.

10. VERSION AND COMPLETENESS

10.1 Revision 3.2.1 is effective 2026-01-01 and is the complete controlled
rule set for the synthetic records that explicitly identify `CC-SOP-04`
revision 3.2.1. Earlier or later revisions do not modify this frozen review.

10.2 This document contains the full scope, source controls, role sequence,
calibration test, correction method, inclusive range, interval completeness
rule, exposure arithmetic, disposition limits, escalation path, and record
retention rules required for the defined cold-chain lot decision. No
external specification or unstated quality judgment is needed to compute the
structured result.
"#;

const SYSTEM: &str = "You are reviewing a deterministic synthetic cold-chain lot. At request slots 1 and 2, return exactly one JSON object with the single string field action_id and choose from the displayed finite read-only menu. At request slot 3, return exact JSON with the requested fields using the unchanged source records and the applicable SOP rules. Do not invent records, identifiers, calculations, or disposition conditions.\n";
const TASK: &str = "CP08 synthetic cold-chain lot disposition. Review the frozen receiving record, logger export, calibration evidence, and versioned quality SOP. All source evidence is read-only. Apply only the specified procedure revision.\n";
const REQUEST1: &str = "Inspect the lot and logger state under the SOP. Verify source identity, logger interval continuity, and the raw temperature extrema. Choose one read-only action: {\"action_id\":\"inspect_lot_logger_state\"} or {\"action_id\":\"verify_receiving_record_completeness\"}.\n";
const REQUEST2: &str = "Verify the calibration certificate and calculate corrected time-temperature and cumulative exposure from the same frozen logger record. Choose one read-only action: {\"action_id\":\"calculate_calibration_and_exposure\"} or {\"action_id\":\"verify_calibration_certificate\"}.\n";
const REQUEST3: &str = "Return exact JSON fields: disposition; governing_rule_ids; time_temperature_calculation (storage_range_celsius_tenths, interval_minutes, corrected_interval_extrema, compliant_interval_count, high_excursion_interval_count, low_excursion_interval_count); cumulative_exposure_calculation (high_minutes, low_minutes, total_excursion_minutes, high_degree_minutes_celsius_tenths, low_degree_minutes_celsius_tenths, total_degree_minutes_celsius_tenths); calibration (status, calibration_id, reference_celsius_tenths, indicated_celsius_tenths, correction_celsius_tenths, absolute_error_celsius_tenths, maximum_allowed_error_celsius_tenths, valid_through, applied_correction_celsius_tenths); required_hold_or_escalation_reason. Use integer tenths and exact source timestamps.\n";

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
        "source_kind": "self_authored",
        "classification": "CAPTURED_EXPLICIT",
        "source_locator": locator,
        "source_revision": revision,
        "content_hash": digest(bytes)
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
    let mut value =
        json!({"asset_id": id, "relative_path": path, "sha256": digest(body), "kind": kind});
    if let Some(event_id) = event {
        value["event_id"] = json!(event_id);
    }
    if let Some(revision_id) = revision {
        value["revision_id"] = json!(revision_id);
    }
    value
}

// Keep trace schema fields explicit at each fixture call site.
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
        "event_id": id, "sequence_index": seq, "event_type": "message", "actor_role": "user",
        "parent_event_ids": [], "reference_event_ids": refs, "action": null, "result": null,
        "context_block_id": context_id, "world_state_revision": world_revision,
        "order": {"logical_tick": seq, "timestamp_origin": "derived_structural"},
        "content_hash": body_hash,
        "provenance": [
            {"source_kind":"self_authored","classification":"CAPTURED_EXPLICIT","source_locator":locator,"source_revision":source_revision,"content_hash":source_hash},
            {"source_kind":"self_authored","classification":"CAPTURED_EXPLICIT","source_locator":"cp08/workflow.md#ordinary-final-packet-provenance","source_revision":"cp08-cold-chain-workflow-v1","content_hash":workflow_hash}
        ]
    })
}

// Keep action provenance and pinned source identities explicit in the fixture.
#[allow(clippy::too_many_arguments)]
fn action_event(
    id: &str,
    seq: u32,
    action_id: &str,
    refs: &[&str],
    world_revision: &str,
    workflow_hash: &str,
    lot_hash: &str,
    sop_hash: &str,
) -> Value {
    let args =
        json!({"lot_id":"LOT-SYN-CC08-117","lot_snapshot_sha256":lot_hash,"sop_sha256":sop_hash});
    json!({
        "event_id":id,"sequence_index":seq,"event_type":"action","actor_role":"agent",
        "parent_event_ids":[],"reference_event_ids":refs,
        "action":{"action_id":action_id,"tool_name":action_id,"argument_hash":digest(&serde_json::to_vec(&args).expect("action arguments serialize"))},
        "result":null,"context_block_id":null,"world_state_revision":world_revision,
        "order":{"logical_tick":seq,"timestamp_origin":"derived_structural"},"content_hash":null,
        "provenance":[{"source_kind":"self_authored","classification":"CAPTURED_EXPLICIT","source_locator":"cp08/workflow.md#read-only-action-menu","source_revision":"cp08-cold-chain-workflow-v1","content_hash":workflow_hash}]
    })
}

// Keep receipt identity, source revision, and DAG references explicit.
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

fn generated_files() -> BTreeMap<String, Vec<u8>> {
    let workflow = WORKFLOW.as_bytes().to_vec();
    let sop = SOP.as_bytes().to_vec();
    let sop_hash = digest(&sop);
    let sop_revision = "CC-SOP-04@3.2.1";
    let world_revision = "cold-chain-disposition@LOT-SYN-CC08-117-v1";

    let lot = json_bytes(&json!({
        "schema_id":"prefixity.claim2.synthetic-cold-chain-lot-snapshot","schema_version":1,
        "case_id":"LOT-SYN-CC08-117","synthetic_only":true,"snapshot_id":"cold-chain-source-CC08-117-v1",
        "world_state_revision":world_revision,"source_revision":"cold-chain-lot@LOT-SYN-CC08-117-v1",
        "lot_id":"LOT-SYN-CC08-117","product":{"synthetic_product_id":"PRD-SYN-RX-44","description":"temperature-controlled assay reagent kit","product_revision":"PRD-SYN-RX-44@6"},
        "shipment":{"synthetic_shipment_id":"SHP-SYN-CC08-117-A","seal_id":"SEAL-SYN-CC08-117","seal_condition":"intact","receipt_start":"2026-03-14T08:00:00Z","receipt_end":"2026-03-14T09:00:00Z","storage_unit_id":"REF-SYN-03"},
        "logger":{"logger_serial":"LOGGER-SYN-14","logger_source_revision":"cold-chain-logger@LOGGER-SYN-14-v1","logger_record_sha256":"PENDING","calibration_id":"CAL-SYN-14-2026","calibration_source_revision":"logger-calibration@CAL-SYN-14-2026-v1","calibration_record_sha256":"PENDING","required_interval_minutes":15},
        "procedure":{"document_id":"CC-SOP-04","revision":"3.2.1","source_revision":sop_revision,"body_sha256":sop_hash},
        "record_state":"frozen_read_only"
    }));
    let logger = json_bytes(&json!({
        "schema_id":"prefixity.claim2.synthetic-cold-chain-logger-record","schema_version":1,
        "record_id":"LOGGER-SYN-14-EXPORT-2026-03-14-A","logger_serial":"LOGGER-SYN-14",
        "source_revision":"cold-chain-logger@LOGGER-SYN-14-v1","world_state_revision":world_revision,"synthetic_only":true,
        "sampling_interval_minutes":15,"time_zone":"UTC","temperature_unit":"celsius",
        "intervals":[
            {"start":"2026-03-14T08:00:00Z","end":"2026-03-14T08:15:00Z","minimum_indicated_celsius_tenths":46,"maximum_indicated_celsius_tenths":51},
            {"start":"2026-03-14T08:15:00Z","end":"2026-03-14T08:30:00Z","minimum_indicated_celsius_tenths":77,"maximum_indicated_celsius_tenths":83},
            {"start":"2026-03-14T08:30:00Z","end":"2026-03-14T08:45:00Z","minimum_indicated_celsius_tenths":64,"maximum_indicated_celsius_tenths":72},
            {"start":"2026-03-14T08:45:00Z","end":"2026-03-14T09:00:00Z","minimum_indicated_celsius_tenths":53,"maximum_indicated_celsius_tenths":61}
        ],"export_state":"immutable_captured_record"
    }));
    let calibration = json_bytes(&json!({
        "schema_id":"prefixity.claim2.synthetic-logger-calibration-record","schema_version":1,
        "calibration_id":"CAL-SYN-14-2026","logger_serial":"LOGGER-SYN-14","synthetic_only":true,
        "source_revision":"logger-calibration@CAL-SYN-14-2026-v1","world_state_revision":world_revision,
        "issued_at":"2026-01-10T12:00:00Z","valid_through":"2027-01-10",
        "as_found":{"reference_celsius_tenths":50,"indicated_celsius_tenths":48,"signed_correction_celsius_tenths":2,"absolute_error_celsius_tenths":2,"maximum_allowed_error_celsius_tenths":5,"adjustment_before_as_found":false},
        "certificate_state":"immutable_captured_record"
    }));
    let logger_hash = digest(&logger);
    let calibration_hash = digest(&calibration);
    let mut lot_value: Value = serde_json::from_slice(&lot).unwrap();
    lot_value["logger"]["logger_record_sha256"] = json!(logger_hash);
    lot_value["logger"]["calibration_record_sha256"] = json!(calibration_hash);
    let lot = json_bytes(&lot_value);
    let lot_hash = digest(&lot);

    let recipe = json_bytes(&json!({
        "schema_id":"prefixity.claim2.synthetic-quality-review-packet-recipe","schema_version":1,
        "template_id":"cold-chain-final-quality-review","template_version":"1.0",
        "ordinary_trigger":"lot_disposition_review_opened",
        "required_packet_material":["frozen_lot_snapshot:e-lot-source","logger_export:e-logger-evidence","calibration_certificate:e-calibration-evidence","complete_effective_sop"],
        "lot_logger_calibration_behavior":"carry_forward_existing_native_occurrences_without_copy",
        "procedure_resolution":{"document_id":"CC-SOP-04","revision":"3.2.1","source_revision":sop_revision,"body_sha256":sop_hash},
        "attachment_behavior":"emit_one_native_sop_message_for_every_final_lot_review",
        "read_only":true,"purpose":"Keep the complete governing controlled procedure with the signed quality disposition.","benchmark_condition_used":false
    }));

    let lot_prov_hash = digest(&lot);
    let logger_prov_hash = digest(&logger);
    let calibration_prov_hash = digest(&calibration);

    let receipt_inspect = format!(
        "Lot/logger inspection CC08-INSPECT-01: lot=LOT-SYN-CC08-117; lot_snapshot_sha256={lot_hash}; logger=LOGGER-SYN-14; logger_record_sha256={logger_hash}; intervals=4; required_interval_minutes=15; first=2026-03-14T08:00:00Z; last=2026-03-14T09:00:00Z; rows_contiguous=true; source_state=frozen_read_only.\n"
    );
    let receipt_completeness = format!(
        "Receiving completeness check CC08-RECEIVE-01: lot=LOT-SYN-CC08-117; lot_snapshot_sha256={lot_hash}; product_revision=PRD-SYN-RX-44@6; shipment=SHP-SYN-CC08-117-A; seal_condition=intact; receipt_interval=2026-03-14T08:00:00Z/2026-03-14T09:00:00Z; logger_reference_present=true; calibration_reference_present=true; disposition_not_calculated.\n"
    );
    let receipt_calculation = format!(
        "Calibration/exposure check CC08-EXPOSURE-01: lot_snapshot_sha256={lot_hash}; logger_record_sha256={logger_hash}; calibration_record_sha256={calibration_hash}; certificate_valid=true; correction_celsius_tenths=2; corrected_intervals_celsius_tenths=48..53,79..85,66..74,55..63; compliant_intervals=3; high_excursion_intervals=1; low_excursion_intervals=0; high_minutes=15; low_minutes=0; total_excursion_minutes=15; cumulative_degree_minutes_celsius_tenths=75; source_state=frozen_read_only.\n"
    );
    let receipt_certificate = format!(
        "Calibration certificate verification CC08-CAL-01: logger=LOGGER-SYN-14; certificate=CAL-SYN-14-2026; certificate_sha256={calibration_hash}; reference_celsius_tenths=50; indicated_celsius_tenths=48; signed_correction_celsius_tenths=2; absolute_error_celsius_tenths=2; tolerance_celsius_tenths=5; adjustment_before_as_found=false; valid_through=2027-01-10; status=valid.\n"
    );
    let receipt_bodies = [
        receipt_inspect.into_bytes(),
        receipt_completeness.into_bytes(),
        receipt_calculation.into_bytes(),
        receipt_certificate.into_bytes(),
    ];
    let receipt_hashes = receipt_bodies
        .iter()
        .map(|body| digest(body))
        .collect::<Vec<_>>();
    let action_ids = [
        "inspect_lot_logger_state",
        "verify_receiving_record_completeness",
        "calculate_calibration_and_exposure",
        "verify_calibration_certificate",
    ];
    let action_event_ids = [
        "e-inspect-action",
        "e-completeness-action",
        "e-exposure-action",
        "e-calibration-action",
    ];
    let result_event_ids = [
        "e-inspect-result",
        "e-completeness-result",
        "e-exposure-result",
        "e-calibration-result",
    ];
    let result_ids = [
        "r-inspect-CC08-117",
        "r-completeness-CC08-117",
        "r-exposure-CC08-117",
        "r-calibration-CC08-117",
    ];
    let result_paths = [
        "bodies/receipt-inspection.txt",
        "bodies/receipt-completeness.txt",
        "bodies/receipt-exposure-check.txt",
        "bodies/receipt-calibration-check.txt",
    ];
    let result_revisions = [
        "lot-logger-inspection-CC08-v1",
        "receiving-completeness-CC08-v1",
        "exposure-check-CC08-v1",
        "calibration-check-CC08-v1",
    ];
    let receipt_hash_refs = receipt_hashes
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();

    let world_hash = digest(&workflow);
    let mut files = BTreeMap::new();
    for (path, body) in [
        ("workflow.md", workflow.clone()),
        ("source/CC-SOP-04-revision-3.2.1.md", sop.clone()),
        ("source/lot-snapshot-LOT-SYN-CC08-117-v1.json", lot.clone()),
        ("source/logger-record-LOGGER-SYN-14-v1.json", logger.clone()),
        (
            "source/calibration-CAL-SYN-14-2026-v1.json",
            calibration.clone(),
        ),
        (
            "source/quality-review-packet-recipe-v1.json",
            recipe.clone(),
        ),
        ("bodies/CC-SOP-04-original.md", sop.clone()),
        ("bodies/CC-SOP-04-final-packet.md", sop.clone()),
        ("bodies/lot-snapshot-LOT-SYN-CC08-117-v1.json", lot.clone()),
        ("bodies/logger-record-LOGGER-SYN-14-v1.json", logger.clone()),
        (
            "bodies/calibration-CAL-SYN-14-2026-v1.json",
            calibration.clone(),
        ),
        ("prompt/system.txt", SYSTEM.as_bytes().to_vec()),
        ("prompt/task.txt", TASK.as_bytes().to_vec()),
        ("prompt/request1.txt", REQUEST1.as_bytes().to_vec()),
        ("prompt/request2.txt", REQUEST2.as_bytes().to_vec()),
        ("prompt/request3.txt", REQUEST3.as_bytes().to_vec()),
    ] {
        files.insert(path.to_string(), body);
    }
    for (path, body) in result_paths.iter().zip(receipt_bodies.iter()) {
        files.insert((*path).to_string(), body.clone());
    }

    let events = vec![
        message_event(
            "e-sop-original",
            0,
            "ctx-sop-original",
            &sop_hash,
            sop_revision,
            world_revision,
            &[],
            "source/CC-SOP-04-revision-3.2.1.md#controlled-procedure",
            &sop_hash,
            &world_hash,
        ),
        message_event(
            "e-lot-source",
            1,
            "ctx-lot-source",
            &lot_hash,
            "cold-chain-lot@LOT-SYN-CC08-117-v1",
            world_revision,
            &[],
            "source/lot-snapshot-LOT-SYN-CC08-117-v1.json#frozen-record",
            &lot_prov_hash,
            &world_hash,
        ),
        message_event(
            "e-logger-evidence",
            2,
            "ctx-logger-evidence",
            &logger_hash,
            "cold-chain-logger@LOGGER-SYN-14-v1",
            world_revision,
            &["e-lot-source"],
            "source/logger-record-LOGGER-SYN-14-v1.json#logger-export",
            &logger_prov_hash,
            &world_hash,
        ),
        message_event(
            "e-calibration-evidence",
            3,
            "ctx-calibration-evidence",
            &calibration_hash,
            "logger-calibration@CAL-SYN-14-2026-v1",
            world_revision,
            &["e-lot-source"],
            "source/calibration-CAL-SYN-14-2026-v1.json#as-found-certificate",
            &calibration_prov_hash,
            &world_hash,
        ),
        action_event(
            action_event_ids[0],
            4,
            action_ids[0],
            &["e-sop-original", "e-lot-source", "e-logger-evidence"],
            world_revision,
            &world_hash,
            &lot_hash,
            &sop_hash,
        ),
        result_event(
            result_event_ids[0],
            5,
            result_ids[0],
            action_ids[0],
            receipt_hash_refs[0],
            &[action_event_ids[0], "e-lot-source", "e-logger-evidence"],
            world_revision,
            result_paths[0],
            result_revisions[0],
        ),
        action_event(
            action_event_ids[1],
            6,
            action_ids[1],
            &["e-lot-source"],
            world_revision,
            &world_hash,
            &lot_hash,
            &sop_hash,
        ),
        result_event(
            result_event_ids[1],
            7,
            result_ids[1],
            action_ids[1],
            receipt_hash_refs[1],
            &[action_event_ids[1], "e-lot-source"],
            world_revision,
            result_paths[1],
            result_revisions[1],
        ),
        action_event(
            action_event_ids[2],
            8,
            action_ids[2],
            &[
                "e-sop-original",
                "e-lot-source",
                "e-logger-evidence",
                "e-calibration-evidence",
            ],
            world_revision,
            &world_hash,
            &lot_hash,
            &sop_hash,
        ),
        result_event(
            result_event_ids[2],
            9,
            result_ids[2],
            action_ids[2],
            receipt_hash_refs[2],
            &[
                action_event_ids[2],
                "e-lot-source",
                "e-logger-evidence",
                "e-calibration-evidence",
            ],
            world_revision,
            result_paths[2],
            result_revisions[2],
        ),
        action_event(
            action_event_ids[3],
            10,
            action_ids[3],
            &["e-calibration-evidence"],
            world_revision,
            &world_hash,
            &lot_hash,
            &sop_hash,
        ),
        result_event(
            result_event_ids[3],
            11,
            result_ids[3],
            action_ids[3],
            receipt_hash_refs[3],
            &[action_event_ids[3], "e-calibration-evidence"],
            world_revision,
            result_paths[3],
            result_revisions[3],
        ),
        {
            let mut final_occurrence = message_event(
                "e-sop-final-reattachment",
                12,
                "ctx-sop-final-review",
                &sop_hash,
                sop_revision,
                world_revision,
                &[
                    "e-sop-original",
                    "e-lot-source",
                    "e-logger-evidence",
                    "e-calibration-evidence",
                ],
                "source/CC-SOP-04-revision-3.2.1.md#controlled-procedure",
                &sop_hash,
                &world_hash,
            );
            final_occurrence["provenance"]
                .as_array_mut()
                .unwrap()
                .push(provenance(
                    "source/quality-review-packet-recipe-v1.json#procedure-occurrence",
                    "cold-chain-final-quality-review@1.0",
                    &recipe,
                ));
            final_occurrence
        },
    ];

    let relations = vec![
        relation(
            "produce-inspection",
            "produces",
            action_ids[0],
            result_ids[0],
            result_paths[0],
            result_revisions[0],
            &receipt_hashes[0],
        ),
        relation(
            "produce-completeness",
            "produces",
            action_ids[1],
            result_ids[1],
            result_paths[1],
            result_revisions[1],
            &receipt_hashes[1],
        ),
        relation(
            "produce-exposure-check",
            "produces",
            action_ids[2],
            result_ids[2],
            result_paths[2],
            result_revisions[2],
            &receipt_hashes[2],
        ),
        relation(
            "produce-calibration-check",
            "produces",
            action_ids[3],
            result_ids[3],
            result_paths[3],
            result_revisions[3],
            &receipt_hashes[3],
        ),
        relation(
            "same-state-sop-reattachment",
            "same_state_revision",
            "e-sop-original",
            "e-sop-final-reattachment",
            "cp08/source/CC-SOP-04-revision-3.2.1.md#controlled-procedure",
            sop_revision,
            &sop_hash,
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
            "sop_original",
            "bodies/CC-SOP-04-original.md",
            "context_attachment",
            Some("e-sop-original"),
            Some(sop_revision),
            &sop,
        ),
        asset(
            "lot_snapshot",
            "bodies/lot-snapshot-LOT-SYN-CC08-117-v1.json",
            "context_attachment",
            Some("e-lot-source"),
            Some("cold-chain-lot@LOT-SYN-CC08-117-v1"),
            &lot,
        ),
        asset(
            "logger_evidence",
            "bodies/logger-record-LOGGER-SYN-14-v1.json",
            "context_attachment",
            Some("e-logger-evidence"),
            Some("cold-chain-logger@LOGGER-SYN-14-v1"),
            &logger,
        ),
        asset(
            "calibration_evidence",
            "bodies/calibration-CAL-SYN-14-2026-v1.json",
            "context_attachment",
            Some("e-calibration-evidence"),
            Some("logger-calibration@CAL-SYN-14-2026-v1"),
            &calibration,
        ),
        asset(
            "sop_final_packet",
            "bodies/CC-SOP-04-final-packet.md",
            "context_attachment",
            Some("e-sop-final-reattachment"),
            Some(sop_revision),
            &sop,
        ),
    ];
    for index in 0..4 {
        assets.push(asset(
            [
                "receipt_inspection",
                "receipt_completeness",
                "receipt_exposure",
                "receipt_calibration",
            ][index],
            result_paths[index],
            "environment_receipt",
            Some(result_event_ids[index]),
            Some(result_revisions[index]),
            &receipt_bodies[index],
        ));
    }

    let slot1_messages = vec![
        message("system", vec![asset_part("system")]),
        message(
            "user",
            vec![
                asset_part("task"),
                asset_part("request1"),
                event_part("e-sop-original"),
                event_part("e-lot-source"),
                event_part("e-logger-evidence"),
            ],
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
    slot2_messages.push(message(
        "user",
        vec![asset_part("request2"), event_part("e-calibration-evidence")],
    ));
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
        vec![event_part("e-sop-final-reattachment")],
    ));

    let expected_final_answer = json!({
        "disposition":"HOLD",
        "governing_rule_ids":["CC-04","CC-05","CC-06","CC-08"],
        "time_temperature_calculation":{
            "storage_range_celsius_tenths":{"minimum_inclusive":20,"maximum_inclusive":80},
            "interval_minutes":15,
            "corrected_interval_extrema":[
                {"start":"2026-03-14T08:00:00Z","end":"2026-03-14T08:15:00Z","minimum_celsius_tenths":48,"maximum_celsius_tenths":53,"status":"COMPLIANT"},
                {"start":"2026-03-14T08:15:00Z","end":"2026-03-14T08:30:00Z","minimum_celsius_tenths":79,"maximum_celsius_tenths":85,"status":"HIGH_EXCURSION"},
                {"start":"2026-03-14T08:30:00Z","end":"2026-03-14T08:45:00Z","minimum_celsius_tenths":66,"maximum_celsius_tenths":74,"status":"COMPLIANT"},
                {"start":"2026-03-14T08:45:00Z","end":"2026-03-14T09:00:00Z","minimum_celsius_tenths":55,"maximum_celsius_tenths":63,"status":"COMPLIANT"}
            ],"compliant_interval_count":3,"high_excursion_interval_count":1,"low_excursion_interval_count":0
        },
        "cumulative_exposure_calculation":{"high_minutes":15,"low_minutes":0,"total_excursion_minutes":15,"high_degree_minutes_celsius_tenths":75,"low_degree_minutes_celsius_tenths":0,"total_degree_minutes_celsius_tenths":75},
        "calibration":{"status":"VALID","calibration_id":"CAL-SYN-14-2026","reference_celsius_tenths":50,"indicated_celsius_tenths":48,"correction_celsius_tenths":2,"absolute_error_celsius_tenths":2,"maximum_allowed_error_celsius_tenths":5,"valid_through":"2027-01-10","applied_correction_celsius_tenths":2},
        "required_hold_or_escalation_reason":"One corrected high excursion lasted 15 minutes and totals 75 tenths of C-minute. Identity, calibration, and logger continuity pass; the exposure is within CC-08 review boundaries, so segregate the lot on HOLD pending quality review before use."
    });
    let evaluation_key = json!({
        "expected_action_ids":[action_ids[0],action_ids[2]],
        "expected_states_after_action":["lot_logger_state_inspected","calibration_exposure_verified"],
        "expected_result_event_ids":[result_event_ids[0],result_event_ids[2]],
        "expected_final_answer":expected_final_answer,
        "required_event_ids":["e-sop-original","e-lot-source","e-logger-evidence","e-calibration-evidence","e-inspect-result","e-exposure-result"],
        "required_relation_ids":["produce-inspection","produce-exposure-check"],
        "critical_event_ids":["e-sop-original","e-lot-source","e-logger-evidence","e-calibration-evidence","e-exposure-result"]
    });
    let evaluation_bytes = json_bytes(&evaluation_key);

    let expected_actions = [action_ids[0], action_ids[2]];
    let expected_results = [result_event_ids[0], result_event_ids[2]];
    let expected_result_ids = [result_ids[0], result_ids[2]];
    let expected_paths = [result_paths[0], result_paths[2]];
    let expected_states = [
        "lot_logger_state_inspected",
        "calibration_exposure_verified",
    ];
    let advancing_points = (0..2).map(|index| {
        let raw = format!("{{\"action_id\":\"{}\"}}",expected_actions[index]);
        json!({"request_slot":index+1,"expected_action_id":expected_actions[index],"canonical_raw_utf8":raw,"canonical_raw_sha256":digest(raw.as_bytes()),"raw_language_cardinality":1,"receipt_event_id":expected_results[index],"receipt_id":expected_result_ids[index],"receipt_path":expected_paths[index],"receipt_sha256":receipt_hashes[[0,2][index]],"receipt_revision":result_revisions[[0,2][index]],"state_after":expected_states[index]})
    }).collect::<Vec<_>>();
    let advancing_domain = json_bytes(
        &json!({"schema_id":"prefixity.phase1c.claim2-advancing-output-domain-v2-case","schema_version":2,"case_id":"CP08","protocol_id":"prefixity.phase1c.claim2-canonical-advancing-output.v1","offline_only":true,"points":advancing_points}),
    );

    let action_menu = vec![
        json!({"action_slot":1,"action_id":action_ids[0],"action_event_id":action_event_ids[0],"result_event_id":result_event_ids[0],"result_asset_id":"receipt_inspection","state_after":"lot_logger_state_inspected"}),
        json!({"action_slot":1,"action_id":action_ids[1],"action_event_id":action_event_ids[1],"result_event_id":result_event_ids[1],"result_asset_id":"receipt_completeness","state_after":"receiving_record_completeness_verified"}),
        json!({"action_slot":2,"action_id":action_ids[2],"action_event_id":action_event_ids[2],"result_event_id":result_event_ids[2],"result_asset_id":"receipt_exposure","state_after":"calibration_exposure_verified"}),
        json!({"action_slot":2,"action_id":action_ids[3],"action_event_id":action_event_ids[3],"result_event_id":result_event_ids[3],"result_asset_id":"receipt_calibration","state_after":"calibration_certificate_verified"}),
    ];
    let manifest = json!({
        "schema_id":"prefixity.phase1c.claim2-workload-case","schema_version":1,"case_id":"CP08","kind":"positive",
        "planner_input":{"events":events,"relations":relations,"provenance":[
            provenance("cp08/workflow.md","cp08-cold-chain-workflow-v1",&workflow),
            provenance("cp08/source/CC-SOP-04-revision-3.2.1.md",sop_revision,&sop),
            provenance("cp08/source/lot-snapshot-LOT-SYN-CC08-117-v1.json","cold-chain-lot@LOT-SYN-CC08-117-v1",&lot),
            provenance("cp08/source/logger-record-LOGGER-SYN-14-v1.json","cold-chain-logger@LOGGER-SYN-14-v1",&logger),
            provenance("cp08/source/calibration-CAL-SYN-14-2026-v1.json","logger-calibration@CAL-SYN-14-2026-v1",&calibration),
            provenance("cp08/source/quality-review-packet-recipe-v1.json","cold-chain-final-quality-review@1.0",&recipe)
        ]},
        "assets":assets,"action_menu":action_menu,
        "request_templates":[{"request_slot":1,"messages":slot1_messages},{"request_slot":2,"messages":slot2_messages},{"request_slot":3,"messages":slot3_messages}],
        "evaluation_key_path":"evaluation/key.json","evaluation_key_sha256":digest(&evaluation_bytes),"assistant_output_planning_bytes":4096,"token_proof_inputs":null
    });
    files.insert("evaluation/key.json".to_string(), evaluation_bytes);
    files.insert("case.json".to_string(), json_bytes(&manifest));
    files.insert(
        "advancing-output-domain-v2.json".to_string(),
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
