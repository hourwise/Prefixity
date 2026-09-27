# Phase 1C Claim-2 context-pressure workload design

Status: `CLAIM_2_WORKLOAD_DESIGN_READY` — design only

```text
LOCAL_9B_INSTRUMENT_ACCEPTED
LOCAL_9B_FEASIBILITY_RESULT_ACCEPTED
LOCAL_9B_FEASIBILITY_PASSED
EXISTING_DEPENDENCY_SAFETY_BATTERY_RETAINED
EXISTING_CLAIM_2_PILOT_WORKLOAD_REJECTED
PILOT_CONTEXT_INADEQUATE
CLAIM_2_CONTEXT_PRESSURE_WORKLOAD_DESIGN_ONLY
CLAIM_2_CONTEXT_PRESSURE_WORKLOAD_DESIGN_COMPLETE
CLAIM_2_WORKLOAD_DESIGN_READY
CLAIM_2_WORKLOAD_MATERIALIZATION_PREPARATION_NEXT
NO_WORKLOAD_FIXTURES_OR_SCORED_PILOT_PREPARED
NO_MODEL_STARTUP_TOKENIZATION_OR_INFERENCE_PERFORMED
```

This is the offline design resulting from the pilot-context adequacy review.
“Ready” means ready for the next offline task, `CLAIM_2_WORKLOAD_MATERIALIZATION_PREPARATION`.
It does not mean that fixtures exist, exact model token counts are known, the
longer tasks have passed a BASELINE competence gate, or a scored pilot is
prepared or authorized. A later `NON_INFERENCE_CONTEXT_TOKENIZATION_PASS`
requires separate authorization after request bodies and renderers exist.

## Research question and inherited evidence

Claim 2 asks whether, on a sufficiently competent model, Prefixity reduces
carried input context without materially reducing task success. This design
addresses context reduction and preservation together: a large candidate
deletion is useful only when the task succeeds, load-bearing dependencies
remain available, and control cases are preserved.

The accepted measuring instrument is Qwen3.5-9B Q4_K_M, reasoning off,
context 8192, `max_tokens` 1024, temperature 0, `top_p` 1, seed 1, and
`stream = false`. The frozen request omits `chat_template_kwargs`, uses
text-only content, and has no `mmproj`. The accepted artifact is
`lmstudio-community/Qwen3.5-9B-GGUF` at revision
`1379f25` (base `Qwen/Qwen3.5-9B`, Apache-2.0), stored at
`D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf`, file
`Qwen3.5-9B-Q4_K_M.gguf`, SHA-256
`cd76ec205963b3b33350093e6904d9de16c4e666fd104e1f632d25c7f15f2a13`,
5,627,044,256 bytes, file identity
`volume=ba2f80f4;index=00050000004555a6`. The accepted llama.cpp executable is
the b10217 build, 15,277,056 bytes, SHA-256
`cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617`, file
identity `volume=c4c93b54;index=00060000001ea970`. The model is not loaded by
this design task.

The sealed local-9B feasibility gate passed three structural probes and one
h001 BASELINE request. h001 measured 391 input tokens and passed its frozen
task, required-context, dependency, and critical-regression checks. This
supports the instrument decision for those probes only. It is not evidence of
competence on the longer coding, repository, metadata, or branch tasks proposed
here. The prior six cases and their identities are unchanged; the adequacy
review retains them as the `DEPENDENCY_SAFETY_BATTERY` and rejects them as the
primary Claim-2 efficacy cohort.

## Retained dependency-safety battery

Retain the historical cases exactly as frozen: h001 (`PRUNE`), h004
(`PRUNE`), h007 (`DEFER`), h009 (`RELOCATE_CANDIDATE`), h006
(`DO_NOTHING`), and h010 (`DO_NOTHING`). Their order and historical evidence
remain unchanged. Five are structural traces; h001 is the only currently
runnable model-visible task. Materializing the five missing model-visible
tasks is separate work. The battery continues to exercise dependency and
policy safety, including DEFER, relocation, and no-op outcomes. It is not
added to the efficacy denominator and supplies no context-pressure result.
The current h001 runner is one inference request per arm. Its
`MAX_TURNS = 3` ceiling and the old 54-request arithmetic were not an
implemented multi-turn loop. The proposed cohort's 54-request maximum is
instead derived from six actual three-request case-arms per each of three
arms; until implemented and executed, it remains a design ceiling.

## Policy boundary and planner decision

There are two relevant decision paths, and this design selects only the
controlled Phase 1B.9 research policy for the new stratum.

The production Phase 1B.0 contract in
[`decision.rs`](../../crates/prefixity-core/src/decision.rs) contains
`KEEP`, `PRUNE`, `DEFER`, `RELOCATE_CANDIDATE`, `DO_NOTHING`, and the reserved,
never-emitted `COMPRESS_CANDIDATE`. It may produce multiple hypothetical
per-block recommendations, including `PRUNE` or `DEFER`; its dependency and
protocol checks are conservative. It is not the policy being evaluated here.
Its structural planner, token size, repetition, or non-gold status alone do
not prove an intervention is safe. Chronological messages and protocol-critical
items are protected.

The selected controlled path is the blinded `controlled-evidence-policy-v1`
in [`phase1b9.rs`](../../crates/prefixity-controlled-benchmark/src/phase1b9.rs),
scope `CONTROLLED_ONLY`. Its frozen rule order is:

