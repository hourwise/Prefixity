use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;

const WORKFLOW: &str = r#"# CP07 synthetic benefits adjudication workflow, version 1

This benchmark fixture is wholly synthetic. `BEN-CP07-041`, `MEM-SYN-071`,
`FAC-SYN-019`, and all references below are fictional identifiers. The source
builder in `examples/claim2_cp07_fixture.rs` creates the policy, frozen case
snapshot, ordinary packet recipe, pinned receipts, event trace, and hidden key
from fixed inputs. It performs no network, provider, or model operation.

## Ordinary case sequence

The intake workspace opens the applicable plan document beside the submitted
claim snapshot. The eligibility desk performs a read-only membership, service,
network, and referral check against those two fixed records. A second desk step
performs the standard allowed-charge, deductible, coinsurance, and annual
benefit-limit arithmetic over the same snapshot. Both checks leave the source
records and their revisions unchanged.

When a case reaches final adjudication, the normal `benefit-final-review`
packet recipe carries forward the already attached frozen case snapshot and
attaches the controlling plan document again. Reviewers receive the authority
document in the adjudication packet itself so that the clause set used for the
signed disposition travels with the decision record. This packet recipe is
applied to every final adjudication request; it does not depend on document
length, repeated context, or any benchmark condition. The second native
occurrence is produced by this ordinary packet assembly from the same
content-addressed plan object.

## Frozen source state

The plan body is `source/northstar-benefit-plan-2026.1.md`, source revision
`northstar-benefit-plan@2026.1`. The case body is
`source/case-snapshot-BEN-CP07-041-v1.json`, source revision
`benefits-case@BEN-CP07-041-v1`. The case snapshot pins the plan identifier and
the exact plan-body SHA-256. Both are read-only for the complete three-request
trajectory. The original policy occurrence is `e-plan-original`; the final
packet occurrence is `e-plan-final-reattachment`. Their body bytes, source
revision, and world-state revision are identical. The earlier occurrence is
retained. The final occurrence has no consumer and no protected relation.

## Deterministic checks

Request 1 exposes the finite choices `inspect_coverage_eligibility` and
`verify_intake_completeness`. The first returns the eligibility receipt pinned
to the claim snapshot, plan version, service, facility network status, and
referral record. Request 2 exposes `calculate_benefit_breakdown` and
`verify_plan_publication`. The first computes integer-cent deductible,
coinsurance, annual-limit, and member-responsibility values from the unchanged
snapshot and the plan clauses. The receipts include their input revisions and
source hashes; neither action changes a source record.

The final answer is exact structured JSON. Its hidden evaluation key is kept
under `evaluation/` and is not a prompt asset or planner-visible event. The
public policy and case facts define the arithmetic; the pinned check receipt
provides an independently traceable deterministic verification result.

## Provenance limits

The source files describe a closed synthetic world, not a real plan, member,
claim, facility, or administrative system. Naturalness here means the
reattachment follows the stated ordinary final-review packet recipe within
that frozen synthetic workflow. No size-derived threshold or tokenizer result
was used to create the policy or the reattachment.
"#;

const POLICY: &str = r#"NORTHSTAR WORKFORCE BENEFIT PLAN
Outpatient Diagnostic Imaging Benefit Policy
Document ID: NS-WB-IMG-POLICY
Version: 2026.1
Effective date: 2026-01-01
Issued by: Northstar Synthetic Benefits Trust
Status: complete governing schedule for outpatient diagnostic imaging claims

1. PURPOSE, SCOPE, AND AUTHORITY

This schedule states the complete coverage and payment rules for outpatient
diagnostic imaging claims submitted under the Northstar Workforce Benefit
Plan, product NS-WB-2026. It governs eligibility, covered service classes,
network conditions, member cost sharing, the annual imaging benefit limit,
exclusions, adjudication order, notice, and review. It applies to services on
or after its effective date during the 2026 plan year. Monetary values in
claim records and decisions are United States dollars represented in integer
cents. This document is a synthetic benchmark policy and has no real issuer or
beneficiary.

Only this numbered version controls the synthetic claims that identify
NS-WB-IMG-POLICY version 2026.1. The version identifier is immutable for a
claim once assigned. A later plan publication does not change a frozen claim's
source revision or the rules applied to it.

2. DEFINITIONS

2.1 Allowed charge is the lower of the submitted charge and the plan's
contracted amount for the service, after ordinary coding and network edits.
The allowed charge is supplied as a verified case-source fact; adjudicators
must not substitute the billed charge.

2.2 Plan year is the calendar year containing the service date. Deductible,
prior benefit payments, and the annual imaging limit are measured separately
for each covered member and plan year.

2.3 In-network facility is a facility whose network status is active on the
service date in the frozen case source. A facility directory row alone does
not establish that the member or service is eligible.

2.4 Experimental service is an imaging method or device documented by the
claim source as not established for ordinary clinical use and not listed in
the covered service schedule. The exclusion in clause EXCL-8.4 applies only
when both conditions are present in the source evidence.

