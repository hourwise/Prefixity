# Phase 1C Capability-Evidence Design Addendum

## Status and relationship to the existing programme

**DESIGN ONLY — NO PROVIDER CALL, LIVE REPLAY, OR PROMPT MUTATION IS
AUTHORIZED**

This document is a capability-focused addendum to the existing Phase 1C
design and authorization gate. It does not replace or rewrite
[`PHASE_1C_DESIGN_AUTHORIZATION_GATE.md`](PHASE_1C_DESIGN_AUTHORIZATION_GATE.md),
the certified Stage 0 procedure, or the external-evidence gate. It makes the
capability question explicit, pins the currently accepted Prefixity candidate,
and defines the review inputs required before any later execution approval.

The canonical next capability-evidence phase is **Phase 1C — Quality-gated
controlled replay**. P0-L14 has no independent specification in this
repository and is not used as a new phase identifier. ContextBench is an
external evidence candidate, not an admitted local benchmark: its current
front-half decision is blocked by trajectory provenance/licensing and it must
not be downloaded, vendored, adapted, or replayed by this design task.

The existing Phase 1C design already supplies the primary three-arm structure,
quality thresholds, accounting contract, abort rules, and later authorization
boundary. The existing Stage 0 certification supplies the offline mock runner,
manifest checks, deterministic evaluator, leakage checks, redaction, budget
checks, and rollback/fail-open tests. This addendum reuses those contracts and
adds capability-specific interpretation; it does not create parallel runner
or schema infrastructure.

## 1. Existing-programme reassessment and conflicts

The relevant source-of-truth documents are:

| Existing contract | Role reused here |
| --- | --- |
| [`docs/phase-1/PHASE_1_PLAN.md`](PHASE_1_PLAN.md) | Defines Phase 1C as quality-gated controlled replay and keeps task quality primary over token deletion. |
| [`docs/phase-1/PHASE_1C_DESIGN_AUTHORIZATION_GATE.md`](PHASE_1C_DESIGN_AUTHORIZATION_GATE.md) | Defines baseline/no-op/intervention arms, evaluator tiers, thresholds, accounting, abort/rollback, and later Stage 1/Stage 2 authorization. |
| [`docs/phase-1/PHASE_1C_STAGE_0_CERTIFICATION.md`](PHASE_1C_STAGE_0_CERTIFICATION.md) | Certifies the 17-case synthetic/mock procedure only; it is not model-quality or provider evidence. |
| [`docs/phase-1/PHASE_1C_EXTERNAL_EVIDENCE_FRONT_HALF_GATE.md`](PHASE_1C_EXTERNAL_EVIDENCE_FRONT_HALF_GATE.md) | Blocks the provider path until external evidence admission, provenance, licensing, and a suitable comparator are resolved. |
| [`docs/phase-1/CONTEXTBENCH_FRONT_HALF_EXTERNAL_EVIDENCE.md`](CONTEXTBENCH_FRONT_HALF_EXTERNAL_EVIDENCE.md) and [`CONTEXTBENCH_EXTERNAL_TRAJECTORY_ADMISSION.md`](CONTEXTBENCH_EXTERNAL_TRAJECTORY_ADMISSION.md) | Establish ContextBench/Tracebench as restricted external inputs, not presently admitted replay material. |
| [`docs/SOURCE_OF_TRUTH.md`](../SOURCE_OF_TRUTH.md) and [`docs/tasks/ACTIVE.md`](../tasks/ACTIVE.md) | Record current project status, limitations, and the accepted P0-L6 closeout. |

The one documentation conflict is historical: `SOURCE_OF_TRUTH.md` still has
pre-Attempt-008 wording that describes P0-L6 as environment-blocked, while
the later accepted closeout at `748e4673e8454d2ac3e27cefabee9259992038aa` and
`ACTIVE.md` record Attempt 008 as complete with structural evidence only.
This addendum uses the sealed commit and current task record for the P0-L6
candidate boundary; it does not silently rewrite the historical source-of-
truth narrative. The Phase 1C Stage 1 external-evidence blocker remains
unchanged.

The following existing infrastructure is reused unchanged:

- Phase 1B.9 `CONTROLLED_ONLY` research decisions as frozen research inputs,
  never as production policy;