1. `EXACT_DUPLICATE_PRUNE`;
2. `EXPLICIT_SUPERSESSION_DEFER`;
3. `SAME_ZONE_PROTOCOL_RELOCATE`;
4. `FAIL_OPEN` to `DO_NOTHING`.

For this stratum, only the first rule supplies a scored positive omission.
DEFER preserves its item in the trace for a later position and relocation
removes zero context tokens, so neither action is part of this context-
reduction cohort. Their safety role remains in the separate battery. The
exact-duplicate predicate selects a later `EventType::Message` with the
same content hash as an earlier occurrence and an explicit
`SameStateRevision` relation. It excludes targets with consumers or
protected relations, and the frozen selection is by the existing sequence
rule, not by size or expected outcome. The original occurrence and its
provenance remain. The selected operation removes one duplicate event and
associated references; it does not remove a chronological tool/result event.
The `apply_decision` path and dependency checks are part of the intervention
identity.

CP01–CP04 therefore target only a duplicate context attachment that a
documented native export or reattachment workflow naturally created. The
attachment must be represented as a non-protocol context `Message` from its
origin, not as a tool result, assistant response, producer/result ID, or a
chronological message relabelled to qualify for pruning. Tool actions,
result envelopes, receipts, protocol links, message order, and their hashes
or locators remain intact in every arm. The duplicate body must be byte-equal
to its same-state original, and the duplicate occurrence must have no
consumer anywhere in the preregistered three-step dependency graph. The
original can remain load-bearing or be referenced by the final task.

CP05 and CP06 must yield `DO_NOTHING` from the actual frozen research policy.
Their candidate-looking items respectively lack same-state identity or have
an explicit consumer/protected dependency. They must contain no other
eligible duplicate, supersession, or relocation candidate. If the frozen
policy selects a different target or class, that case fails offline
admission; the target must not be hand-picked to match this design.

The current h001 runner consumes frozen arm projections; it does not invoke
the research policy at runtime. A later materialization adapter must reproduce
the complete eligibility and dependency-topology checks, run the exact
existing research selection on a blinded trace, freeze the decision before
any model outcome is observed, then apply it to the identity-matched planner
input and prove parity with those functions. Their current private visibility
may require a narrow exposure change in that later implementation. Do not
re-rank by bytes saved, alter rule order/cardinality, or reinterpret event
semantics. If the native
attachment form cannot pass the existing predicate, or if semantic grouping,
new DEFER behaviour, or a policy change is needed, stop and return for a
separate decision marked `MECHANISM_CHANGE_REQUIRED`.

This design uses one naturally occurring eligible target at the single
pre-request-3 checkpoint in each positive case. It does not require multiple
simultaneous targets, grouped targets, or planner changes. It therefore
preserves the frozen one-target Phase 1B.9 decision behavior for this bounded
cohort. Additional reductions at future checkpoints are untested and would
require a separate design. This is not a general production context-pruning
claim.

For clarity, the options are: A, multiple eligible removable items but one
registered policy-selected target (a possible later design, never a size-ranked
or grouped choice); B, aggregate several items into one target (a mechanism
change); C, extend the planner or its rule behavior (a mechanism change); or D,
one naturally large eligible target per evaluated checkpoint, selected by the
unchanged rule. This design chooses D. If the frozen rule does not select that
target, do not substitute A, B, C, or a hand-selected candidate.

## Scenario families considered

The scenario search considered the six requested families. CP01–CP04 select
coding/debugging, repository investigation, tool-metadata reconciliation, and
branch/backtrack. The exact-output family supplies both a positive candidate
pattern and a matched control. Broad planning and general-purpose research
deferral are not used to inflate the first cohort.

| Family | Natural history and context decision | Main dependency risk and deterministic evaluation | Disposition |
| --- | --- | --- | --- |
| Coding/debugging with repeated reads and edits | Initial source/config inspection, a diagnostic or test run, then a later export that reattaches an identical same-revision context attachment. Prune only the unconsumed duplicate attachment. | Preserve task constraints, original source, current test evidence, and the failed diagnostic facts. A pinned local repository and exact answer/test oracle can score the result. | Selected as CP01. |
| Repository investigation with stale searches | Broad symbol inventory, narrowed call-site inspection, then a normal investigation packet reattaches the unchanged inventory beside the new call-site evidence. Prune only the same-state duplicate attachment. | A broad search may contain the only relevant call site; preserve unique search facts and direct reads. A pinned repository and expected path/symbol set allow exact scoring. | Selected as CP02. |
| Multi-step planning with superseded state | A plan changes after an assumption is disproved. Old plan text may still be the sole record of a user constraint. | A checklist can score explicit invariants, but open-ended plan quality is less deterministic. The research policy's explicit-supersession DEFER retains/repositions material and does not remove input context. | Considered; no primary efficacy case. |
| Research/evidence with material needed later | Source records accumulate before a later question needs a subset. A fixed source set could support exact facts and citation checks. | Removing a source can hide contradictions or require a costly reread. The available DEFER action retains the full trace and is not context reduction for this endpoint. | Considered; general evidence deferral is deferred to later design. |
| Repeated identical tool output | A read-only workflow may emit the same inventory/result again. Exact same-state identity makes a duplicate attachment candidate; identical bytes from different states do not. | Output hash alone is insufficient; preserve command, state/revision, and occurrence identities. Frozen command outputs and state IDs make the pair deterministically testable. | Same-state form eligible in principle; different-state form is control CP05. |
| Branch/backtrack leaving old branch state | A bounded diagnostic fails, the environment restores a known source revision, and the normal workflow reattaches that original context alongside branch outcome evidence. | Failed-branch diagnostics may explain why an approach is rejected. Keep them; target only the repeated source attachment, with exact source/branch IDs and final checks. | Selected as CP04. |

