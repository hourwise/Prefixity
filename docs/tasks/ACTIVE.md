# Active Task — Phase 1B.2 Evidence Modeling Gap Study and Importer Revision Design

Status: complete - PIVOT.

## Objective

Determine which evidence missing from the accepted Phase 1A representation can
truthfully be recovered from the underlying CodeTraceBench trajectory artifacts,
which can be deterministically derived, which would require unsafe inference,
and which is genuinely absent.

Design the minimum provenance-preserving importer/evidence-model revision needed
to exercise the Phase 1B decision hypothesis meaningfully.

This task is a study and design gate.

Do not implement the importer revision.
Do not modify the Phase 1B.0 planner.
Do not begin Phase 1C.

## Why this task exists

Phase 1B.1 characterized the frozen Phase 1B.0 planner over all 719 accepted
request traces.

Result:

- 719/719 plans succeeded;
- 719/719 emitted `DO_NOTHING`;
- 0 safety-audit failures;
- deterministic aggregate hashes matched;
- 24,416 normalized blocks contained:
  - no true `optional` flags;
  - no true `required` flags;
  - no true `stale` flags;
  - no dependency edges;
  - no provider usage;
  - only the limited normalized semantic zones available from Phase 1A.

This is a representation/evidence gap, not permission to weaken the planner.

The next question is therefore:

> What evidence can Prefixity legitimately preserve or derive from the original
> trajectories so that the planner can reason from facts rather than invented
> safety labels?

## Required context

Read only the relevant sections of:

- `../phase-1/PHASE_1_PLAN.md`
- `../phase-1/SUCCESS_CRITERIA.md`
- `../phase-1/QUALITY_GATE.md`
- `../phase-1/PHASE_1A_CORPUS_CLOSEOUT.md`
- `../phase-1/PHASE_1B_DECISION_CONTRACT.md`
- `../phase-1/PHASE_1B1_CHARACTERIZATION.md`
- `../phase-1/PHASE_1B1_CHARACTERIZATION_SCHEMA.md`
- `../SOURCE_OF_TRUTH.md`

Inspect:

- the current Phase 1A importer/adapter;
- the Prefixity trace schema;
- dependency and semantic-zone representation;
- accepted provenance fixtures;
- the raw locally available CodeTraceBench artifacts only where required.

Do not recursively read unrelated documentation.

## Frozen checkpoints

Treat these as evidence checkpoints:

- Phase 1B.0 planner:
  `3436e16afcdf359a33a691c15202900d796b25bc`
- Phase 1B.1 characterization:
  `836db0b8b6965bac0f587376d571bbdc837b19c5`
- CodeTraceBench:
  `NJU-LINK/CodeTraceBench`
- exact corpus revision:
  `aa213b84ffb6690fc37ca15766d6ca174ec36d4d`
- split:
  `verified`

Do not broaden to a different corpus during this task.

A different corpus may be recommended only as an outcome of the study.

## Evidence taxonomy

Every candidate field must be classified using one of these categories.

### `CAPTURED_EXPLICIT`

The upstream artifact directly contains the fact.

Examples may include an explicit role, identifier, timestamp, action type,
tool-call ID, result ID, step number, or relationship field.

No interpretation beyond format normalization is required.

### `DERIVED_STRUCTURAL`

The fact is not directly stored in the desired Prefixity form but follows
deterministically from explicit upstream structure.

Examples might include:

- message order;
- exact parent/child relationships where IDs make them unambiguous;
- tool-call -> tool-result linkage where a stable explicit identifier exists;
- semantic zone derived purely from an explicit protocol role.

The derivation rule must be deterministic, documented and provenance-preserving.

### `EVALUATION_ONLY`

The fact exists in benchmark/evaluation metadata but must remain outside planner
inputs.

Examples include:

- solved/unsolved outcome;
- incorrect-step labels;
- unuseful-step labels;
- benchmark gold context where applicable.

Evaluation evidence may support later post-hoc quality analysis but cannot become
planner safety evidence merely because it exists.

### `INFERRED_UNSAFE`

The field could be guessed from content, position, model behaviour or benchmark
outcome, but the source does not establish it.

Examples may include:

- calling a block `optional` because it appears unimportant;
- calling a result `stale` because it is old;
- inventing dependency edges from topical similarity;
- treating non-gold context as removable;
- treating an incorrect step as safely prunable.

These must not be proposed as importer facts.

### `ABSENT`

The required evidence is not available from the checked artifact structure and
cannot be safely derived.

The correct representation is unknown/absent.

Do not fabricate a replacement.

## Fields to investigate

At minimum study the following.

### Identity and joinability