- Stage 0 `BASELINE`, `NO_OP`, and `INTERVENTION` manifest/arm construction;
- disposable-copy transformation validation and baseline/no-op equivalence;
- Tier 0 structural checks, Tier 2 deterministic task/tool checks, and the
  sidecar-only evaluation key;
- per-request native usage, latency, tool, round, reread, recovery, and
  physical-call accounting;
- deterministic redaction, abort, budget, retry, rollback, and fail-open
  contracts;
- the existing Phase 0/P0-L6 structural and cache diagnostics as secondary
  explanatory evidence only.

No ContextBench adapter, raw-data import, new evaluator implementation,
production planner change, or runtime integration is part of this design.

## 2. Research question

> Does the currently frozen Prefixity candidate preserve or improve useful
> model capability under constrained context relative to a matched baseline?

The primary endpoint is task capability under the same underlying task and
runtime conditions. Cache reuse, prompt-processing time, input-token count,
throughput, implementation correctness, and structural cleanliness are
secondary or procedural facts; none can substitute for a capability result.

The design asks whether the intervention can reduce or change the context
provided to the model while preserving the information and protocol needed to
complete the task. It does not ask whether fewer tokens are intrinsically
better.

## 3. Hypothesis

The preregistered directional hypothesis is:

> For tasks where the frozen Prefixity candidate removes or relocates context
> that is structurally non-load-bearing under its declared evidence, task
> capability will be preserved relative to a matched full-context control;
> some tasks may expose a capability regression, and no improvement will be
> claimed unless the predeclared task evaluator records one without any hard
> safety regression.

The null and valid outcome are both allowed: the candidate may preserve no
capability advantage, or `DO_NOTHING`/no-op behavior may be the correct result.
The hypothesis does not predict provider cache behavior, latency savings, or
generalization beyond the frozen cohort and selected runtime.

## 4. Frozen candidate definition

The capability experiment must use the Prefixity implementation at the sealed
commit:

| Identity | Frozen value |
| --- | --- |
| Source checkpoint | `748e4673e8454d2ac3e27cefabee9259992038aa` |
| Source commit message | `docs: seal P0-L6 controlled fresh-arm result` |
| P0-L6 semantic experiment | `2c80b9273970af54289acd9b7d8a4e0cffc3d7e56596d71247b27d56377388a9` |
| P0-L6 parent experiment | `730c9785aee03483ba8d169e68d8c41a4788abce6c08a3ec83de73017fa539bd` |
| Candidate/materialization identity | `e7fc579745aeabf52c215ec124df66d5c5c4cf97e2af8336d63fabb0f6ead97b` |
| Candidate pair identity | `fed1a73f84a4a06bc7d6516b5ea47ddadb91da07ca06894a4729e5d5aa36ff1d` |
| Safety certificate | `7375cebd045e529e879ab4dbe50379e4d6f024dc4ca9b018c3833e6e3889a55d` |
| P0-L6 runtime configuration fingerprint | `43038980202a0d6054df881cd76130a9ebe49c23ada628153f1b03547ef8dea5` |

The candidate is the already accepted Prefixity candidate only. No pruning,
KEEP/DEFER decision, relocation, compression, scoring, heuristic, token
accounting, context-selection, prompt-construction, caching, adapter, or
runtime behavior may be added or changed for this design. Structured state,
`SKILL.state`-style behavior, structured-state relocation, and interventions
discovered in later research are explicitly outside the candidate.

## 5. Control and treatment definitions

Each task is a paired unit with three fixed arms:

| Arm | Definition | Capability role |
| --- | --- | --- |
| `BASELINE` | Canonical unmodified full context and original task/protocol. | Establishes the matched task-capability reference. |
| `NO_OP` | Byte-equivalent context and protocol with a frozen `DO_NOTHING` decision, differing only in run metadata. | Detects arm wiring and replay variability; it is the primary efficiency comparator. |
| `INTERVENTION` | The same task on a disposable copy using only the frozen Prefixity candidate transformation named by the transformation manifest. | Measures capability under the candidate context. |