Only naturally produced history is admissible. A case that needs padding,
truncation, artificial repeated messages, aggregation of distinct outputs,
or forced reattachment solely to meet a size target is rejected. If normal
workflows fail to generate a qualifying duplicate, revise the offline design
before observing model outcomes; do not substitute a post-hoc case.

## Proposed context-pressure cohort

The proposed cohort has six cases in this fixed order: four positives and two
dependency controls. Each positive has one naturally generated,
policy-eligible duplicate attachment. Each control has substantial history
and a candidate-looking item that the policy must preserve. The roster is a
design proposal, not a frozen fixture set.

### CP01 — coding/configuration diagnosis (positive)

**Turn 1:** Present task constraints and a bounded action menu. The model
selects source/config inspection. The deterministic local environment returns
the real source evidence and a complete same-revision context attachment.

**Turn 2:** The model selects a diagnostic or test check. The environment
returns the result and updates the task state. The diagnostic export carries
forward evidence needed for the next decision.

**Turn 3:** The current request asks for the constrained resolution or exact
diagnosis. The normal export reattaches the identical source/config attachment
beside new test evidence. Prefixity acts immediately before this request.

- Required/current: task constraints, original source, current state, latest
  diagnostic/test result, and the facts required by the final answer.
- Repeated: one complete non-protocol context attachment with the same source
  hash and revision as its earlier occurrence.
- Stale/superseded/deferred: none is a candidate; unique diagnostic evidence
  remains visible, and no later-only material is introduced.
- Governance/evidence: source revision, attachment event IDs, diagnostic run
  ID, provenance locator, and an evaluation-only expected-answer record.
- Candidate removal: only the later duplicate attachment. Expected action:
  `PRUNE`, conditional on exact policy selection, its complete preregistered
  dependency closure, and zero consumers or protected relations on the
  selected duplicate.
- Load-bearing dependencies: original source and current diagnostic; the
  duplicate occurrence itself must have no consumer.
- Deterministic task success: exact required diagnosis/fix facts plus the
  frozen local test and source-state checks.

### CP02 — repository investigation (positive)

**Turn 1:** The model chooses a broad symbol inventory. A deterministic,
pinned repository search returns its result and the normal workflow attaches
the inventory as context.

**Turn 2:** The model chooses narrowed call-site inspection. The environment
returns directly opened source and call-site evidence, preserving any unique
facts from the broad search.

**Turn 3:** The final request asks for exact paths and symbol relationships.
The investigation packet naturally carries the same inventory attachment
again beside the new call-site evidence. Prefixity acts before this request.

- Required/current: task scope, direct reads, selected call-site evidence,
  unique search facts, and the final path/symbol mapping.
- Repeated: one identical inventory attachment linked to the same repository
  revision and search identity.
- Stale/superseded/deferred: no unique search result is labeled removable;
  broad output that was not confirmed by direct reads remains required.
- Governance/evidence: repository commit, query identity, search result hash,
  opened-file locators, occurrence IDs, and hidden expected path/symbol set.
- Candidate removal: the later same-state duplicate inventory attachment
  only. Expected action: `PRUNE` if the frozen policy selects it.
- Load-bearing dependencies: direct reads and any unique broad-search fact;
  the earlier inventory remains available.
- Deterministic task success: exact expected paths, symbols, and relationships
  with repository-state validation.

### CP03 — tool metadata reconciliation (positive)

**Turn 1:** The model requests a full dependency/build inventory from a pinned
local project state. The environment returns the complete inventory as a
context attachment.

**Turn 2:** The model requests a verification against the unchanged snapshot.
The environment returns a small current verification record.

**Turn 3:** A standard reconciliation workflow re-emits the same full
inventory attachment with that verification record. The final task asks for
exact version, compatibility, and reconciliation facts.

- Required/current: the original inventory, verification output, task
  constraints, and all facts used in the final reconciliation.
- Repeated: same inventory bytes, inventory hash, and unchanged snapshot
  identity.
- Stale/superseded/deferred: none is a target; metadata that differs across
  snapshots or verification runs stays visible.
- Governance/evidence: project snapshot, inventory producer ID, verification
  run ID, both occurrence IDs, and an evaluation-only answer key.
- Candidate removal: only the duplicate attachment occurrence. Expected
  action: `PRUNE` if it has no consumer or protected relation.
- Load-bearing dependencies: the original inventory and the distinct
  verification result.
- Deterministic task success: exact dependency versions, compatibility facts,
  and reconciliation result checked against the frozen project state.

### CP04 — branch/backtrack (positive)

**Turn 1:** Present a task and immutable starting source attachment. The model
selects a bounded diagnostic branch.

**Turn 2:** The environment records the failed candidate and restores the
known original revision. The model selects current-state verification; the
environment returns state evidence and preserves the failed-branch record.

**Turn 3:** The normal recovery workflow reattaches the identical original
source revision beside branch outcomes. The final request asks for a valid
configuration/branch choice and its invariant. Prefixity acts before this
request.

- Required/current: task constraints, original source and revision, failed
  branch outcome, current restored state, and final invariant.
- Repeated: the original attachment is repeated byte-for-byte with the same
  immutable revision identity.
- Stale/superseded/deferred: failed diff and branch outcome are stale relative
  to the current working tree, but they are not targets; keep their reason
  and provenance. No material is deferred.