- trajectory ID;
- task ID/name;
- stage ID;
- step ID;
- message/event ID;
- request/turn identity;
- action ID;
- observation/result ID;
- any stable upstream parent/reference identifiers.

Determine whether Prefixity can preserve sufficient identity to create an exact
post-hoc join between normalized blocks and benchmark step labels.

Do not expose evaluation labels to the planner.

### Protocol and chronology

Determine whether the source provides enough explicit structure to preserve:

- message role;
- system/user/assistant/tool distinction;
- assistant reasoning versus externally visible assistant message where
  represented;
- tool invocation;
- tool result/observation;
- chronological ordering;
- turn boundaries;
- stage boundaries;
- current request boundary.

Classify each fact according to the evidence taxonomy.

### Semantic zones

Determine which richer Prefixity semantic zones can be assigned from explicit or
deterministically derivable protocol structure.

Do not classify zones using semantic interpretation of raw natural-language
content.

Produce an explicit proposed mapping table:

`upstream structure -> Prefixity zone -> evidence class -> provenance`

Identify zones that remain unavailable.

### Tool relationships

Determine whether the artifacts support reliable relationships such as:

- tool call -> tool result;
- action -> observation;
- assistant turn -> generated action;
- result -> originating invocation;
- stage -> contained steps.

Distinguish explicit IDs from positional assumptions.

A positional relationship may be proposed only if the upstream format defines
that ordering relationship unambiguously.

### Dependencies

Investigate whether any dependency edges can be established without semantic
guessing.

Separate:

1. protocol dependency;
2. explicit reference dependency;
3. tool-call/result dependency;
4. chronology only;
5. semantic/load-bearing dependency.

Do not convert chronology into a semantic dependency.

Do not invent semantic dependencies from content similarity.

State clearly which dependency types the current corpus cannot establish.

### `required`

Determine whether the corpus provides any explicit evidence that a block is
required.

Benchmark gold context, successful trajectories, or later use do not
automatically mean `required=true`.

If benchmark data can only be used as evaluation evidence, classify it
`EVALUATION_ONLY`.

### `optional`

Determine whether any explicit upstream field establishes optionality.

If not, leave it unavailable.

Do not infer optionality from:

- tool-result age;
- low Prefixity score;
- non-gold status;
- repetition;
- solved/unsolved labels;
- absence of later reference.

### `stale`

Determine whether the artifact explicitly represents invalidation,
supersession, replacement or another defensible stale-state event.

Age alone is not staleness.

If no explicit or deterministic stale transition exists, classify this field
as unavailable rather than inventing a rule.

### Evaluation join

Determine whether the Phase 1B.1 message-ID versus benchmark-step-ID mismatch
can be solved by preserving additional upstream identifiers.

The desired result is:

`normalized block/request -> upstream step ID -> evaluation label`

The evaluation label must remain external to planner input.

Document precisely where the identifier originates and how it survives import.

## Raw-artifact inspection rules

The accepted raw trajectory artifacts may be inspected locally where needed.

Do not commit:

- raw prompts;
- raw reasoning;
- raw assistant content;
- raw tool outputs;
- reconstructed conversations;
- credentials;
- upstream archives.

Prefer structural inspection:

- field names;
- object types;
- IDs;
- counts;
- relationship shapes;
- format/version metadata.

A small read-only inspection tool is allowed if useful.

It must not become a second importer and must not modify artifacts.

If a compact evidence artifact is produced, store only sanitized structural
metadata and hashes.

## Deterministic sampling

Do not manually select favourable examples.

If inspecting fewer than all 24 selected trajectories in detail, choose a
deterministic sample before interpretation.

Prefer covering the existing solved × short/medium/long selection cells while
using stable trajectory-ID ordering.

Record the sample rule.

Use broader/all-trajectory structural counting where inexpensive.

## External-source rule

If upstream documentation must be checked, use primary sources only and pin the
exact revision where possible.

Do not infer missing CodeTraceBench semantics from:

- related repositories;
- author intent;
- similar benchmark formats;
- unrelated versions;
- blog posts or secondary descriptions.

Record disagreements between documentation and actual artifact structure.

## Required evidence matrix

Produce one authoritative matrix covering at least:

| Desired Prefixity evidence | Upstream source | Classification | Deterministic rule | Planner-safe? | Evaluation-only? | Import revision? |
| --- | --- | --- | --- | --- | --- | --- |

Rows must include:

- trajectory identity;
- stage identity;
- step identity;
- message identity;
- role/protocol type;
- chronology;
- current-request identity;
- tool invocation identity;
- tool-result identity;
- tool invocation/result linkage;
- semantic zone;
- protocol dependency;
- explicit reference dependency;
- semantic dependency;
- required;
- optional;
- stale;
- supersession/invalidation where available;
- evaluation-step join;
- provider usage;
- exact token usage.