The `INTERVENTION` arm is the treatment. The evaluator, tool contract,
authentication material, system/developer instructions, task identity, and
expected-answer sidecar are identical across arms. Only the declared
candidate context change may differ.

Arm order is frozen before the first scored case. The proposed order is
`BASELINE`, `NO_OP`, `INTERVENTION` within each task/replicate, with a fresh or
explicitly isolated session per arm. An alternate order requires a new
manifest and review; it cannot be selected after observing outcomes.

## 6. Benchmark and task population

The initial population reuses the existing Phase 1C/Stage 0 task identities;
it does not create a larger benchmark merely for quantity.

### Proposed scored cohort

The proposed first scored cohort is the twelve eligible synthetic task cases
`h001`–`h012` from the existing Stage 0 construction. They cover the existing
Phase 1B.9 decision and evaluator paths, including selected intervention
cases, no-op/ambiguous cases, dependency/protocol-sensitive cases,
repetition/distractor conditions, and already-efficient or non-winning
behavior. Their exact payloads, source hashes, required state, tool contract,
and evaluator keys must be frozen in a later task manifest; Stage 0's mock
payloads are not themselves model-capability results.

The following existing Stage 0 identities remain procedure controls and are
not eligible capability successes or failures in the scored denominator:

- `h013`: evaluator-inconclusive path;
- `h014`: missing-accounting path;
- `s015`: hard structural-safety abort;
- `s016`: exact budget-boundary abort;
- `s017`: structurally valid but no-efficiency-win path.

They remain useful for offline runner certification and must not be silently
converted into model-quality observations.

### Capability coverage matrix

| Failure mode / capability claim | Required task coverage | Current design status |
| --- | --- | --- |
| Exact factual recovery from supplied context | Required facts and independently keyed expected state/result. | Covered in the task contract; exact case mapping is a manifest input. |
| Instruction retention | Required output/protocol obligations and forbidden regressions. | Covered by the Tier 0/Tier 2 evaluator contract. |
| Multi-step dependency retention | Explicit dependency closure and ordered tool/result obligations. | Covered where the admitted task supplies explicit dependency evidence. |
| Conflicting or competing context | Tasks with explicit conflict or ambiguity outcomes, not evaluator-inferred conflicts. | Partially covered by existing ambiguity/protocol paths; provenance must be explicit. |
| Distractor-heavy and repeated context | Repeated/irrelevant blocks with independently keyed required items. | Covered by existing repeated/distractor cases where the manifest retains the key. |
| Information early versus late | Paired position-sensitive cases with fixed task identity. | Structural position is measurable; capability effect must be scored, not inferred. |
| Context pressure near the usable limit | Bounded tasks whose admitted context approaches the selected runtime limit. | **Gap:** the current Stage 0 certificate does not establish this stratum. Add one reviewed synthetic/permission-cleared case or explicitly mark it out of scope before execution. |

The pressure gap is a design limitation, not permission to widen the cohort
ad hoc. Any added case must be included before manifest freeze, receive a new
cohort hash, and fit the same evaluator, privacy, and request ceilings.

## 7. Primary capability metrics

The primary evaluator is deterministic Tier 0 plus Tier 2, versioned and
independent of the planner-facing decision input. It records, per task, arm,
replicate, and turn:

1. `task_success`: whether the expected result/state was obtained;
2. required fact/state retention and required-context recall;
3. required tests/checks and declared tool outcomes;
4. dependency closure and protocol/order validity;
5. forbidden security, safety, state, or task regressions;
6. unexpected tool calls, extra recovery turns, and unreconciled rereads;
7. evaluator completeness and any explicit inconclusive reason.

The primary paired capability record must retain `PASS`, `FAIL`, or
`INCONCLUSIVE` per arm and task. A baseline failure, missing required signal,
provider schema mismatch, incomplete trajectory, or evaluator disagreement is
not evidence that the intervention improved capability.

The hard capability gates are:

- 100% retention of every explicitly admitted required/gold item;
- zero unexplained required-context false negatives;
- zero baseline-pass -> intervention-fail cases;
- zero critical task, security, safety, protocol, or dependency regressions;
- no intervention success credited when the corresponding baseline is
  invalid, incomplete, or inconclusive.