- Governance/evidence: source/revision IDs, branch IDs, restore operation,
  diagnostic run IDs, and hidden expected branch/invariant record.
- Candidate removal: the later original-source attachment only. Expected
  action: `PRUNE` if the exact policy selects that occurrence.
- Load-bearing dependencies: the original source, current state, and failed
  branch evidence needed to avoid repeating the rejected option.
- Deterministic task success: expected branch/config state, required invariant,
  and frozen local checks.

### CP05 — same bytes, different states (control)

**Turn 1:** Present the task and an identified source revision. The model
receives request 1 and chooses inspection of that source;
the deterministic environment returns the initial attachment, and the frozen
intermediate oracle checks the selected action and returned occurrence ID.
**Turn 2:** The
model receives request 2 and chooses state-comparison verification; a
deterministic state change then causes the environment to return a second
attachment with exactly the same bytes but a different revision/state ID.
The intermediate oracle checks the verification action and the state-change
evidence.
**Turn 3:** The model receives request 3 to compare the two states and cite the
correct occurrence IDs.

- Required/current: task, both states and occurrence IDs, state-change
  evidence, and the comparison rule.
- Repeated: body bytes are equal; semantic revision/state identities differ.
- Stale/superseded/deferred: neither occurrence is declared stale, superseded,
  or deferred for this task.
- Governance/evidence: both revision IDs, operation identity, provenance,
  and evaluation-only expected comparison.
- Candidate removal: none. There is no `SameStateRevision` relation; expected
  actual policy result is `DO_NOTHING`.
- Load-bearing dependencies: both state identities and their relation to the
  intervening operation.
- Deterministic task success: correct comparison and both occurrence IDs.

### CP06 — same-state duplicate with a consumer (control)

**Turn 1:** Present the task and an explicit evidence/receipt locator.
The model receives request 1 and chooses inspection or receipt retrieval.
The deterministic environment returns the identified record or receipt; the
intermediate oracle checks the action and exact occurrence/receipt identity.
**Turn 2:** The model receives request 2 and chooses state-comparison or audit
receipt verification. The deterministic environment reattaches the same-state
record with its explicit consumer or protected protocol relation to the final
audit step; the intermediate oracle checks the verification action and
dependency edge. **Turn 3:** The model must complete that audit and cite the
correct attachment and receipt. All three are actual model requests.

- Required/current: task, original and repeated occurrences, receipt, consumer
  edge, and audit requirements.
- Repeated: same-state body and revision are identical, so this looks like a
  duplicate by content alone.
- Stale/superseded/deferred: neither occurrence is stale or superseded; none
  is deferred.
- Governance/evidence: event IDs, same-state relation, protected consumer or
  protocol edge, receipt ID, and hidden audit oracle.
- Candidate removal: none; the later occurrence has a consumer/protected
  relation. Expected actual policy result is `DO_NOTHING`.
- Load-bearing dependencies: both occurrences and their exact receipt/audit
  links.
- Deterministic task success: valid receipt linkage and correct occurrence
  IDs, not merely the repeated fact text.

Both controls must be similar in prompt size and apparent duplicate size to
the positives. A candidate-looking attachment around 1.0–1.4k estimated
tokens is a planning range only; controls have zero legal removable tokens.
If the actual frozen policy does not return `DO_NOTHING` after all eligibility
paths are checked, the case is not admitted.

## Actual trajectory and arm protocol

Each case-arm consists of three genuine model requests with the conversation
and state carried forward. The action emitted at each request comes from a
finite menu frozen before the trial. A deterministic local environment
executes a valid action and appends its actual result, next context attachment,
and provenance. An environment step is not counted as an inference request.
Model output is never scripted, copied from another arm, or treated as an
environment response. There is no fourth request, hidden retrieval turn, or
open-ended loop. An invalid, malformed, unknown, or out-of-menu action ends
that arm without repair or retry. An invalid, malformed, unknown, or
out-of-menu response, or a response the frozen evaluator establishes is
incorrect, is an observed task/structural `FAIL`, even if later requests are
not run. Record each skipped slot as
`NOT_EXECUTED_AFTER_FAILURE`; retain the arm and every planned request slot in
the fixed denominator. Transport, integrity, or context-bound errors, and
pre-intervention raw-output divergence that breaks the equality gate, are
`INCONCLUSIVE`. Report arm completeness separately from adjudicated model
failure. No incomplete arm can pass the overall gate.

The sequence is request 1 inspection, request 2 verification, and request 3
the final task over accumulated history. Each request receives that arm's own
prior assistant output and deterministic environment results. Prefixity acts
only before request 3 in an intervention arm, removing its one authorized
duplicate attachment from the model-visible view. Raw histories and removed
content remain archived with provenance outside the prompt and are not
re-injected. Thus this design tests one reduction against a naturally
accumulated three-request history; it does not claim a months-long agent
trajectory or repeated pruning at later checkpoints.

The comparison uses one replicate, in case-major order:

```text
CP01: BASELINE -> NO_OP -> INTERVENTION
CP02: BASELINE -> NO_OP -> INTERVENTION
CP03: BASELINE -> NO_OP -> INTERVENTION
CP04: BASELINE -> NO_OP -> INTERVENTION
CP05: BASELINE -> NO_OP -> INTERVENTION
CP06: BASELINE -> NO_OP -> INTERVENTION
```