Add fields discovered during inspection where relevant.

For every proposed importer addition, state whether it is:

- direct preservation;
- deterministic derivation;
- evaluation-only preservation.

No proposed planner input may originate from `INFERRED_UNSAFE`.

## Proposed provenance model

Design how any new imported/derived field would record its origin.

At minimum distinguish:

- `source_explicit`;
- `derived_structural`;
- `unknown`.

Evaluation metadata should remain in its existing external/evaluation channel.

Do not overload an ordinary boolean in a way that hides whether it was captured
or derived.

If the existing schema already has a better compatible mechanism, reuse it.

This task may recommend a schema extension but must not implement one.

## Importer revision design

If the study finds sufficient evidence, produce a concrete minimal revision plan
for the next task.

Specify:

- exact source fields consumed;
- deterministic derivation rules;
- new/preserved normalized fields;
- provenance attached to each;
- validation changes required;
- tests required;
- fixture changes required;
- label-isolation guarantees;
- privacy/licence implications;
- backward-compatibility implications;
- whether the Phase 1B.0 planner requires any later adaptation.

The preferred design should preserve more truthful structure without embedding
planner policy inside the importer.

The importer must remain an evidence adapter, not a hidden classifier.

## Planner boundary

Do not change:

- intervention eligibility;
- planner thresholds;
- planner reason codes;
- dependency safety rules;
- `DO_NOTHING` behaviour;
- compression behaviour.

If the study suggests the planner's contract needs a later change, record it as
a separate recommendation.

Do not implement it here.

## Decision gate

At the end, answer these questions explicitly:

1. Does the underlying accepted CodeTraceBench artifact contain materially more
   useful structural evidence than the Phase 1A representation preserved?

2. Can that evidence be preserved/derived without semantic guessing?

3. Is there enough planner-safe evidence to justify an importer/evidence-model
   revision and rerun of Phase 1B characterization?

4. Which positive planner gates could the proposed evidence legitimately
   exercise?

5. Which planner gates would still remain untestable?

6. Can evaluation step IDs be preserved sufficiently for a reliable post-hoc
   label join?

7. Should CodeTraceBench remain the Phase 1B corpus after the proposed revision?

## Assessment outcomes

Choose one.

### `PASS`

The raw artifact contains enough explicit/deterministically derivable evidence
to justify a narrow importer/evidence-model revision capable of meaningfully
re-exercising Phase 1B.

### `PASS WITH RECORDED LIMITATIONS`

A useful subset can be recovered safely, enough to justify a narrow importer
revision, but important evidence classes remain unavailable.

### `PIVOT`

The accepted corpus cannot provide sufficient planner-safe evidence even after
truthful structural preservation. Recommend a separately reviewed corpus or
evaluation strategy change.

### `STOP`

Licence/privacy/provenance ambiguity or another hard problem means this
evidence path should not proceed without external resolution.

Do not choose an outcome based on expected intervention count.

## Required outputs

Produce:

- `docs/phase-1/PHASE_1B2_EVIDENCE_GAP_STUDY.md`
- an explicit evidence classification matrix;
- raw-artifact structural findings;
- proposed provenance model;
- minimal importer/evidence-model revision design if justified;
- decision-gate answers;
- completion record in this file.

A compact sanitized structural audit JSON may be added if it materially aids
reproducibility.

Do not create a full replacement corpus fixture.

Update `SOURCE_OF_TRUTH.md` only if the study resolves an existing authoritative
uncertainty. Do not describe proposed importer changes as implemented.

## Tests/checks

This is primarily a research/design task.

If a structural inspection script is added:

- add focused tests where appropriate;
- ensure deterministic output;
- verify source artifacts remain unchanged;
- verify no raw content appears in tracked output.

Run relevant existing checks sufficient to establish that no product behaviour
was changed.

At minimum run:

- `git diff --check`;
- any focused tests for newly added tooling.

If Rust product code is untouched, a full workspace test run is optional unless
repository guidance requires it.

## Acceptance criteria

The task is complete when:

- every requested evidence class has been investigated;
- captured facts are separated from deterministic derivations;
- unsafe inference is explicitly rejected;
- absent evidence remains absent;
- evaluation-only labels remain isolated;
- semantic zones are mapped only from defensible structure;
- dependency types are distinguished rather than conflated;
- `required`, `optional`, and `stale` are not fabricated;
- feasibility of exact evaluation-step joining is established;
- privacy/licence implications are recorded;
- the proposed provenance representation makes captured versus derived evidence
  auditable;
- a concrete next importer revision exists only if supported by evidence;
- the decision gate answers whether CodeTraceBench remains suitable;
- no planner or importer behaviour was changed.

## Stop conditions