2.5 DEF-2.5 - Plan benefit payment is the amount payable by the plan after deductible
and coinsurance, limited by clause LIMIT-4.1. Member responsibility is the
allowed charge minus the final plan benefit payment. It includes deductible,
coinsurance, and any allowed charge left unpaid because of the annual limit.

3. MEMBER AND SERVICE ELIGIBILITY

3.1 ELIG-3.1 - A member is eligible for a service when enrollment is active on its date,
the identified product is NS-WB-2026, and the service falls inside the plan
year. Eligibility is based on the member and product facts in the unchanged
case source.

3.2 COV-3.2 - Covered outpatient diagnostic imaging includes standard computed
tomography, magnetic-resonance, and ultrasound services identified in the
published imaging schedule. Service code IMG-CT-04 is standard outpatient
computed tomography. A covered service must be performed at an in-network
facility and must be supported by an effective referral where clause 3.3
requires one.

3.3 COV-3.3 - IMG-CT-04 requires a referring-clinician order recorded before the service
date. The referral must identify the member, the imaging service class, and a
valid interval containing the service date. A referral is evidence of
eligibility; it does not set the allowed charge or change member cost sharing.

3.4 A service outside the published covered schedule is not covered under
this imaging benefit. An incomplete claim may be returned for records review;
it is not converted to a covered claim merely because a referral exists.

4. NETWORK AND CHARGE BASIS

4.1 NET-4.1 - The frozen case source supplies the network result for the service date.
Only the `in_network` value is eligible for this schedule's ordinary benefit
calculation. Out-of-network charges are not repriced under the in-network
terms in this document.

4.2 CHARGE-4.2 - The verified allowed charge is the basis for all calculations below. A
submitted charge greater than the allowed charge creates no additional plan
benefit. The plan does not round intermediate values because all rates and
amounts in this schedule produce whole cents for the claims admitted to this
synthetic workflow.

5. CLAIM RECORDS AND READ-ONLY VERIFICATION

5.1 The case-source snapshot is the controlling record for enrollment,
product, service date and code, facility, referral, allowed charge, remaining
deductible, annual imaging cap, prior cap payments, and exclusion facts.

5.2 Eligibility review and payment arithmetic are read-only operations. They
must cite the snapshot revision and must not alter enrollment, referral,
claims history, deductible balance, cap history, or plan text.

5.3 The eligibility receipt records the active-member, product, covered-code,
network, and referral findings. The benefit-check receipt records each
arithmetic input and result. A receipt is evidence of the check performed; it
does not create or revise the source fact that it verifies.

6. DEDUCTIBLE AND COINSURANCE

6.1 COST-6.1 - The annual individual imaging deductible is $1,500.00. For a claim, the
deductible amount applied is the lower of the allowed charge and the
member's remaining deductible immediately before the claim. The applied
amount is member responsibility. Deductible remaining after the claim equals
the prior remaining amount minus the amount applied.

6.2 COST-6.2 - After the deductible in clause 6.1, the member pays 20 percent of the
remaining allowed charge as coinsurance. The plan's pre-limit amount is the
other 80 percent. Compute both amounts from integer cents. The coinsurance
calculation does not include a second deductible and does not reset within a
claim.

6.3 There is no separate copayment for IMG-CT-04. If eligibility is absent,
the claim receives no benefit under this schedule and the cost-share
calculation is not used to imply coverage.

7. ANNUAL IMAGING BENEFIT LIMIT

7.1 LIMIT-7.1 - The maximum plan benefit payment for covered outpatient imaging is
$10,000.00 per member per plan year. Prior payment history is supplied as
plan payments already applied to this limit; it is not the member's allowed
charge history.

7.2 LIMIT-7.2 - Remaining limit equals the annual maximum minus prior applied plan
payments, floored at zero. The payable plan benefit is the lower of the
pre-limit plan amount under clause 6.2 and the remaining limit. A negative
remaining amount is treated as zero.

7.3 LIMIT-7.3 - The difference between the pre-limit plan amount and the payable benefit
is unpaid because the annual plan limit has been reached. Under clause 2.5,
that difference remains part of member responsibility. It must be reported
separately from deductible and coinsurance.

8. EXCLUSIONS

8.1 A claim must be covered under clauses 3 and 4 before payment terms are
applied. Exclusions are evaluated against the exact facts in the frozen case
source and the plan revision pinned to that case.

8.2 Services performed at an out-of-network facility are outside this
schedule's in-network benefit. This network restriction is evaluated under
clause 4.1.

8.3 A service without the referral required by clause 3.3 is not eligible for
this schedule. Missing referral evidence is an eligibility failure, not a
deductible or limit adjustment.

8.4 EXCL-8.4 - EXPERIMENTAL IMAGING METHOD OR DEVICE. This exclusion is
triggered only when the frozen claim source records both (a) an experimental
method or device and (b) absence from the published covered schedule. When
triggered, no plan benefit is payable for that service. When either condition
is false, this exclusion is not triggered. For IMG-CT-04 performed with the
standard listed method, a documented false experimental-method fact means
EXCL-8.4 is not triggered.