Maximum work is 18 case-arm trajectories, 54 inference requests, 18 fresh
server starts, one replicate, no warmups, and no retries or adaptive
replicates. Each arm starts on a fresh server under the same frozen runtime
identity and conditions; cache state is carried across that arm's three
requests only. No cache is shared across cases or arms. Starting a new server
for every request would not erase a transcript that the renderer explicitly
resends, but it would change the cache/runtime condition and raise the count to
54 server starts. It is therefore not the design. Do not use a whole-cohort
server that warms one case or arm for another.

BASELINE sees the complete natural transcript. NO_OP passes through the same
Prefixity selection, projection, application, serialization, token-count,
runtime, and clock boundaries but receives a frozen, explicitly forced
`DO_NOTHING` procedural decision. This forcing is the NO_OP definition; do
not report that the model or selection policy independently chose no action.
At every corresponding request, BASELINE and NO_OP must have byte-identical
model-visible messages, order, wrappers, and context. INTERVENTION uses the
same path and differs only by omission of the frozen eligible attachment at
request 3; requests 1 and 2 must match BASELINE and NO_OP exactly. Count
pipeline overhead in all arms.

Run arms independently. If an arm's assistant output diverges before
intervention, its later carried prompt may diverge too; record a procedural
mismatch/inconclusive result. If BASELINE and NO_OP diverge under otherwise
identical inputs, the pair is also a mismatch. Do not normalize, copy, or
replay outputs to manufacture equality, and do not retry. After treatment,
INTERVENTION need not match BASELINE text, but it must pass the same task and
dependency checks. Single-replicate temperature-zero execution is a bounded
falsification design; it is not evidence of perfect model determinism,
non-inferiority, statistical power, or population generalization.

## Estimated context sizes and materiality rule

All sizes below are design estimates, not measured tokenizer counts. For
mixed code, logs, and structured text, the rough conversion is 2.5–4 UTF-8
bytes per token; decimal kB means 1,000 bytes. Token density varies by content.
These are not Qwen tokenizer estimates or measurements. The later
materialization will record actual byte lengths.

| Request | Estimated input tokens before intervention | Approximate UTF-8 bytes (decimal kB) |
| --- | ---: | ---: |
| 1, task plus first inspection and attachments | 800–1,600 | 2–6.4 kB |
| 2, carried request 1 history plus verification | 2,200–3,400 | 5.5–13.6 kB |
| 3, full carried history and final task | 4,000–5,500 | 10–22 kB |

These counts include prior assistant output, actual environment messages,
wrappers, and the final request. Positive target bodies are expected to be
about 1,000–1,400 tokens (roughly 2.5–5.6 decimal kB), but that is not a
promise of eligibility or exact savings. They remain natural artifacts; do
not add content to meet a target.

| Case | Final BASELINE / NO_OP prompt | Candidate attachment | Legal reduction expected |
| --- | ---: | ---: | ---: |
| CP01 | 4,000–5,500 tokens / 10–22 kB | 1,000–1,400 tokens / 2.5–5.6 kB | Same-state duplicate only; must meet both positive thresholds. |
| CP02 | 4,000–5,500 tokens / 10–22 kB | 1,000–1,400 tokens / 2.5–5.6 kB | Same-state duplicate only; must meet both positive thresholds. |
| CP03 | 4,000–5,500 tokens / 10–22 kB | 1,000–1,400 tokens / 2.5–5.6 kB | Same-state duplicate only; must meet both positive thresholds. |
| CP04 | 4,000–5,500 tokens / 10–22 kB | 1,000–1,400 tokens / 2.5–5.6 kB | Same-state duplicate only; must meet both positive thresholds. |
| CP05 | 4,000–5,500 tokens / 10–22 kB | 1,000–1,400 tokens / 2.5–5.6 kB apparent duplicate | 0; different state identities must remain. |
| CP06 | 4,000–5,500 tokens / 10–22 kB | 1,000–1,400 tokens / 2.5–5.6 kB apparent duplicate | 0; consumer/protected relation must retain it. |

The runtime bound is `input_tokens + 1024 <= 8192`, so the absolute input
ceiling is 7,168 tokens. The proposed materialization target is at most 5,500
tokens for every request, with a 6,000-token preflight hard stop (at least
1,168 tokens below the absolute ceiling). This leaves room for wrappers and
final messages; designing prompts near 7,168 is not justified. A case that
exceeds the hard stop is returned for offline design review, not shortened,
truncated, or run with a lower output ceiling.

The proposed positive-case thresholds are numerical design criteria, not
results. Freeze these criteria before any model outcome. Offline materialization
and a later separately authorized tokenization pass must prove admission using
conservative lower bounds on savings and upper bounds on full and cumulative
input, or an exhaustive count over the finite permitted rendered-body space.
Do not assume token counts add across serialized messages, templates, or
wrappers. Count exact realized BASELINE and INTERVENTION requests separately
when their actual carried histories exist. For conservative bounds, require
`D3_low >= 800`, `D3_low / B3_high >= 0.20`, and
`Dsum_low / sum(B)_high >= 0.08`; each upper context bound covers the complete
rendered request, including all wrappers and allowed prior-output envelopes.

- authoritative final-request input reduction of at least 800 tokens **and**
  at least 20% of that case's BASELINE final-request input;
- cumulative three-request input reduction of at least 8%; the proposed
  single request-3 omission is the only source of reduction;
- legal controls have exactly zero reduction and preserve all candidate
  occurrences.

