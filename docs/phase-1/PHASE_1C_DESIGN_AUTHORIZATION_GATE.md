# Phase 1C — Controlled Replay Design and Authorization Gate

## Status

**DESIGN GATE COMPLETE — EXECUTION NOT AUTHORIZED**

This document defines the proposed Phase 1C controlled replay, evaluation,
accounting, quality thresholds, abort/rollback rules, provider/model boundary,
and later-execution authorization requirements. It authorizes no provider,
model, API, replay, live prompt mutation, credential provisioning, paid
request, production change, or Phase 1D/later work.

The Phase 1B.9 `CONTROLLED_ONLY` policy remains research-only. It may be used
as a frozen research decision source in a separately authorized replay
experiment, but it is not promoted into `prefixity-core`, the production
planner, a request proxy, or a live prompt-mutating path.

## Authoritative starting checkpoint

- Commit: `10d5ad36bfbad7d9cac34eebe52c85315a138232`
- Commit message: `feat: add Phase 1B.9 blinded held-out study`
- CI: [run #33](https://github.com/hourwise/Prefixity/actions/runs/31369660499),
  completed successfully for that commit.
- Phase 1B.9 result: four true-positive selected interventions, zero false
  positives, zero false negatives, zero unsafe interventions, and deterministic
  report hash
  `be13da638022e6123c21ee9b009f9eaeb58a752e8547508b9f3900fc9d04c0a5`.
- Protected local state: `docs/tasks/ACTIVE.md` is unrelated Ananke work and
  is excluded from this design. Its pre-existing local modification must
  remain byte-for-byte unchanged and must never be staged with Phase 1C work.

Phase 1B.9 demonstrates a bounded held-out structural result. It does not
demonstrate provider behavior, model quality, end-to-end savings, live
rollback, or production readiness. Those are the questions this design makes
eligible for later review.

## 1. Research question and scope

The proposed replay asks:

> On a pre-registered, controlled task set with an independent evaluator, does
> a frozen Prefixity-selected intervention reduce full-trajectory context
> burden while preserving task success, required context, protocol validity,
> and dependency closure?

The first execution is a small controlled quality/efficiency study, not a
general agent benchmark, production deployment, prompt optimizer, cache
scheduler, or provider comparison. It must answer both sides of the question:

- whether any selected intervention produces an end-to-end efficiency win;
- whether the design correctly identifies no-op, harmful, inconclusive, and
  already-efficient cases.

The initial design excludes compression and learned/semantic rewriting. A
selected intervention can only be one of the frozen Phase 1B.9 research
classes (`PRUNE`, `DEFER`, or `RELOCATE_CANDIDATE`) and must have an immutable
source trace, opaque target, exact transformation manifest, and independent
quality evaluator.

## 2. Execution stages

The stages are sequential. Completion of one stage does not authorize the
next stage.

### Stage 0 — Offline certification (permitted now)

Before any provider call, the future runner must validate the frozen manifest
and execute the complete procedure against a deterministic mock transport or
scripted local world. This stage must prove:

- byte-identical baseline/no-op/intervention manifests from the frozen inputs;
- unchanged production planner and unchanged Phase 1B.9 policy bytes;
- exact target and transformation identity for every selected case;
- Tier 0 structural checks and evaluator wiring;
- accounting aggregation, artifact redaction, budget enforcement, abort, and
  fail-open behavior;
- zero network calls and zero credential reads during dry-run/mock execution.

Stage 0 is the only implementation/validation stage included in this design
gate. It does not create live artifacts or contact a provider.

### Stage 1 — Later-authorized provider schema smoke

If and only if separately authorized, run one explicitly named provider/model/
API-surface schema-smoke request using the existing allowlisted live harness.
This validates the response schema and accounting fields before any replay
cohort. A schema mismatch, unexpected field interpretation, credential
exposure, redirect, retry, or budget violation stops the provider path.

### Stage 2 — Later-authorized controlled replay

If Stage 1 passes and a second explicit execution authorization is granted,
run the frozen cohort using the three arms below. No task, prompt, threshold,
model setting, or evaluator may be changed after the first scored case.

The proposed initial cohort is 12–24 eligible controlled tasks, with a final
manifest hash frozen before scoring. It should contain positive candidates
across the available intervention classes, true no-op/ambiguous controls,
protected dependency/protocol controls, and at least one case expected to
expose negative or non-economic intervention behavior. The final count and
case identities are design inputs requiring review; this document does not
create or score them.

## 3. Replay arms and pairing

Every eligible task is evaluated as a paired unit:

| Arm | Input | Purpose |
| --- | --- | --- |
| Baseline | Original full context and original protocol | Establishes task success and full-trajectory accounting. |
| No-op control | Same original context, with a frozen `DO_NOTHING` decision | Measures replay/control variability and proves the comparison is not caused by arm wiring. |
| Intervention | Exact frozen Phase 1B.9 transformation applied to a disposable replay copy | Tests the selected intervention. |

The baseline and no-op arms must be byte-equivalent apart from run metadata.
The intervention arm may differ only in the exact blocks/order named by its
frozen transformation manifest. System instructions, tool contracts,
authentication material, evaluator instructions, and task identity are not
modifiable by the intervention.

Each arm uses a separate provider session or an explicitly documented cache
isolation protocol. Arm order is fixed in the manifest before execution. If
the provider cannot isolate or characterize cache state sufficiently for the
planned metric, provider-cache and cost conclusions are marked inconclusive;
the run must not silently attribute a cache effect to Prefixity.

The initial design does not require an external specialist baseline. Adding
one is a separate design change and requires a new authorization decision.

## 4. Task and evaluator contract

An eligible task must have all of the following before replay:

- public, synthetic, or otherwise explicitly approved provenance;
- sanitized inputs and no secrets or private raw data;
- an immutable task/trajectory identifier and source hash;
- a bounded tool/environment contract with no unrestricted filesystem or
  network side effects;
- a deterministic or independently versioned task evaluator;
- explicit required facts/state, protocol obligations, expected tool outcomes,
  and forbidden regressions where applicable;
- an evaluation key that is separate from the planner-facing trace and is not
  available to the policy.

The primary evaluator is Tier 0 plus Tier 2 deterministic checks:

- task completed;
- expected result/state obtained;
- required tests/checks and tool outcomes passed;
- required/gold context retained;
- dependency closure and protocol order valid;
- no forbidden state, security, or task regression;
- no unexpected tool call, extra recovery turn, or unreconciled reread.

Tier 3 model or human semantic evaluation is permitted only as a separately
versioned secondary evaluator when deterministic checks cannot decide the
task. It must use the same procedure for all arms, record evaluator version
and uncertainty, preserve blinded labels until scoring, and never override a
Tier 0 protocol/security failure. A judge-only pass cannot convert an unknown
critical preservation result into success.

For each baseline-pass case, the intervention is scored against the same
task-success predicate. A baseline failure, evaluator failure, missing
required signal, provider schema mismatch, or incomplete trajectory is
`INCONCLUSIVE`, not a pass and not evidence of intervention benefit.

## 5. Proposed frozen quality and efficiency thresholds

These are proposed thresholds for review and must be frozen in the execution
manifest before any replay result is inspected. They are design choices, not
measurements or claims about provider behavior.

### Hard safety gates

The following must hold for every scored intervention:

- **Tier 0 structural safety: 100%.** No required block removal, dependency
  break, illegal relocation, protocol violation, invalid request, or
  unverified critical preservation.
- **Baseline-pass → intervention-fail: 0 cases.** One such case aborts the
  cohort and fails the intervention gate; it is reported individually.
- **Critical task regressions: 0 cases.** Security, safety, required-state,
  forbidden-output, and protocol failures are not averaged away.
- **Required/gold-context retention: 100%** for every explicitly admitted
  required/gold item.
- **No unplanned behavior:** 0 retries, redirects, fallback providers,
  unplanned tool calls, or unapproved request-shape changes.

### Task-quality gate

For the initial binary deterministic cohort, every baseline-pass intervention
must also pass the same task predicate. Thus the primary paired quality
difference is required to be `0` regressions, not merely an acceptable average.

If a later approved cohort includes a graded secondary semantic score, the
predeclared exploratory envelope is:

- paired intervention score not more than 0.05 below the no-op mean;
- no individual case more than one rubric level below its no-op control;
- no critical evaluator item may regress;
- unresolved evaluator disagreement is inconclusive and cannot be counted as
  a pass.

The binary hard gates remain controlling even when a secondary score appears
favorable.

### Efficiency gate

An individual intervention is an end-to-end win only if all of these hold
against its paired no-op control:

- provider-native total input does not increase;
- provider-native fresh input decreases by at least **10%**, or the exact
  frozen provider pricing profile shows billed input cost decreases by at
  least **5%**;
- output tokens, rounds, tool calls, rereads/refetches, recovery turns, and
  physical request count do not increase;
- measured wall latency may not increase by more than **10%** unless the
  approved task manifest explicitly identifies latency as non-comparable;
- Prefixity planning/serialization overhead is recorded and does not erase
  the claimed benefit when total elapsed time is the declared endpoint.

The study-level positive gate requires at least one accepted intervention win
and reports every non-winning or negative-ROI intervention. A selected
intervention that does not meet the efficiency gate is not relabelled as a
success; it is evidence for rejection or `DO_NOTHING`. No threshold may be
relaxed after results are seen.

### Completeness and inconclusive gate

- The full planned cohort must be accounted for by `PASS`, `FAIL`, or
  `INCONCLUSIVE` per arm and per task.
- No positive claim is allowed if a required quality or accounting field is
  missing for the claimed case.
- A final cohort may not exceed **10% inconclusive eligible tasks**. Exceeding
  that limit pauses the study for redesign; it is not treated as a favorable
  denominator adjustment.
- Re-running a failed or inconclusive case solely to improve the report is
  prohibited. Any approved rerun is a new predeclared cohort or a documented
  recovery test with its own budget.

## 6. Abort and rollback rules

### Abort before the first provider call

Abort without a provider call if any preflight check fails:

- commit, design hash, task manifest, transformation manifest, evaluator
  version, or policy hash does not match the approved manifest;
- provider, model, API surface, endpoint, region, or settings are not the
  explicitly authorized values;
- any credential is missing, appears in an argument, log, artifact, or error;
- dry-run is not byte-identical to the approved request plan;
- request, token, time, or spend ceiling cannot be enforced locally;
- endpoint is not on the fixed allowlist or TLS/redirect policy is not
  enforced;
- required task/evaluator or cache-isolation metadata is absent;
- `docs/tasks/ACTIVE.md` is in the intended staged set or its protected hash
  differs from the checkpoint hash.

### Abort during replay

Stop the run immediately, with no automatic retry, if any of these occurs:

- one hard safety failure or baseline-pass → intervention-fail regression;
- unexpected provider/model response schema or malformed tool result;
- timeout, transport error, redirect, rate-limit handling that would require a
  retry, or provider refusal that changes the planned procedure;
- unplanned tool call, reread, recovery action, or request-shape mutation;
- a budget ceiling is reached or an accounting field becomes unavailable;
- evaluator disagreement affects a critical item;
- credential, private-data, or artifact-redaction violation;
- any result makes the approved arm order or task identity ambiguous.

The failure is recorded as evidence with the last completed request and
without exposing secrets or unrestricted response bodies.

### Rollback/fail-open behavior

No source trace, prompt, live request object, or production configuration is
mutated in place. Every transformation is applied to a disposable copy
created from the immutable baseline. If policy evaluation, transformation,
validation, or artifact writing fails before a request is sent, the original
request remains available and the case is aborted.

After a provider request has been sent, the runner does **not** issue an
automatic baseline fallback or retry. The original context is retained for
audit and for any future separately approved recovery run. A future live-agent
integration would need a separately authorized next-turn fail-open protocol;
that protocol is not part of this initial replay gate.

## 7. Provider/model-call boundary

No provider/model/API call is permitted under this design document. A later
execution authorization must identify all of the following:

- exactly one provider, model, API-surface schema, endpoint, and region/account
  scope for the run;
- the existing allowlisted adapter/harness path to be used;
- the credential environment-variable name and confirmation that the value is
  supplied outside source, arguments, logs, and artifacts;
- temperature, seed behavior, max output, tool-choice, system/developer
  instructions, timeout, and any provider cache-control settings;
- session/cache-isolation procedure and its limitations;
- exact task manifest, arm order, replicate count, max turns, max physical
  requests, estimated-input ceiling, output ceiling, wall-time ceiling, and
  hard spend ceiling;
- exact evaluator, pricing profile if cost is claimed, artifact destination,
  retention, and redaction procedure.

The execution path must preserve the existing Phase 0B boundaries: TLS
verification, no redirects, no automatic retries, sequential requests,
allowlisted provider URLs, environment-only credentials, bounded response
handling, and sanitized artifacts. No generic proxy or production request
interceptor is authorized.

Provider-specific claims are limited to the selected API surface and model.
There is no provider fallback, cross-provider aggregation, universal token
conversion, or pricing inference. OpenAI/Anthropic live behavior remains
unvalidated unless separately authorized and executed under their own schema
smoke and limits.

## 8. Accounting requirements

Accounting is per task, per arm, per request/turn, and at full-trajectory
aggregate level. The report must retain both raw bounded provider usage and
normalized fields with an explicit API-surface schema.

Required per-request fields:

- task/arm/replicate/request/turn identifiers and request order;
- Prefixity estimated input units, kept separate from provider units;
- provider-native total input, fresh input, cache-read, cache-write, and
  output units when the registered schema supplies them;
- tool calls, rounds, rereads/refetches, recovery turns, and physical request
  count;
- start, response-header, first-body/first-token when available, and total
  latency;
- timeout, retry, redirect, schema, and evaluator status;
- bounded provider request ID if safe to retain;
- cost only from the frozen exact provider/model/API pricing profile.

Required aggregate fields:

- full-trajectory totals and arm-to-no-op deltas;
- native-unit fresh/total/cache/output summaries, never cross-tokenizer
  subtraction;
- Prefixity planning and serialization overhead;
- latency distribution and worst case;
- extra calls, tool calls, rereads, refetches, output expansion, recovery, and
  failed/inconclusive work;
- cost in native currency and pricing-profile version, or an explicit
  `UNAVAILABLE` result when pricing is not frozen;
- negative-ROI and non-winning cases, not only favorable cases.

The primary efficiency report must distinguish structural potential from
realized provider cache reuse. Provider-reported cache units are not proof of
task quality, and Prefixity estimates are not provider billing units.

## 9. Determinism, provenance, and leakage controls

Before scoring, freeze and hash:

- this design document and its review version;
- source checkpoint and Phase 1B.9 policy/preregistration hashes;
- task/trajectory and evaluator manifests;
- baseline/no-op/intervention request manifests;
- provider/model/API settings and pricing profile;
- transformation outputs and arm order;
- report schema and redaction policy.

Planner-facing inputs contain no evaluation answers, oracle outcomes,
expected risk, purpose, gold labels, provider response, or semantic
answer-coded identifiers. Evaluation keys remain sidecar-only. The policy
decision is recorded before the evaluator is consulted. Any leakage invalidates
the cohort and requires a new frozen design.

The offline/mock runner must be byte-deterministic. Live provider responses
may be nondeterministic; the design therefore requires fixed settings,
predeclared replicate counts, complete raw bounded accounting, and paired
evaluation rather than claiming byte-identical model output.

## 10. Later execution authorization checklist

A future execution request must explicitly approve the specific frozen
manifest and include:

1. the design commit/document hash and confirmation that the thresholds are
   frozen;
2. the exact provider, model, API surface, endpoint, account/region, and
   credential environment boundary;
3. permission for provider/model calls and whether they may incur paid cost;
4. task manifest hash, arm order, replicate count, max turns, and evaluator
   version;
5. hard request, token, output, time, and spend ceilings;
6. cache/session isolation procedure and accepted limitations;
7. explicit approval of the artifact directory, retention, and redaction
   behavior;
8. confirmation that no production planner, live prompt path, or generic
   proxy may be changed;
9. a named abort owner and confirmation that no automatic retry/fallback is
   allowed;
10. the exact scope of the authorization and its expiration—no expansion to
    another provider, model, task cohort, rerun, or later phase is implied.

Approval to prepare a dry-run or mock test is not approval to make a provider
call. Approval for one schema-smoke is not approval for the replay cohort.
Approval for one cohort is not approval to rerun it after seeing outcomes.

## 11. Gate outcome and next step

The design gate is complete because the replay arms, evaluator, thresholds,
accounting, provider boundary, abort/rollback rules, and later authorization
requirements are explicit and reviewable. The gate does **not** claim Phase 1C
success, provider evidence, task-quality preservation, cost savings, or live
readiness.

One next task is recommended:

> Review and, if accepted, separately authorize Stage 0 offline/mock
> certification of this frozen Phase 1C design.

No provider call, live replay, credential provisioning, production planner
change, policy promotion, or later phase begins until a subsequent direct
authorization satisfies Section 10.