8.5 No other exclusion modifies the eligible standard service, allowed
charge, deductible, coinsurance, or annual limit calculations in this
schedule. The final decision must identify the applicable exclusion check
and its triggered status rather than infer one from a generic denial rule.

9. ADJUDICATION ORDER

9.1 ADJ-9.1 - Verify member, product, service date, code, facility, and referral under
clauses 3 and 4. Record a coverage status of COVERED only if each required
eligibility condition is met.

9.2 ADJ-9.2 - Evaluate the experimental-service condition in EXCL-8.4 using the frozen
source facts. If it is triggered, set the plan payment to zero and identify
the exclusion. Do not apply deductible, coinsurance, or annual-limit
calculations to an excluded service.

9.3 ADJ-9.3 - For an eligible non-excluded service, apply the remaining deductible to
the allowed charge, compute member and plan shares under clause 6, then apply
the remaining annual limit under clause 7. Keep the deductible, coinsurance,
limit adjustment, plan benefit, and total member responsibility as distinct
amounts.

9.4 ADJ-9.4 - A covered claim whose payable plan benefit is reduced by clause 7 is
adjudicated as COVERED_WITH_CAP_LIMIT. The reduced benefit does not change
the coverage status of the service.

10. NOTICE, REVIEW, AND RECORD RETENTION

10.1 The final notice identifies the policy version, covered service,
governing clause IDs, eligibility and exclusion result, the allowed charge,
each cost-share calculation, remaining limit, plan benefit, and member
responsibility. Amounts are reported in integer cents in the synthetic
decision record.

10.2 A member may request a clerical review within 60 calendar days of the
notice. Review checks source identity and arithmetic against the version
assigned to the claim. It does not mutate the source snapshot or silently
replace the governing plan version.

10.3 The adjudication packet retains its case-source revision, policy source
revision, check receipts, and final disposition together. Each final review
packet contains the complete controlling policy body as a native attachment
so the decision can be read with the authority applied to it.

11. VERSION CONTROL AND INTERPRETATION

11.1 Version 2026.1 is effective from 2026-01-01 through 2026-12-31 for this
synthetic product. Its clauses are applied as written to a case that pins
this version and remains unchanged during review.

11.2 Where an operational receipt reports an input, use the corresponding
case-source value. Where the case source and this schedule jointly determine
a result, follow the order in clause 9. A receipt cannot amend a clause, and
the evaluation answer cannot change source facts.

11.3 This document contains the complete terms needed to decide an
IMG-CT-04 outpatient imaging claim under NS-WB-2026: eligibility, covered
service, network, referral, allowed-charge basis, deductible, coinsurance,
annual plan limit, exclusion, adjudication, notice, review, and version
control. No external policy document or unstated rule is required for this
synthetic case.
"#;

const SYSTEM: &str = "You are completing a read-only deterministic synthetic benefits adjudication. At request slots 1 and 2, return exactly one JSON object with the single string field action_id and select an action from the displayed menu. At request slot 3, return exactly one JSON object with the answer field and the requested facts. Do not invent identifiers, calculations, or fields.\n";
const TASK: &str = "CP07 synthetic benefits coverage adjudication. Use the pinned case snapshot, the published Northstar plan, and the deterministic verification receipts to decide the claim under the cited plan revision. The claim snapshot and plan are read-only.\n";
const REQUEST1: &str = "Check member, product, service, network, and referral eligibility against the versioned plan and frozen claim source. Choose one read-only action: {\"action_id\":\"inspect_coverage_eligibility\"} or {\"action_id\":\"verify_intake_completeness\"}.\n";
const REQUEST2: &str = "Perform the required allowed-charge, remaining-deductible, coinsurance, annual benefit-limit, and exclusion verification using the same unchanged source snapshot. Choose one read-only action: {\"action_id\":\"calculate_benefit_breakdown\"} or {\"action_id\":\"verify_plan_publication\"}.\n";
const REQUEST3: &str = "Complete the final adjudication from the source snapshot, plan terms, and both check receipts. Return exact JSON fields: coverage_status; governing_clause_ids; deductible (allowed_charge_cents, remaining_before_cents, applied_cents, after_deductible_cents); coinsurance (member_rate_basis_points, member_share_cents, plan_before_limit_cents); benefit_limit (annual_cap_cents, paid_before_cents, remaining_cents, plan_payment_cents, limit_reduction_cents); exclusion (clause_id, triggered); member_responsibility_cents; plan_benefit_cents; final_disposition.\n";

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
    asset_id: &str,
    relative_path: &str,
    kind: &str,
    event_id: Option<&str>,
    revision_id: Option<&str>,
    bytes: &[u8],
) -> Value {
    let mut value = json!({
        "asset_id": asset_id,
        "relative_path": relative_path,
        "sha256": digest(bytes),
        "kind": kind
    });
    if let Some(event_id) = event_id {
        value["event_id"] = json!(event_id);
    }
    if let Some(revision_id) = revision_id {
        value["revision_id"] = json!(revision_id);
    }
    value
}