If a later separately approved cohort adds a graded semantic evaluator, it
must be independently versioned, blinded, and applied identically to all arms.
It may provide a secondary score but cannot override a Tier 0 failure or turn
an unknown critical preservation result into a pass.

## 8. Secondary structural and efficiency metrics

These metrics explain a capability result; they do not replace it:

- provider-native total input, fresh input, cache-read, cache-write, and output
  units, using only the selected API-surface schema;
- Prefixity-estimated input/context units kept separate from provider-native
  units;
- rounds, tool calls, rereads/refetches, recovery turns, and physical model
  requests;
- response-header, first-token/body, and total latency where available;
- Prefixity planning and serialization overhead;
- candidate structural diff, retained/removed/relocated items, and required
  context coverage;
- exact cost only if a frozen provider/model/API pricing profile is supplied;
  otherwise cost is `UNAVAILABLE`, never inferred.

An input-token reduction, cache-read increase, prompt-time decrease, or
throughput change cannot be labeled capability improvement. Provider cache
metrics remain distinct from structural reuse potential and from task quality.

## 9. Matched runtime variables

The later manifest must freeze these values before any scored request:

- provider, model/version, API surface/schema, endpoint, account/region, and
  credential environment-variable boundary;
- context limit, quantization/runtime profile where applicable, system and
  developer instructions, tool definitions, and protocol settings;
- temperature, seed behavior, top-p or equivalent sampling parameters, max
  output, tool choice, timeout, and provider cache-control settings;
- arm order, session/cache-isolation procedure, replicate count, max turns,
  maximum physical requests, input/output/time ceilings, and spend ceiling;
- task manifest, evaluator version, pricing profile if used, redaction
  version, artifact path/retention, and abort owner.

The Qwen/llama.cpp runtime used for P0-L6 is not automatically the runtime for
this capability study. Reusing it is a later design/authorization choice,
not an implicit carryover. All arms must use the same selected runtime and
settings. Fresh sessions or a documented isolation protocol are required;
where provider cache state cannot be isolated or characterized, cache and
cost conclusions are inconclusive. Nondeterministic model output must be
reported as such; fixed settings do not establish byte determinism.

## 10. Contamination and leakage controls

The following checks are required before and during a later run:

- evaluation keys, expected answers, gold labels, required-risk labels, and
  critical-failure labels stay in a sidecar unavailable to Prefixity's policy
  decision;
- planner-facing task IDs are opaque and do not encode arm, expected outcome,
  intervention class, or score;
- baseline and no-op serialized prompts are byte-equivalent apart from run
  metadata; intervention diffs are independently recomputed from the frozen
  transformation manifest;
- control, no-op, and treatment prompts use the same task input, tools,
  instructions, and evaluator procedure;
- task context is supplied only through the declared transformation; no
  expected answer, evaluator hint, outcome, or later-generated summary may be
  inserted into treatment metadata;
- each arm/replicate has a fresh or explicitly isolated session, and cache
  reuse/carryover limitations are recorded rather than assumed away;
- repeated requests, task labels, file names, request order, and response
  metadata do not reveal the expected result to the model;
- generated summaries, transformations, and tool outputs are checked for
  evaluator leakage before scoring;
- any leakage, unplanned prompt difference, or identity ambiguity invalidates
  the affected cohort under Section 13.

## 11. Evidence identity, lineage, and sealing contract

Before execution, the manifest and report must bind:

| Identity/evidence | Required treatment |
| --- | --- |
| Candidate source | Commit `748e4673e8454d2ac3e27cefabee9259992038aa`; working tree and implementation diff must be verified. |
| Candidate identity | P0-L6 candidate, pair, safety, semantic, parent, and runtime fingerprints in Section 4. |
| Existing policy inputs | Phase 1B.9 policy version `controlled-evidence-policy-v1`, policy hash `2139e084d97b16f3ae4ad36d95f0c73b4b1f448fe68f197139aa744dfe0e4`, and preregistration hash `e12846776660960093f9208b099ca171dc4b9c9583150b58de340e965409cd3b`. |
| Existing Stage 0 inputs | Stage 0 cohort hash `3e75e1e2dddb5456f69b8a6470650ff27e88c6b187edfc307eb181e8c5f8d776`, evaluator hash `059caacecf53ccf26ffe6489b4fe9b1247bcaca382107f05b23a34359ed7f229`, and report/design hashes from the certified Stage 0 record. |
| This design | SHA-256 of this exact document, frozen after review and recorded in the later manifest; the draft must not self-claim a hash before freeze. |
| Task population | Immutable task/trajectory IDs, source hashes, provenance/license/retention decision, task manifest hash, and exact cohort membership. |
| Arm/pair identity | Baseline, no-op, intervention, task, replicate, turn, session, and pair IDs; no arm identity may be inferred from output. |
| Request identity | Bounded request fingerprint, arm/order, provider request ID where safe, response schema, raw usage, and normalized result. |
| Evaluation identity | Evaluator version/hash, sidecar hash, per-case result, uncertainty/inconclusive reason, and blinded-scoring status. |
| Aggregate identity | Report schema/version, aggregation hash, all eligible/inconclusive/non-winning cases, and final claim permissions. |

Runtime evidence remains in an ignored local run directory under the existing
`experiments/runs/` retention policy unless a later approved policy explicitly
requires another destination. Raw prompts, unrestricted response bodies,
credentials, private data, and unlicensed external benchmark material must not
be retained in the repository.

## 12. Retry and ambiguity policy

The default is **zero automatic retries**.

If a request may have been consumed but its response is incomplete, timed out,
schema-invalid, or otherwise ambiguous:

- do not repeat it;
- preserve the bounded lifecycle and last durable evidence;
- mark the task/arm/cohort `INCONCLUSIVE` or `INVALID_EXPERIMENT` according to
  the predeclared failure type;
- do not infer success, failure, zero usage, or no capability effect;
- stop the affected execution boundary when identity, budget, or arm order
  can no longer be certified;
- permit recovery only as a newly named, predeclared, separately authorized
  cohort, never as an invisible continuation.

Attempt 007 remains the precedent for conservative accounting. No baseline
fallback, provider fallback, or adaptive extra replicate is allowed after a
possibly consumed request.

## 13. Invalidation criteria

The planned experiment is invalid or blocked, without a favorable
reinterpretation, if any of the following occurs:

- source commit, design hash, task manifest, transformation, evaluator,
  policy, report schema, pricing profile, or protected task-record hash does
  not match the frozen manifest;
- provider/model/API/endpoint/region/settings/session isolation differ from
  the authorization;
- credentials are absent, exposed, read outside the declared boundary, or
  written to logs/artifacts;
- baseline/no-op equivalence fails or intervention changes an undeclared field;
- expected answers, gold labels, evaluator results, task IDs, or treatment
  hints leak into planner inputs or prompts;
- required-context evidence, dependency/protocol evidence, usage fields, or
  evaluator signals are missing for a claimed result;
- timeout, schema failure, redirect, rate-limit, retry requirement, request
  ambiguity, or budget ceiling would require an unplanned recovery;
- any unplanned tool call, reread, refetch, request-shape mutation, model
  restart, or arm-order mutation occurs;
- a baseline-pass -> intervention-fail or critical safety regression occurs;
- the full planned cohort cannot be accounted for by `PASS`, `FAIL`, or
  `INCONCLUSIVE` without post-hoc denominator changes.

An invalid experiment produces an auditable invalidation record; it is not
converted into a capability regression or success claim.

## 14. Pre-registered capability result classes

The result class is selected mechanically from the frozen evaluator and
invalidation rules:

| Result class | Required evidence | Permitted interpretation |
| --- | --- | --- |
| `CAPABILITY_PRESERVED` | Complete eligible cohort; baseline-pass intervention cases all pass; required-context recall 100%; zero critical/protocol/dependency regressions; no invalidation. | Capability was preserved for this frozen cohort/runtime, subject to its limitations. It does not establish generalization or causality beyond the paired study. |
| `CAPABILITY_IMPROVED` | All preservation gates pass, plus a separately predeclared graded or binary improvement against matched no-op that is decidable for the same tasks, with no individual critical regression and no post-hoc evaluator choice. | Capability improvement may be reported only for this cohort/runtime and evaluator. Token/cache/latency reduction alone cannot satisfy this class. The current binary Stage 0 mock cannot establish it. |
| `CAPABILITY_DEGRADED` | Any baseline-pass -> intervention-fail, critical regression, required-context false negative, or hard task-quality regression with complete evidence. | Capability degradation is recorded and the intervention gate fails or returns to `KEEP`/`DO_NOTHING`; it is not averaged away. |
| `MIXED_INCONCLUSIVE` | Incomplete/ambiguous requests, evaluator disagreement, missing required fields, contrary non-critical outcomes, or unresolved cohort evidence without a hard invalidation. | Evidence is insufficient or mixed; no positive capability claim is permitted. |
| `INVALID_EXPERIMENT` | Contract, identity, leakage, safety, budget, provider-boundary, or arm-procedure violation. | No capability result is assigned; the cohort is invalid and requires a new authorization for any recovery. |

The study must report every task and arm, including non-winning,
`INCONCLUSIVE`, and invalidated cases. A capability-preservation result can
coexist with no efficiency win; efficiency and capability are separate
outcome axes.

## 15. Aggregation semantics

Aggregation is deterministic and occurs only after all required raw and
normalized records are durably persisted and their identities validate:

1. validate candidate, task, arm, evaluator, runtime, and policy hashes;
2. validate each task's baseline/no-op/intervention pairing and complete
   per-arm accounting;
3. compute per-task capability outcomes before cohort totals;
4. apply hard safety and preservation gates before any efficiency calculation;
5. compute paired no-op deltas for native input/fresh input/cache/output,
   latency, tools, rounds, rereads, recovery, physical calls, and overhead;
6. report capability class and efficiency class independently, retaining all
   negative, neutral, and inconclusive cases;
7. issue only claim permissions allowed by the result class and evidence
   source; never upgrade a structural or proxy result to capability evidence.

The existing Phase 1C efficiency gate remains the secondary accounting gate:
native total input must not increase; fresh input must decrease by at least
10% or the exact frozen pricing profile must show at least 5% billed-input
savings; output, rounds, tools, rereads, recovery, physical calls, and allowed
latency must not regress. These thresholds do not override the primary
capability gates.

## 16. Permitted claim classes

If the later evidence supports them, claims are limited to:

- structural facts about the frozen candidate and its declared transformation;
- task-evaluator outcomes for the named cohort, provider/model/API, and
  approved runtime settings;
- bounded capability preservation, improvement, degradation, or inconclusive
  status under the result table in Section 14;
- secondary native accounting and latency observations with their exact
  evidence source and comparability limits;
- a bounded efficiency result only when both capability and accounting gates
  are satisfied.

No result may be generalized to all models, providers, tasks, agents,
contexts, cache implementations, or production deployments without a new
study.

## 17. Explicitly prohibited claims and work

This design does not authorize:

- causal claims from P0-L6 structural/cache observations;
- capability claims from token reduction, cache reads, prompt time,
  throughput, or implementation correctness alone;
- provider superiority, universal savings, production safety, or general
  model quality;
- ContextBench/Tracebench admission, raw-data redistribution, or benchmark
  generalization before its external evidence gate passes;
- automatic live prompt mutation, a generic proxy, a daemon, or production
  planner integration;
- structured-state/SKILL.state work, structured-state relocation, compression,
  a new intervention, heuristic tuning, or implementation changes;
- a capability experiment, provider schema smoke, credential provisioning,
  paid request, or model/runtime start under this design commit;
- retries, adaptive replicates, task removal, denominator changes, or recovery
  after inspecting outcomes;
- starting P0-L14, Attempt 009, ContextBench replay, or another later phase.

## 18. Exact later execution plan

The following is a proposed sequence, not current authorization:

### Preparation and external admission

1. Review this addendum and the existing Phase 1C design.
2. Resolve the external evidence/provenance blocker or explicitly approve a
   fully synthetic/permission-cleared task population that does not claim
   ContextBench generalization.
3. Freeze the exact task manifest, candidate transformation manifest,
   evaluator/sidecar, runtime variables, replicate plan, budgets, session
   isolation, redaction, and report schema.
4. Compute and record all identities and hashes, including this design,
   candidate source `748e4673…`, task population, evaluator, and policy inputs.
