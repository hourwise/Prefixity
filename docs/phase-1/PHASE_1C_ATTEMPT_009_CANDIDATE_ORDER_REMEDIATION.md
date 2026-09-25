# Phase 1C Attempt 009 candidate-order gate remediation

Status: `PHASE_1C_CANDIDATE_ORDER_GATE_REMEDIATION_ACCEPTED`

This bounded record covers only offline forensics and source remediation after
the consumed Attempt-009 live boundary. It does not authorize an Attempt-009
rerun, a replacement live run, or Attempt-010 preparation.

## Scope and accounting

The remediation entered from the clean promoted baseline:

```text
HEAD/main/origin/main = b0ab9e2e2ac0b9d992e6b2aca801aee16a122fe7
```

The required remediation accounting is:

```text
model_server_startups=0
port_8080_contacts=0
tcp_readiness_contacts=0
http_model_requests=0
inference_requests=0
attempt_009_executions_added=0
attempt_010_executions=0
```

The earlier Attempt-009 live accounting remains historical evidence in its
execution record. It is not included in the remediation accounting above.

## Forensic finding

Attempt-009 preparation accepted the tracked Attempt008-to-Attempt009 budget
transition from:

```text
fixtures/phase1c/attempt-009-budget-provenance.json
```

The preparation path also performed a local audit of ignored Attempt008 raw
evidence. The live child did not use either of those semantics. In
`validate_candidate_order(512)`, the runtime derived the prior registered
budget (1024) and constructed the generic prior-candidate path:

```text
experiments/runs/phase1c-reasoning-budget-calibration/budget-1024/candidate-result.json
```

The accepted Attempt008 candidate result, when present locally, is instead
under:

```text
experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-008/candidate-result.json
```

That raw evidence is ignored and is not a clean-checkout dependency. The
Attempt-009 child therefore failed before readiness with the recorded error:

```text
candidate order is not satisfied: prior candidate result is absent
```

This is a preparation/runtime semantic disagreement, not a model result. The
Attempt-009 execution remains consumed, integrity-rejected, and calibration-
inadmissible. **Attempt 009 is not evidence about reasoning budget 512.**
**Budget 512 remains untested.**

Provenance classification:

| Finding | Classification |
| --- | --- |
| Runtime looked for the generic prior-candidate `budget-1024/candidate-result.json` path | PROVEN — established by the source path construction and the recorded child error |
| The accepted Attempt008 result is represented by the tracked transition and an ignored raw evidence audit | PROVEN — recorded fixture and prior accepted evidence agree |
| Runtime candidate order did not consume the tracked transition validated by preparation | PROVEN — preparation and runtime called different validators |
| The original intended meaning of the generic candidate path | UNKNOWN — not inferred from the consumed run |

## Remediation

The existing tracked provenance fixture is the single authoritative transition
for the first live-required successor, so no duplicate source of truth was
introduced. The validator now:

1. requires the tracked Attempt008-admissible `FAIL` transition;
2. validates its schema, selected budget, source attempt, integrity and
   calibration-admissibility fields, including explicit Attempt007 exclusion;
3. uses that same loader from Attempt009 preparation and the runtime
   candidate-order gate; and
4. leaves the ignored Attempt008 and Attempt009 raw evidence as immutable,
   independent forensic material.

The new offline command is:

```text
attempt-009-candidate-order
```

It reports `RUNTIME_DEPENDENCIES_COMPLETE`,
`PREDECESSOR_TRANSITION_PRESENT`, `PREDECESSOR_TRANSITION_VALID`, and
`CANDIDATE_ORDER_VALID`, together with the explicit 512 transition fields:
`SOURCE_ATTEMPT_008`, `SOURCE_BUDGET_1024`, `SOURCE_STATE_FAIL`,
`NEXT_BUDGET_512`, and `CANDIDATE_BUDGET_512_ORDER_VALID`.

The command is offline-only and does not require a virgin Attempt009 evidence
root, ignored raw predecessor evidence, a listener, a model server, or an
inference request.

## State preservation and forward disposition

The accepted calibration state remains Attempt008, candidate budget 1024,
result `FAIL`, with `next_budget=512`. Attempt009 remains permanently
consumed, integrity-rejected, calibration-inadmissible, and non-reusable; its
zero-request pre-readiness failure is not converted into calibration evidence.

The protocol therefore supports the following next-fresh-attempt semantics,
without preparing that attempt here:

```text
NEXT_FRESH_ATTEMPT_ID=010
NEXT_FRESH_CANDIDATE_BUDGET=512
ATTEMPT_010_PREPARED=false
```

Attempt007's historical raw `next_budget=512` cannot satisfy the order gate.
It remains excluded because it was integrity-invalid, calibration-inadmissible,
and permanently consumed. Attempt008 wins the transition by being the accepted,
admissible `FAIL` predecessor.

## Offline validation evidence

Focused tests cover the valid tracked transition, missing predecessor,
Attempt007 exclusion, Attempt008 precedence, preparation/runtime parity,
clean-checkout operation, consumed Attempt009 regression, and CLI parsing.
The required repository checks are run before publication and recorded with
their exact results in the task closeout.

Review correction (follow-up to `644acc6`): the transition emitted by the
shared loader carried a truncated Attempt008 evidence-manifest hash (63 hex
characters). It now carries the canonical
`f20c4ce0149070e3ca1bc167f4400d71b88fe0bd7adac41851169ba8540e4779`, and a
regression binds both emitted hashes to the tracked Attempt009 identity
lineage. Further regressions reject a budget not selected by the admissible
predecessor (256, unregistered 768) and reject a corrupt or missing
transition field rather than reconstructing it. The forensic path above was
also corrected from `budget-512` to the prior-candidate `budget-1024` path
actually constructed by the original validator.

No Attempt009 raw evidence file, Attempt009 execution record, Attempt008 raw
evidence file, or historical calibration result was edited by this remediation.