// Fixture records carry separate immutable source and workflow provenance.
#[allow(clippy::too_many_arguments)]
fn message_event(
    event_id: &str,
    sequence: u32,
    context_id: &str,
    body_sha256: &str,
    source_revision: &str,
    world_revision: &str,
    references: &[&str],
    locator: &str,
    source_hash: &str,
    workflow_hash: &str,
) -> Value {
    json!({
        "event_id": event_id,
        "sequence_index": sequence,
        "event_type": "message",
        "actor_role": "user",
        "parent_event_ids": [],
        "reference_event_ids": references,
        "action": null,
        "result": null,
        "context_block_id": context_id,
        "world_state_revision": world_revision,
        "order": { "logical_tick": sequence, "timestamp_origin": "derived_structural" },
        "content_hash": body_sha256,
        "provenance": [
            { "source_kind": "self_authored", "classification": "CAPTURED_EXPLICIT", "source_locator": locator, "source_revision": source_revision, "content_hash": source_hash },
            { "source_kind": "self_authored", "classification": "CAPTURED_EXPLICIT", "source_locator": "cp07/workflow.md#ordinary-packet-provenance", "source_revision": "cp07-benefit-workflow-v1", "content_hash": workflow_hash }
        ]
    })
}

#[allow(clippy::too_many_arguments)]
fn action_event(
    event_id: &str,
    sequence: u32,
    action_id: &str,
    references: &[&str],
    world_revision: &str,
    workflow_hash: &str,
    snapshot_hash: &str,
    policy_hash: &str,
) -> Value {
    let args = json!({
        "case_id": "BEN-CP07-041",
        "snapshot_sha256": snapshot_hash,
        "policy_sha256": policy_hash
    });
    json!({
        "event_id": event_id,
        "sequence_index": sequence,
        "event_type": "action",
        "actor_role": "agent",
        "parent_event_ids": [],
        "reference_event_ids": references,
        "action": {
            "action_id": action_id,
            "tool_name": action_id,
            "argument_hash": digest(&serde_json::to_vec(&args).expect("action arguments serialize"))
        },
        "result": null,
        "context_block_id": null,
        "world_state_revision": world_revision,
        "order": { "logical_tick": sequence, "timestamp_origin": "derived_structural" },
        "content_hash": null,
        "provenance": [
            { "source_kind": "self_authored", "classification": "CAPTURED_EXPLICIT", "source_locator": "cp07/workflow.md#read-only-action-menu", "source_revision": "cp07-benefit-workflow-v1", "content_hash": workflow_hash }
        ]
    })
}

#[allow(clippy::too_many_arguments)]
fn result_event(
    event_id: &str,
    sequence: u32,
    result_id: &str,
    action_id: &str,
    observation_hash: &str,
    references: &[&str],
    world_revision: &str,
    receipt_path: &str,
    receipt_revision: &str,
    receipt_hash: &str,
) -> Value {
    json!({
        "event_id": event_id,
        "sequence_index": sequence,
        "event_type": "result",
        "actor_role": "tool",
        "parent_event_ids": [],
        "reference_event_ids": references,
        "action": null,
        "result": {
            "result_id": result_id,
            "originating_action_id": action_id,
            "observation_hash": observation_hash,
            "status": "success"
        },
        "context_block_id": null,
        "world_state_revision": world_revision,
        "order": { "logical_tick": sequence, "timestamp_origin": "derived_structural" },
        "content_hash": null,
        "provenance": [
            { "source_kind": "self_authored", "classification": "CAPTURED_EXPLICIT", "source_locator": receipt_path, "source_revision": receipt_revision, "content_hash": receipt_hash }
        ]
    })
}

fn relation(
    relation_id: &str,
    kind: &str,
    from: &str,
    to: &str,
    locator: &str,
    revision: &str,
    source_hash: &str,
) -> Value {
    json!({
        "relation_id": relation_id,
        "relation_type": kind,
        "from_id": from,
        "to_id": to,
        "scope": "scenario_local",
        "semantics_version": "controlled-benchmark-relations-v1",
        "provenance": [{
            "source_kind": "self_authored",
            "classification": "CAPTURED_EXPLICIT",
            "source_locator": locator,
            "source_revision": revision,
            "content_hash": source_hash
        }]
    })
}

fn message(role: &str, parts: Vec<Value>) -> Value {
    json!({ "role": role, "parts": parts })
}

fn asset_part(asset_id: &str) -> Value {
    json!({ "kind": "asset", "asset_id": asset_id })
}

fn event_part(event_id: &str) -> Value {
    json!({ "kind": "event_body", "event_id": event_id })
}

