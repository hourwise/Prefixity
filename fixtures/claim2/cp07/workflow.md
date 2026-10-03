# CP07 synthetic benefits adjudication workflow, version 1

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