5. Re-run only the applicable offline/mock certification against the frozen
   manifest if the manifest or evaluator changed; no provider contact is
   allowed.

### Stage 1 — separately authorized schema smoke

6. After the admission and manifest gates pass, obtain a separate direct
   authorization naming provider, model, API, endpoint, credential boundary,
   settings, and budget.
7. Issue exactly one schema-smoke request through the existing allowlisted
   harness. It is not a scored capability case. A mismatch, ambiguity,
   credential exposure, retry, redirect, or missing accounting stops the
   provider path with zero retry.

### Stage 2 — separately authorized capability replay

8. Obtain a second direct authorization for the exact frozen cohort and arm
   plan. No task, prompt, evaluator, threshold, runtime setting, or replicate
   may change after the first scored request.
9. For each proposed task `h001`–`h012`, run the fixed arm order
   `BASELINE` -> `NO_OP` -> `INTERVENTION` for two fixed replicates, with a
   maximum of three model turns per arm/replicate. Use fresh or explicitly
   isolated sessions and persist each arm before advancing its boundary.
10. Apply the treatment only from the disposable copy and only when the
    frozen transformation validator passes. Do not fallback or retry after a
    possibly consumed request.
11. Normalize and evaluate all cases with the frozen evaluator, preserving
    raw bounded usage and all inconclusive/non-winning records.
12. Run deterministic aggregation only after all identities and required
    records validate. Persist the report and seal hash, then stop.

## 19. Estimated maximum inference count

The proposed scored cohort has 12 tasks, 3 arms, 2 fixed replicates, and a
maximum of 3 model turns per arm/replicate:

`12 tasks × 3 arms × 2 replicates × 3 turns = 216 maximum model inference requests`

The separately authorized Stage 1 schema smoke adds at most **1** model
request, for a full programme ceiling of **217** model inference requests.
Offline preparation, Stage 0/mock certification, evaluator work, and
deterministic aggregation add **0** model inference requests. Tool calls are
accounted for separately and may not create unbudgeted provider/model calls.

The 216/217 values are planning ceilings, not permission to execute. A later
manifest may use a smaller cohort or lower ceiling, but it must state the
exact value before execution. No adaptive increase, extra replicate, retry,
or recovery is permitted.

## 20. Stop gates and next authorization boundary

| Gate | Must be true before proceeding | Stop outcome if false |
| --- | --- | --- |
| Design/preparation | Candidate source, design, task, evaluator, policy, transformation, runtime, budgets, leakage controls, and retention policy are frozen and hashed; no implementation change. | Do not request a provider call; revise/re-review the design. |
| External evidence | Provenance/licensing and task population are admitted or the cohort is explicitly synthetic/permission-cleared without an external-benchmark claim. | `STAGE_1_BLOCKED_FRONT_HALF_EXTERNAL_VALIDATION_REQUIRED`. |
| Stage 0/offline | Existing or updated mock certification passes with zero network/credential reads, baseline/no-op equivalence, transformation integrity, evaluator, budget, abort, redaction, and determinism checks. | No Stage 1 authorization request. |
| Stage 1 smoke | One separately authorized schema smoke completes with required usage/accounting and no retry/redirect/credential/identity issue. | Stop provider path; do not begin the cohort. |
| Control | Baseline and no-op arm records are complete, paired, equivalent, and durably persisted for the fixed task boundary. | Stop before treatment for the affected boundary; preserve evidence; no retry. |
| Treatment | The exact intervention record is complete, normalized, evaluator-key separated, and identity-matched to control/no-op. | Mark invalid/inconclusive as specified; do not aggregate. |
| Aggregation | All cohort cases are accounted for, hashes validate, no leakage or hard safety failure occurred, and capability outcomes are computed before secondary metrics. | Persist bounded invalid/mixed result; make no positive claim. |
| Closeout | Report and hashes are sealed, task record updated, runtime evidence retained under policy, and no later phase is implied. | Stop; any new run requires a new authorization. |

The next action is review and, if accepted, a separate Stage 1/Stage 2
authorization decision after the external-evidence and manifest gaps are
resolved. This document itself starts no live phase.