fn generated_files() -> BTreeMap<String, Vec<u8>> {
    let workflow = WORKFLOW.as_bytes().to_vec();
    let workflow_hash = digest(&workflow);
    let policy = POLICY.as_bytes().to_vec();
    let policy_hash = digest(&policy);
    let source_revision = "northstar-benefit-plan@2026.1";
    let world_revision = "benefits-adjudication@BEN-CP07-041-v1";

    let snapshot = json_bytes(&json!({
        "schema_id": "prefixity.claim2.synthetic-benefits-case-snapshot",
        "schema_version": 1,
        "case_id": "BEN-CP07-041",
        "synthetic_only": true,
        "snapshot_id": "benefits-source-snapshot-CP07-041-v1",
        "world_state_revision": world_revision,
        "source_revision": "benefits-case@BEN-CP07-041-v1",
        "plan_document_id": "NS-WB-IMG-POLICY",
        "plan_version": "2026.1",
        "plan_source_revision": source_revision,
        "plan_document_sha256": policy_hash,
        "member": {
            "synthetic_member_id": "MEM-SYN-071",
            "product_id": "NS-WB-2026",
            "enrollment_status_on_service_date": "active",
            "plan_year": 2026
        },
        "claim": {
            "claim_id": "BEN-CP07-041",
            "service_date": "2026-02-24",
            "service_code": "IMG-CT-04",
            "service_description": "standard outpatient computed tomography",
            "facility_id": "FAC-SYN-019",
            "network_status_on_service_date": "in_network",
            "referral": {
                "referral_id": "REF-CP07-081",
                "member_id": "MEM-SYN-071",
                "service_class": "IMG-CT-04",
                "valid_from": "2026-02-01",
                "valid_through": "2026-03-31",
                "recorded_before_service": true
            },
            "submitted_charge_cents": 612000,
            "verified_allowed_charge_cents": 420000
        },
        "accounting_before_claim": {
            "remaining_individual_imaging_deductible_cents": 50000,
            "annual_individual_imaging_benefit_limit_cents": 1000000,
            "prior_plan_payments_applied_to_imaging_limit_cents": 820000
        },
        "exclusion_facts": {
            "method_or_device_is_experimental": false,
            "service_is_listed_in_published_schedule": true,
            "out_of_network": false,
            "referral_present_and_valid": true
        },
        "source_record_state": "frozen_read_only"
    }));
    let snapshot_hash = digest(&snapshot);
    let recipe = json_bytes(&json!({
        "schema_id": "prefixity.claim2.synthetic-benefits-packet-recipe",
        "schema_version": 1,
        "template_id": "benefit-final-review",
        "template_version": "1.0",
        "ordinary_trigger": "final_adjudication_requested",
        "required_packet_material": [
            "existing_frozen_case_source_snapshot_occurrence:e-case-source",
            "complete_controlling_plan_document"
        ],
        "case_snapshot_behavior": "carry_forward_existing_native_occurrence_without_copy",
        "plan_resolution": {
            "document_id": "NS-WB-IMG-POLICY",
            "version": "2026.1",
            "source_revision": source_revision,
            "body_sha256": policy_hash
        },
        "attachment_behavior": "emit_one_native_plan_message_for_each_final_packet",
        "read_only": true,
        "purpose": "Keep the authority document with the final decision record for clause-level review.",
        "benchmark_condition_used": false
    }));

    let eligibility_receipt = format!(
        "Eligibility check ELI-CP07-041-01: case=BEN-CP07-041; snapshot=benefits-source-snapshot-CP07-041-v1; snapshot_sha256={snapshot_hash}; plan=NS-WB-IMG-POLICY@2026.1; plan_sha256={policy_hash}; member=MEM-SYN-071; enrollment=active_on_2026-02-24; product=NS-WB-2026; service=IMG-CT-04; schedule_status=listed; facility=FAC-SYN-019; network=in_network; referral=REF-CP07-081; referral_valid_on_service_date=true; source_state=frozen_read_only.\n"
    );
    let intake_receipt = format!(
        "Intake completeness check INTAKE-CP07-041-01: case=BEN-CP07-041; snapshot_sha256={snapshot_hash}; received_fields=member,product,service_date,service_code,facility,referral,submitted_charge,allowed_charge; missing_required_fields=0; coverage_and_payment_not_adjudicated.\n"
    );
    let calculation_receipt = format!(
        "Benefit verification BNF-CP07-041-01: case=BEN-CP07-041; snapshot_sha256={snapshot_hash}; plan_sha256={policy_hash}; allowed_charge_cents=420000; remaining_deductible_before_cents=50000; deductible_applied_cents=50000; after_deductible_cents=370000; member_coinsurance_basis_points=2000; member_coinsurance_cents=74000; plan_before_limit_cents=296000; annual_imaging_limit_cents=1000000; prior_plan_payments_cents=820000; remaining_limit_cents=180000; plan_payment_after_limit_cents=180000; limit_reduction_cents=116000; member_responsibility_cents=240000; experimental_method=false; listed_service=true; exclusion_EXCL-8.4_triggered=false; source_state=frozen_read_only.\n"
    );
    let publication_receipt = format!(
        "Publication verification PUB-CP07-041-01: document=NS-WB-IMG-POLICY; version=2026.1; source_revision={source_revision}; body_sha256={policy_hash}; status=published_for_2026_plan_year; arithmetic_not_performed.\n"
    );

    let receipt_hashes = [
        digest(eligibility_receipt.as_bytes()),
        digest(intake_receipt.as_bytes()),
        digest(calculation_receipt.as_bytes()),
        digest(publication_receipt.as_bytes()),
    ];
    let action_ids = [
        "inspect_coverage_eligibility",
        "verify_intake_completeness",
        "calculate_benefit_breakdown",
        "verify_plan_publication",
    ];
    let action_event_ids = [
        "e-eligibility-action",
        "e-intake-action",
        "e-benefit-action",
        "e-publication-action",
    ];
    let result_event_ids = [
        "e-eligibility-result",
        "e-intake-result",
        "e-benefit-result",
        "e-publication-result",
    ];
    let result_ids = [
        "r-eligibility-CP07-041",
        "r-intake-CP07-041",
        "r-benefit-CP07-041",
        "r-publication-CP07-041",
    ];
    let result_paths = [
        "bodies/receipt-eligibility.txt",
        "bodies/receipt-intake.txt",
        "bodies/receipt-benefit-check.txt",
        "bodies/receipt-publication.txt",
    ];
    let result_revisions = [
        "eligibility-check-CP07-041-v1",
        "intake-check-CP07-041-v1",
        "benefit-check-CP07-041-v1",
        "publication-check-CP07-041-v1",
    ];
    let result_bodies = [
        eligibility_receipt.as_bytes(),
        intake_receipt.as_bytes(),
        calculation_receipt.as_bytes(),
        publication_receipt.as_bytes(),
    ];

    let policy_original_path = "bodies/northstar-plan-original.md";
    let policy_repeat_path = "bodies/northstar-plan-final-packet.md";
    let snapshot_path = "bodies/case-snapshot-BEN-CP07-041-v1.json";
    let policy_source_path = "source/northstar-benefit-plan-2026.1.md";
    let snapshot_source_path = "source/case-snapshot-BEN-CP07-041-v1.json";
    let recipe_path = "source/benefit-final-review-recipe-v1.json";

    let mut files = BTreeMap::new();
    files.insert("workflow.md".to_string(), workflow.clone());
    files.insert(policy_source_path.to_string(), policy.clone());
    files.insert(snapshot_source_path.to_string(), snapshot.clone());
    files.insert(recipe_path.to_string(), recipe.clone());
    files.insert(policy_original_path.to_string(), policy.clone());
    files.insert(policy_repeat_path.to_string(), policy.clone());
    files.insert(snapshot_path.to_string(), snapshot.clone());
    files.insert("prompt/system.txt".to_string(), SYSTEM.as_bytes().to_vec());
    files.insert("prompt/task.txt".to_string(), TASK.as_bytes().to_vec());
    files.insert(
        "prompt/request1.txt".to_string(),
        REQUEST1.as_bytes().to_vec(),
    );
    files.insert(
        "prompt/request2.txt".to_string(),
        REQUEST2.as_bytes().to_vec(),
    );
    files.insert(
        "prompt/request3.txt".to_string(),
        REQUEST3.as_bytes().to_vec(),
    );
    for (path, body) in result_paths.iter().zip(result_bodies) {
        files.insert((*path).to_string(), body.to_vec());
    }

    let world_state_revision = "benefits-adjudication@BEN-CP07-041-v1";
    let policy_provenance_hash = digest(&policy);
    let snapshot_provenance_hash = digest(&snapshot);
    let events = vec![
        message_event(
            "e-plan-original",
            0,
            "ctx-plan-original",
            &policy_hash,
            source_revision,
            world_state_revision,
            &[],
            "source/northstar-benefit-plan-2026.1.md#document",
            &policy_provenance_hash,
            &workflow_hash,
        ),
        message_event(
            "e-case-source",
            1,
            "ctx-case-snapshot",
            &snapshot_hash,
            "benefits-case@BEN-CP07-041-v1",
            world_state_revision,
            &[],
            "source/case-snapshot-BEN-CP07-041-v1.json#frozen-snapshot",
            &snapshot_provenance_hash,
            &workflow_hash,
        ),
        action_event(
            action_event_ids[0],
            2,
            action_ids[0],
            &["e-plan-original", "e-case-source"],
            world_state_revision,
            &workflow_hash,
            &snapshot_hash,
            &policy_hash,
        ),
        result_event(
            result_event_ids[0],
            3,
            result_ids[0],
            action_ids[0],
            &receipt_hashes[0],
            &["e-plan-original", "e-case-source"],
            world_state_revision,
            result_paths[0],
            result_revisions[0],
            &receipt_hashes[0],
        ),
        action_event(
            action_event_ids[1],
            4,
            action_ids[1],
            &["e-case-source"],
            world_state_revision,
            &workflow_hash,
            &snapshot_hash,
            &policy_hash,
        ),
        result_event(
            result_event_ids[1],
            5,
            result_ids[1],
            action_ids[1],
            &receipt_hashes[1],
            &["e-case-source"],
            world_state_revision,
            result_paths[1],
            result_revisions[1],
            &receipt_hashes[1],
        ),
        action_event(
            action_event_ids[2],
            6,
            action_ids[2],
            &["e-plan-original", "e-case-source"],
            world_state_revision,
            &workflow_hash,
            &snapshot_hash,
            &policy_hash,
        ),
        result_event(
            result_event_ids[2],
            7,
            result_ids[2],
            action_ids[2],
            &receipt_hashes[2],
            &["e-plan-original", "e-case-source"],
            world_state_revision,
            result_paths[2],
            result_revisions[2],
            &receipt_hashes[2],
        ),
        action_event(
            action_event_ids[3],
            8,
            action_ids[3],
            &["e-plan-original"],
            world_state_revision,
            &workflow_hash,
            &snapshot_hash,
            &policy_hash,
        ),
        result_event(
            result_event_ids[3],
            9,
            result_ids[3],
            action_ids[3],
            &receipt_hashes[3],
            &["e-plan-original"],
            world_state_revision,
            result_paths[3],
            result_revisions[3],
            &receipt_hashes[3],
        ),
        {
            let mut final_occurrence = message_event(
                "e-plan-final-reattachment",
                10,
                "ctx-plan-final-review",
                &policy_hash,
                source_revision,
                world_state_revision,
                &["e-plan-original", "e-case-source"],
                "source/northstar-benefit-plan-2026.1.md#document",
                &policy_hash,
                &workflow_hash,
            );
            final_occurrence["provenance"]
                .as_array_mut()
                .unwrap()
                .push(provenance(
                    "source/benefit-final-review-recipe-v1.json#controlling-plan-occurrence",
                    "benefit-final-review@1.0",
                    &recipe,
                ));
            final_occurrence
        },
    ];

    let relations = vec![
        relation(
            "produce-eligibility",
            "produces",
            action_ids[0],
            result_ids[0],
            "cp07/bodies/receipt-eligibility.txt",
            result_revisions[0],
            &receipt_hashes[0],
        ),
        relation(
            "produce-intake",
            "produces",
            action_ids[1],
            result_ids[1],
            "cp07/bodies/receipt-intake.txt",
            result_revisions[1],
            &receipt_hashes[1],
        ),
        relation(
            "produce-benefit-check",
            "produces",
            action_ids[2],
            result_ids[2],
            "cp07/bodies/receipt-benefit-check.txt",
            result_revisions[2],
            &receipt_hashes[2],
        ),
        relation(
            "produce-publication-check",
            "produces",
            action_ids[3],
            result_ids[3],
            "cp07/bodies/receipt-publication.txt",
            result_revisions[3],
            &receipt_hashes[3],
        ),
        relation(
            "same-state-plan-reattachment",
            "same_state_revision",
            "e-plan-original",
            "e-plan-final-reattachment",
            "cp07/source/northstar-benefit-plan-2026.1.md#document",
            source_revision,
            &policy_hash,
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
            "plan_original",
            policy_original_path,
            "context_attachment",
            Some("e-plan-original"),
            Some(source_revision),
            &policy,
        ),
        asset(
            "case_snapshot",
            snapshot_path,
            "context_attachment",
            Some("e-case-source"),
            Some("benefits-case@BEN-CP07-041-v1"),
            &snapshot,
        ),
        asset(
            "plan_final_packet",
            policy_repeat_path,
            "context_attachment",
            Some("e-plan-final-reattachment"),
            Some(source_revision),
            &policy,
        ),
    ];
    for index in 0..4 {
        assets.push(asset(
            [
                "receipt_eligibility",
                "receipt_intake",
                "receipt_benefit",
                "receipt_publication",
            ][index],
            result_paths[index],
            "environment_receipt",
            Some(result_event_ids[index]),
            Some(result_revisions[index]),
            result_bodies[index],
        ));
    }

    let slot1_messages = vec![
        message("system", vec![asset_part("system")]),
        message(
            "user",
            vec![
                asset_part("task"),
                asset_part("request1"),
                event_part("e-plan-original"),
                event_part("e-case-source"),
            ],
        ),
    ];
    let mut slot2_messages = slot1_messages.clone();
    slot2_messages.push(message(
        "assistant",
        vec![json!({ "kind": "prior_assistant_output", "request_slot": 1 })],
    ));
    slot2_messages.push(message(
        "user",
        vec![json!({ "kind": "prior_environment_receipt", "request_slot": 1 })],
    ));
    slot2_messages.push(message("user", vec![asset_part("request2")]));
    let mut slot3_messages = slot2_messages.clone();
    slot3_messages.push(message(
        "assistant",
        vec![json!({ "kind": "prior_assistant_output", "request_slot": 2 })],
    ));
    slot3_messages.push(message(
        "user",
        vec![json!({ "kind": "prior_environment_receipt", "request_slot": 2 })],
    ));
    slot3_messages.push(message("user", vec![asset_part("request3")]));
    slot3_messages.push(message(
        "user",
        vec![event_part("e-plan-final-reattachment")],
    ));

    let evaluation_key = json!({
        "expected_action_ids": [action_ids[0], action_ids[2]],
        "expected_states_after_action": ["coverage_eligibility_checked", "benefit_arithmetic_verified"],
        "expected_result_event_ids": [result_event_ids[0], result_event_ids[2]],
        "expected_final_answer": {
            "coverage_status": "COVERED",
            "governing_clause_ids": ["DEF-2.5", "ELIG-3.1", "COV-3.2", "COV-3.3", "NET-4.1", "CHARGE-4.2", "COST-6.1", "COST-6.2", "LIMIT-7.1", "LIMIT-7.2", "LIMIT-7.3", "EXCL-8.4", "ADJ-9.1", "ADJ-9.2", "ADJ-9.3", "ADJ-9.4"],
            "deductible": {
                "allowed_charge_cents": 420000,
                "remaining_before_cents": 50000,
                "applied_cents": 50000,
                "after_deductible_cents": 370000
            },
            "coinsurance": {
                "member_rate_basis_points": 2000,
                "member_share_cents": 74000,
                "plan_before_limit_cents": 296000
            },
            "benefit_limit": {
                "annual_cap_cents": 1000000,
                "paid_before_cents": 820000,
                "remaining_cents": 180000,
                "plan_payment_cents": 180000,
                "limit_reduction_cents": 116000
            },
            "exclusion": { "clause_id": "EXCL-8.4", "triggered": false },
            "member_responsibility_cents": 240000,
            "plan_benefit_cents": 180000,
            "final_disposition": "COVERED_WITH_CAP_LIMIT"
        },
        "required_event_ids": ["e-plan-original", "e-case-source", "e-eligibility-result", "e-benefit-result"],
        "required_relation_ids": ["produce-eligibility", "produce-benefit-check"],
        "critical_event_ids": ["e-plan-original", "e-case-source", "e-benefit-result"]
    });
    let evaluation_bytes = json_bytes(&evaluation_key);

    let expected_actions = [action_ids[0], action_ids[2]];
    let expected_results = [result_event_ids[0], result_event_ids[2]];
    let expected_result_ids = [result_ids[0], result_ids[2]];
    let expected_paths = [result_paths[0], result_paths[2]];
    let expected_receipt_hashes = [receipt_hashes[0].as_str(), receipt_hashes[2].as_str()];
    let expected_revisions = [result_revisions[0], result_revisions[2]];
    let expected_states = [
        "coverage_eligibility_checked",
        "benefit_arithmetic_verified",
    ];
    let advancing_points = (0..2)
        .map(|index| {
            let raw = format!("{{\"action_id\":\"{}\"}}", expected_actions[index]);
            json!({
                "request_slot": index + 1,
                "expected_action_id": expected_actions[index],
                "canonical_raw_utf8": raw,
                "canonical_raw_sha256": digest(raw.as_bytes()),
                "raw_language_cardinality": 1,
                "receipt_event_id": expected_results[index],
                "receipt_id": expected_result_ids[index],
                "receipt_path": expected_paths[index],
                "receipt_sha256": expected_receipt_hashes[index],
                "receipt_revision": expected_revisions[index],
                "state_after": expected_states[index]
            })
        })
        .collect::<Vec<_>>();
    let advancing_domain = json_bytes(&json!({
        "schema_id": "prefixity.phase1c.claim2-advancing-output-domain-v2-case",
        "schema_version": 2,
        "case_id": "CP07",
        "protocol_id": "prefixity.phase1c.claim2-canonical-advancing-output.v1",
        "offline_only": true,
        "points": advancing_points
    }));

    let action_menu = vec![
        json!({ "action_slot": 1, "action_id": action_ids[0], "action_event_id": action_event_ids[0], "result_event_id": result_event_ids[0], "result_asset_id": "receipt_eligibility", "state_after": expected_states[0] }),
        json!({ "action_slot": 1, "action_id": action_ids[1], "action_event_id": action_event_ids[1], "result_event_id": result_event_ids[1], "result_asset_id": "receipt_intake", "state_after": "intake_completeness_verified" }),
        json!({ "action_slot": 2, "action_id": action_ids[2], "action_event_id": action_event_ids[2], "result_event_id": result_event_ids[2], "result_asset_id": "receipt_benefit", "state_after": expected_states[1] }),
        json!({ "action_slot": 2, "action_id": action_ids[3], "action_event_id": action_event_ids[3], "result_event_id": result_event_ids[3], "result_asset_id": "receipt_publication", "state_after": "plan_publication_verified" }),
    ];
    let manifest = json!({
        "schema_id": "prefixity.phase1c.claim2-workload-case",
        "schema_version": 1,
        "case_id": "CP07",
        "kind": "positive",
        "planner_input": {
            "events": events,
            "relations": relations,
            "provenance": [
                provenance("cp07/workflow.md", "cp07-benefit-workflow-v1", &workflow),
                provenance("cp07/source/case-snapshot-BEN-CP07-041-v1.json", "benefits-case@BEN-CP07-041-v1", &snapshot),
                provenance("cp07/source/northstar-benefit-plan-2026.1.md", source_revision, &policy),
                provenance("cp07/source/benefit-final-review-recipe-v1.json", "benefit-final-review@1.0", &recipe)
            ]
        },
        "assets": assets,
        "action_menu": action_menu,
        "request_templates": [
            { "request_slot": 1, "messages": slot1_messages },
            { "request_slot": 2, "messages": slot2_messages },
            { "request_slot": 3, "messages": slot3_messages }
        ],
        "evaluation_key_path": "evaluation/key.json",
        "evaluation_key_sha256": digest(&evaluation_bytes),
        "assistant_output_planning_bytes": 4096,
        "token_proof_inputs": null
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
