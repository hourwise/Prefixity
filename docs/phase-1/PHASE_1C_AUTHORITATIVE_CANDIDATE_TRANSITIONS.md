# Phase 1C authoritative candidate-transition generalization

Status: remediation of the recorded budget-256 candidate-order defect. No
attempt is prepared or executed by this change.

```text
LAST_ADMISSIBLE_ATTEMPT = 010  (budget 512, FAIL, integrity ACCEPTED)
NEXT_CANDIDATE_BUDGET   = 256
BUDGET_256_NOT_PREPARED
ATTEMPT_011_NOT_PREPARED
```

## Root cause

Before this change `validate_candidate_order_report` had two predecessor
paths:

- budget 512 special-cased a hard-coded Attempt-008 loader over
  `fixtures/phase1c/attempt-009-budget-provenance.json`;
- every other successor, i.e. budget 256, fell through to the generic
  ignored path
  `experiments/runs/phase1c-reasoning-budget-calibration/budget-512/candidate-result.json`.

The accepted Attempt-010 result lives under `budget-512-attempt-010/`, and in
a clean checkout no raw evidence exists at all. A 256 live child would
therefore have failed before readiness with
`candidate order is not satisfied: prior candidate result is absent` — the
same defect class that consumed Attempt 009. The generic path could also
have been satisfied by a stale file from a non-accepted run.

## Canonical transition model

One tracked registry, `fixtures/phase1c/calibration-candidate-transitions.json`,
records every authoritative predecessor transition:

```text
source 008: 1024 FAIL -> 512   identity 917fde56…dfb5  manifest f20c4ce0…4779
source 010:  512 FAIL -> 256   identity 9292e9ec…16f7  manifest 5673e55b…c4b9
excluded:   007, 009
```

Each transition binds source attempt, source budget, source state, integrity
and admissibility, case-set completeness, request/retry counts, the selected
next budget, the source identity path and canonical SHA-256, the evidence
manifest SHA-256, and the execution record path and normalized SHA-256.

`resolve_authoritative_candidate_transition(candidate_budget)` is the single
resolver. A candidate resolves only when:

1. the registry schema, experiment, and candidate order are exact, and the
   excluded list contains Attempts 007 and 009;
2. every transition is complete: an admissible, integrity-accepted,
   case-set-complete `FAIL`, no retries, complete 64-hex hashes, tracked
   `docs/` paths, and a source attempt that is not excluded;
3. each transition selects exactly the protocol successor
   (`next_candidate_budget(source_budget, FAIL)`);
4. source attempts, source budgets, and selected budgets are unique;
5. exactly one transition selects the candidate, and its source budget is
   the candidate's protocol predecessor;
6. the tracked source identity hashes to the recorded value (and matches its
   sidecar, attempt, and budget), the tracked execution record hashes to the
   recorded value, and that record states `ATTEMPT_NNN_INTEGRITY_ACCEPTED`,
   `ATTEMPT_NNN_CALIBRATION_ADMISSIBLE`, and the evidence-manifest hash.

`validate_candidate_order_report` uses the resolver for every non-initial
candidate; the special case and the generic `candidate-result.json` lookup
are removed. Ignored run evidence is never read. The initial candidate (1024)
has no predecessor transition.

## Parity

`calibration_prestart_checks`, which the live calibration path runs
immediately before its single listener check, calls
`validate_candidate_order_report`, as do all preparation paths. There is no
preparation-only or live-only predecessor lookup. A regression asserts that
for candidate 256 the resolver, the candidate-order report, and the runtime
prestart checks return the same transition.

## Consumed Attempt-010 identity

The Attempt-010 identity binds the implementation sources of its frozen
executables. This change edits the child source, so the Attempt-010
repository contract and live prerequisites now fail closed with
`… does not match current source`, which is the intended behaviour for a
consumed attempt. The identity document check was split from the
current-source binding so the recorded identity remains verifiable; every
preparation and live path still requires the binding. The integration test
formerly asserting the Attempt-010 contract was valid now asserts this
fail-closed state.

## Historical Attempt-008 validator defect

Commit `5918141` placed the Attempt-009 supervisor-command expectation inside
`validate_attempt_008_identity`. The generalized resolver does not call that
validator; it binds the Attempt-008 identity by canonical hash and sidecar.
Fixing it is not required for correctness here, so it remains deferred.

## Evidence preservation

No Attempt-008, Attempt-009, or Attempt-010 raw evidence, manifest, sidecar,
identity, or execution record was modified; the registry references their
accepted hashes. The historical `attempt-009-budget-provenance.json` fixture
is unchanged and still bound by the Attempt-009 and Attempt-010 identities.

## Next-attempt eligibility

The protocol state supports `NEXT_FRESH_ATTEMPT_ID = 011` at
`NEXT_CANDIDATE_BUDGET = 256`. No Attempt-011 identity, frozen object,
evidence root, or command was created.

```text
model_server_startups=0
port_8080_contacts=0
tcp_readiness_contacts=0
http_model_requests=0
inference_requests=0
attempt_010_executions_added=0
attempt_011_executions=0
```