These bounds target hundreds to more than a thousand tokens, well above the
old small duplicate deletion, without padding. At a rounded planning rate of
9–10 tokens/s, 800 tokens is about 80–89 seconds of cold prefill as a scale
illustration, not proof that elapsed-time variance is exceeded. Exact
authoritative token differences on matched realized histories are the primary
mechanism evidence; timings are secondary. If offline proof cannot guarantee
the proposed admission thresholds, return the workload for design review
before any outcome. If a realized matched case misses one, report
failure-to-qualify/inconclusive; do not lower a threshold, pad the prompt,
choose a larger target by hand, or exclude the case after outcomes.

## Deterministic evaluation and acceptance gates

Every case has a frozen finite action schema, exact expected intermediate
state, terminal answer key, required-context set, dependency graph, structural
contract, and critical-regression list. The evaluation key is isolated from
planner input and model-visible context: hidden gold keys are available only to
the evaluator. Model-visible inputs may include genuine receipts and
provenance, never a hidden gold answer. The dependency closure is fixed before
outcomes from the frozen world and finite valid actions. If an unexpected
dependency appears, stop that case for offline review; do not adapt evaluator
labels to model answers. Local task state and tool/environment results are
deterministic.
For code tasks, use pinned source revision and tests; for search and metadata,
exact path/symbol/version/fact sets; for branch tasks, exact state and branch
identity; for controls, both state/occurrence IDs or receipt links. Do not use
“answer seems better” as the primary score.

Score the full request sequence and retain separate results for:

- **Task success:** each of the three actions and final facts match the frozen
  task oracle; an arm-level pass requires all three requests and the terminal
  check to complete.
- **Required-context recall:** all preregistered required source/event IDs
  remain model-visible at their required checkpoint and all required output
  facts are recalled. Report visible availability separately from answer
  recall.
- **Dependency validity:** exact source, revision, action/result, consumer,
  receipt, and protocol links satisfy the complete preregistered dependency
  graph.
- **Critical regression:** any lost required/protocol/protected item,
  wrong-state answer, broken receipt/consumer link, invalid final task state,
  or baseline-pass to intervention-fail event.
- **Structural validity:** each model response parses the exact bounded
  action/final-answer schema, with no unknown action or tool call. A malformed
  output is not repaired.
- **Completeness:** planned and completed case-arm and request denominators;
  incomplete and procedural-mismatch results remain visible and are never
  scored as passes.

The proposed positive stratum contains CP01–CP04; CP05–CP06 are controls. The
first longer-task BASELINE adequacy rule is deliberately demanding: all six
cases must pass all three requests with required-context recall 1.0, valid
dependency protocol, and zero critical regression. This means all 18
BASELINE requests complete successfully. The four positive opportunities and
two distinct controls are each essential; 5/6 would lose 25% of positive
coverage or half the controls. This is not a population threshold or a reuse
of the earlier 5/6 or 80% proposal. A failure means the proposed cohort is
not adequate for a Claim-2 preservation comparison. It does not reverse the
accepted model gate, and it does not permit dropping a case after observing
outcomes.

Assess this all-six BASELINE rule as the final analytical gate after the full
case-major schedule above, not by running all BASELINE arms first. Keep the
case-major order fixed and do not reorder or adapt the schedule to favorable
outcomes. The preregistered arm-failure and integrity/context safety stops
still apply; all skipped slots remain reported. Report every BASELINE failure.
If any occurs, make
no preservation claim, regardless of later arm results.

Proposed cohort acceptance after that BASELINE gate:

- NO_OP passes all six cases and has byte-identical model-visible inputs to
  BASELINE at every corresponding request;
- INTERVENTION passes all six cases with required-context recall 1.0, valid
  dependency closure, and zero critical regressions;
- there are zero BASELINE-pass to INTERVENTION-fail cases;
- each of CP01–CP04 meets both materiality thresholds; CP05 and CP06 have
  zero mutation and exact preserved-state/receipt evidence;
- all 18 arm and 54 maximum-request slots are reported, including failures,
  incomplete tasks, and procedural mismatches.

These are proposed preregistration criteria, not results. An inadequate
BASELINE, a control mutation, an intervention failure, or an incomplete arm
prevents a Claim-2 quality-preservation pass. Report observed complete
failures separately from technical incompleteness; never count inconclusive
as success. If BASELINE cannot complete the tasks, conduct a separate model
capability/runtime practicality review rather than rewriting the tasks
post-outcome.

## NO_OP contract

NO_OP is procedural. It must pass the exact same Prefixity pipeline, use the
same trace and current request identity, receive an explicit frozen forced
`DO_NOTHING`, preserve context and ordering exactly, and use the same
wrappers, arm projector, serializer, token counter, runtime, deadlines, and
measurement boundaries as the other arms. It is not a standalone copied
BASELINE prompt. Its raw assistant outputs and environment results form its
own carried trajectory. The equality gates above detect pipeline effects and
state divergence; copying BASELINE responses into NO_OP would invalidate the
control.

## Primary metrics

Report these separately for each case, request, arm, and complete trajectory:

**Mechanism:** authoritative input tokens; absolute and percentage input
reduction; complete rendered-message identity/order; selected target and
frozen policy rule; dependency closure before and after; required-context
availability and answer recall; legally removable target bytes/tokens; and
control-case preservation. For each positive with matching pre-request
histories, define `B3` as BASELINE request-3 input tokens and `I3` as
INTERVENTION request-3 input tokens; `D3 = B3 - I3` and `R3 = D3 / B3`.
Across its three matched requests, define `Dsum = sum(B) - sum(I)` and
`Rsum = Dsum / sum(B)`. Report per-request and summed trajectory inputs with
all wrappers included. A NO_OP mismatch invalidates its equality/control
comparison and cannot be counted as intervention savings. Keep server-reported
prompt tokens processed and cache-related counters separate from authoritative
total input.