Do not:

- implement the importer revision;
- alter the Phase 1B planner;
- tune thresholds or rules;
- fabricate `optional`, `required`, `stale`, or dependency metadata;
- use benchmark labels as planner inputs;
- infer removability from non-gold status;
- commit raw trajectory content;
- broaden to another corpus;
- begin Phase 1C;
- replay or mutate prompts;
- make live provider calls;
- implement compression;
- add current provider pricing;
- start the recommended next task;
- commit or push.

## Completion record

On completion record:

- corpus/revision inspected;
- inspection method/sample;
- evidence matrix;
- captured evidence;
- deterministic derivations;
- evaluation-only evidence;
- unsafe inferences rejected;
- genuinely absent evidence;
- evaluation-join result;
- provenance-model recommendation;
- importer-revision design, if justified;
- privacy/licence findings;
- decision-gate answers;
- tests/checks;
- Phase 1B.2 assessment;
- remaining limitations;
- recommended next task.

Do not begin the recommended next task.

## Completion record - Phase 1B.2

- Corpus/revision inspected: accepted `NJU-LINK/CodeTraceBench` revision
  `aa213b84ffb6690fc37ca15766d6ca174ec36d4d`, `verified` split, 24
  trajectories and 719 normalized request traces. No other corpus was used.
- Inspection method/sample: structural JSON-only inspection of all 719
  traces and 24,416 blocks, plus the lexicographically first selected
  trajectory in each solved x short/medium/long cell. No raw content was read
  or reconstructed. No raw `.traj.json` or `.tar.zst` is available under the
  accepted local fixture root.
- Evidence matrix and study: recorded in
  `docs/phase-1/PHASE_1B2_EVIDENCE_GAP_STUDY.md`; compact sanitized counts,
  relationship checks, sample rule and fixture hashes are in
  `results/phase1b2-structural-audit.json`.
- Captured evidence: trajectory/task identity, request/session identity,
  explicit message roles, source order/index, hashes, source paths, byte
  counts and pinned provenance metadata. Evaluation stage/step IDs remain in
  the external label file only.
- Deterministic derivations: message-index IDs, request/turn prefixes,
  normalized positions, source-kind/semantic-zone mapping, structural paths,
  content hashes and surrogate token estimates. Each is documented as a
  derivation rather than an upstream fact.
- Evaluation-only evidence: solved/unsolved outcomes, stage IDs, step IDs,
  incorrect labels and unuseful labels. None entered planner inputs.
- Unsafe inference rejected: no optional, required, stale, removability,
  semantic dependency, adjacency-based tool relation, positional step join,
  provider usage, exact token usage, or content-derived zone claim was added.
- Absent evidence: explicit action/tool-call IDs, observation IDs,
  invocation/result links, stage/step mapping in traces, dependency edges,
  invalidation/supersession, optional/required/stale facts, provider usage and
  exact provider token counts.
- Evaluation-step join: trajectory-level overlap exists, but no exact
  `normalized block/request -> upstream step ID -> label` mapping is present.
  Position- or step-count-based reconstruction was rejected.
- Provenance model: recommend typed per-field origin
  `source_explicit | derived_structural | unknown`, source locators and
  versioned derivation rule IDs, with evaluation-only markers kept external.
  Existing booleans must not hide unknown evidence behind `false`.
- Importer revision design: a conditional, provenance-preserving design was
  documented only. It would preserve explicit upstream IDs and protocol kinds
  if verified, keep the hash-only privacy boundary, and add negative tests;
  it was not implemented and does not authorize a planner rerun.
- Privacy/licence: the pinned revision has the recorded MIT metadata/README
  declaration, but its README-linked root `LICENSE` file is absent. No raw
  archive, prompt, reasoning or tool output was added.
- Decision gate: captured/derived chronology and protocol structure are
  planner-safe for retention and `DO_NOTHING`, but the evidence path does not
  justify positive Phase 1B coverage or an importer rerun without raw-schema
  and identifier verification.
- Checks: structural audit JSON parsed successfully; `git diff --check`
  passed; existing product code was untouched, so no Rust product behavior
  check was required beyond the prior Phase 1B validation baseline.
- Phase 1B.2 assessment: `PIVOT`.
- Remaining limitations: raw accepted trajectory artifacts were not locally
  inspectable; optional/required/stale/dependency and exact evaluation-join
  claims remain unresolved/absent; no quality or provider evidence exists.
- Recommended next task: a narrowly scoped raw-artifact access and
  upstream-schema verification gate for this exact CodeTraceBench revision,
  including explicit step/action/tool-reference fields and licence evidence.
  If those facts remain unavailable or absent, review a separately authorized
  corpus/evaluation-strategy pivot. Do not begin it in this task.