**Efficacy:** exact task success at every step and for the complete arm;
baseline-pass to intervention-fail count and identities; critical regressions;
required-context recall; dependency validity; structural validity; NO_OP
procedural mismatches; and incomplete/inconclusive count over the fixed
denominators. No incomplete case is counted as preserved task success.

**Secondary efficiency:** prompt-processing time, end-to-end arm time,
environment/tool work, and any repeat/recovery cost. Do not promote output or
reasoning-token reduction to a primary mechanism metric. One replicate and
this bounded local cohort cannot support a population-level significance,
universal savings, or general task-quality claim.

## Local feasibility and CPU practicality

All proposed request sizes are within the 8,192-token context by design if
the 5,500-token target and 6,000-token preflight stop hold and the fixed
1,024-token output ceiling is reserved. This is only context-size feasibility.
The accepted 9B result used a 391-token h001 request; competence on the new
multi-step tasks remains to be demonstrated under the stricter BASELINE gate.
Model capability and CPU practicality stay separate classifications.

Measured in the accepted local-9B gate on this workstation, prompt processing
was 9.2–10.3 tok/s and decode was 1.22–2.04 tok/s. These short-request
observations are descriptive, not runtime guarantees. For planning arithmetic
below, use rounded rates of 9–10 tok/s prefill and 1.2–2 tok/s decode. The
proposed full trajectory request inputs total about 7,000–10,500 tokens per arm. Eighteen
arms therefore represent roughly 126,000–189,000 baseline input tokens before
the four authorized request-3 omissions, or about 120,400–185,800 total
prompt-input tokens if each positive saves 800–1,400 tokens. These are
arithmetic estimates from proposed ranges, not measured workload counts.

Assuming 50–150 completion tokens per request gives 2,700–8,100 completion
tokens for 54 requests. At the rounded planning rates, an illustrative
prefill-plus-decode estimate is about 3.7–7.6 hours, excluding server starts,
token counting, validation, and orchestration. The 50–150 completion-token
range is a workload assumption; the accepted four-request 9B gate produced 34–100
completion tokens, and the longer tasks may produce more. One midpoint
illustration is
1,200 + 3,000 + 4,800 input tokens per trajectory, 1,100 saved in each of
four intervention request-3 prompts, and 80 output tokens per request: about
157,600 input tokens plus 4,320 output tokens, or 5.0–5.9 hours of planning
rate-based prefill/decode arithmetic. If every request generated the full
1,024 output tokens, decode alone would add 7.7–12.8 hours before prefill.
That ceiling scenario is not proof the full trajectories are executable:
generated outputs also grow carried histories and can exceed the context
bound. Actual cache, output, and startup conditions remain unknown.

Illustrative cold-prefill planning time and remaining input headroom at the
`input_tokens + 1024 <= 8192` runtime bound:

| Planned prompt input | Cold prefill at 9–10 tok/s | Headroom after reserving 1,024 output tokens |
| ---: | ---: | ---: |
| 2,000 tokens | 3.3–3.7 min | 5,168 tokens |
| 4,000 tokens | 6.7–7.4 min | 3,168 tokens |
| 6,000 tokens | 10–11.1 min | 1,168 tokens |
| about 7,000 tokens | 11.7–13 min | 168 tokens |

Estimate work as:

```text
sum(authoritative input tokens) / planning prefill rate (9–10 tok/s)
+ sum(completion tokens) / planning decode rate (1.2–2 tok/s)
+ 18 * fresh-server startup time
+ tokenization, verification, and orchestration time
```

Fresh-start duration has not been measured; the earlier gate's prerequisite
hashing duration was not a server-start measurement. An 8-hour projected
inference review boundary is a planning trigger, not a scientific criterion.
If a representative workload is scientifically suitable but CPU execution
exceeds practical limits, report
`MODEL_CAPABILITY_ACCEPTED` and
`CPU_RUNTIME_PRACTICALITY_REVIEW_REQUIRED`. Do not reject the model for being
slow or silently change model, runtime, context, or output ceiling. This
single-replicate cohort is preferred over a larger powered-looking cohort:
eight positives plus four controls would require 108 requests and 36 fresh
server starts without supporting a population claim.

## Exact tokenization and context bound

No exact request token count is available at design time, and none is
generated here. A later separately authorized
`NON_INFERENCE_CONTEXT_TOKENIZATION_PASS` must separate two proofs. First, it
counts every fully specified static request component, template, and scenario
variant through the accepted `/v1/chat/completions/input_tokens` path and
establishes conservative envelopes for carried assistant outputs (at most
1,024 tokens per prior response), deterministic environment/tool-result byte
sizes, renderer output/byte sizes, and wrappers. Token counts are not assumed
additive across message serialization or templates. The finite variant
analysis or justified envelopes must establish the 6,000-token planned-request
maximum, positive thresholds using conservative guaranteed savings and upper
bounds on complete and cumulative inputs, and zero legal control reductions.
If these bounds
cannot be proven, return for offline design review. Never invent or replay
assistant answers to claim exact future counts.

Second, during any separately authorized future execution, count each actual,
fully rendered request with its realized carried assistant output and
environment history immediately before dispatch. Require both the planned
6,000-token maximum and `input_tokens + 1024 <= 8192` (an absolute 7,168-token
input limit). A request that violates either guard is not sent; record a
context-bound/inconclusive result and do not retry, truncate, summarize, or
lower the output ceiling. Confirm the final and cumulative token deltas on
actual matched histories; if a realized case misses a fixed admission
threshold, report failure-to-qualify/inconclusive without post-outcome
exclusion or threshold changes. This tokenization pass is not model inference,
but requires its own tokenizer/server-start authorization. No tokenization,
server startup, port-8080 contact, or inference occurred for this design.

## Implementation work required for the next task

`CLAIM_2_WORKLOAD_MATERIALIZATION_PREPARATION` is the next authorized
substantive task. It is offline preparation only. It should:

1. Define local deterministic state machines, finite action schemas, native
   context-attachment/rendering contracts, provenance identities, raw output
   carry-forward, and exact expected task/evaluation records for CP01–CP06.
2. Demonstrate that each positive's attachment repetition is caused by a
   documented normal workflow; compute actual bytes; build full three-step
   dependency graphs; isolate all oracle labels from planner input; and prove
   message-versus-result representation is faithful.
3. Run the frozen, blinded Phase 1B.9 eligibility and exact target-selection
   path on every case, then freeze the decision before outcomes. Reject any
   case that selects a different target/class, has an unclosed dependency,
   lacks same-state identity, has another eligible intervention, or cannot
   produce the expected no-op controls. Prove parity with the existing
   selection/apply functions; do not hand-select a target.
4. Keep the dependency-safety battery separately identified, with its five
   unmaterialized cases still outside the new cohort. It does not supply
   fixtures or cases to pad the efficacy stratum.
5. Prepare arm-local history and no-retry/completeness rules, exact BASELINE /
   NO_OP equality gates, deterministic evaluators, frozen denominator, and
   safety checks. No model server is needed for that work.
6. Report natural sizes and admission issues for review. If the proposed
   action semantics, eligibility, or thresholds cannot be realized without
   a mechanism change or filler, stop and return for a separate decision.

After materialization review, exact tokenization is a distinct later task
requiring authorization. A scored pilot, BASELINE competence execution, or
any inference requires further authorization as well. This document changes
no Phase 1B policy, runtime, model identity, historical evidence, or fixture.

## Risks and open questions

- The attachment event must be real in the defined local workflow. An
  artificially duplicated context message would invalidate the scenario.
- A correct body hash does not establish same state. Snapshot/revision and
  operation identities must be independently verified.
- A context attachment cannot conceal or replace the original chronological
  tool response, receipt, assistant action, or dependency edge.
- Full output/history carry-forward can cause arm divergence. Strict equality
  may produce an inconclusive comparison; this is preferable to forcing
  matching responses.
- Model competence on these longer tasks is unknown. The accepted 9B gate and
  the proposed six-case BASELINE threshold answer different questions.
- The research policy functions are private today and the h001 runner uses
  frozen projections; exact offline parity must be shown before a future
  experiment.
- Positive size ranges, target thresholds, byte/token conversion, request
  sizes, output lengths, and CPU duration are design estimates until
  materialization and tokenization; only measured outcomes may validate them.
- A one-replicate six-case cohort can reveal serious unsafe behavior or a
  bounded mechanism result. It cannot estimate population task success,
  powered non-inferiority, or broad agent generalization.
- Server startup and per-arm cache conditions have not been measured for this
  new request pattern. They must be reported separately from token evidence.

## Records reviewed

- [`PHASE_1C_PILOT_CONTEXT_ADEQUACY_REVIEW.md`](PHASE_1C_PILOT_CONTEXT_ADEQUACY_REVIEW.md)
  — accepted instrument, inadequacy finding, retained battery, and current
  scope boundary.
- [`PHASE_1C_LOCAL_9B_FEASIBILITY_GATE_EXECUTION_RECORD.md`](PHASE_1C_LOCAL_9B_FEASIBILITY_GATE_EXECUTION_RECORD.md)
  — accepted artifact/runtime identity, gate outcome, and measured throughput.
- [`PHASE_1C_CAPABLE_MODEL_DESIGN_DECISION.md`](PHASE_1C_CAPABLE_MODEL_DESIGN_DECISION.md)
  — Claim-1/2/3 separation, instrument decision, context-bound rule, and
  deferred context-adequacy threshold.
- [`PHASE_1B9_PREREGISTRATION.md`](PHASE_1B9_PREREGISTRATION.md) and
  [`PHASE_1B9_HELD_OUT_INTERVENTION_RECALL.md`](PHASE_1B9_HELD_OUT_INTERVENTION_RECALL.md)
  — controlled-only research policy, frozen rule order, and scope.
- [`PHASE_1B_DECISION_CONTRACT.md`](PHASE_1B_DECISION_CONTRACT.md) and
  [`SUCCESS_CRITERIA.md`](SUCCESS_CRITERIA.md) — production decision boundary,
  task-quality criteria, and no-op/control requirements.
- [`WORKLOAD_CORPUS.md`](WORKLOAD_CORPUS.md) — ordered trajectory, provenance,
  dependency-label, and evaluation-leakage requirements.
- [`phase1b9.rs`](../../crates/prefixity-controlled-benchmark/src/phase1b9.rs),
  [`decision.rs`](../../crates/prefixity-core/src/decision.rs), and
  [`phase1c_h001.rs`](../../crates/prefixity-controlled-benchmark/src/phase1c_h001.rs)
  — current selection, safety, and existing h001 arm/evaluator boundaries.

This is a proposed controlled workload, not a materialized corpus, a
BASELINE result, or evidence that context reduction preserves success.

`CLAIM_2_WORKLOAD_DESIGN_READY`
