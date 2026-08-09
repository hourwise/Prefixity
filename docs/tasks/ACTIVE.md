# Active Task — Phase 1B.3 Raw Artifact Access and Upstream Schema Verification Gate

Status: complete; assessment `PASS WITH RECORDED LIMITATIONS`.

## Objective

Obtain or inspect raw CodeTraceBench artifacts from the exact accepted dataset
revision and determine which structural identifiers and relationships actually
exist upstream.

Resolve the central uncertainty left by Phase 1B.2:

> Does CodeTraceBench itself contain materially richer explicit structure than
> the current Phase 1A derivative representation preserved?

This is a raw-artifact access and schema-verification gate.

Do not revise the importer.
Do not change the Phase 1B planner.
Do not begin Phase 1C.

## Background

Phase 1B.1 ran the frozen Phase 1B.0 planner over 719 accepted request traces.

All 719 produced `DO_NOTHING`.

Phase 1B.2 established that the accepted derivative representation lacks
sufficient evidence to exercise positive planner gates:

- no evidenced true `optional`, `required`, or `stale` values;
- no dependency edges;
- no explicit tool-call/result linkage;
- no provider usage;
- no exact block/request -> evaluation step mapping;
- limited semantic-zone information.

However, Phase 1B.2 could not inspect the exact upstream raw `.traj.json` /
archive structure because those raw artifacts were not present in the accepted
local fixture.

Therefore CodeTraceBench itself has NOT yet been shown to be unsuitable.

This task must resolve that uncertainty before any importer revision or corpus
pivot.

## Required context

Read only the relevant sections of:

- `../phase-1/PHASE_1A_CORPUS_CLOSEOUT.md`
- `../phase-1/PHASE_1B_DECISION_CONTRACT.md`
- `../phase-1/PHASE_1B1_CHARACTERIZATION.md`
- `../phase-1/PHASE_1B2_EVIDENCE_GAP_STUDY.md`
- `../phase-1/WORKLOAD_CORPUS.md`
- `../phase-1/QUALITY_GATE.md`
- `../SOURCE_OF_TRUTH.md`

Inspect the existing Phase 1A selection/provenance files so that any raw
artifact obtained can be checked against the already accepted corpus identity.

Do not recursively read unrelated documentation.

## Pinned corpus identity

The only authorized corpus for this task is:

- Dataset: `NJU-LINK/CodeTraceBench`
- Revision:
  `aa213b84ffb6690fc37ca15766d6ca174ec36d4d`
- Split: `verified`
- Accepted Phase 1A fixture:
  `fixtures/phase-1a/codetracebench-mini-swe-v1`
- Accepted selected trajectories: 24
- Accepted request traces: 719

Do not silently use:

- latest/main;
- another CodeTraceBench revision;
- ContextBench current main;
- a regenerated dataset;
- another benchmark;
- similarly named repositories.

If the exact revision cannot be accessed, record that and stop.

## Source hierarchy

Use primary sources only.

Acceptable evidence includes:

1. exact files from the pinned dataset revision;
2. exact dataset metadata from that revision;
3. the pinned revision's README/documentation;
4. raw artifacts referenced by that exact revision.

Do not infer raw-schema semantics from:

- current/latest revisions;
- another repository version;
- related papers when artifact structure disagrees;
- blog posts;
- secondary descriptions;
- assumptions about mini-SWE-agent formats.

Observed artifact structure outranks descriptive prose when they differ.

Record any disagreement.

## Raw-artifact acquisition

Attempt to obtain only the raw artifacts necessary to resolve the schema
questions.

Prefer the narrowest viable access.

Start with the already accepted 24 trajectories if the upstream artifact layout
allows direct selection.

If archives package many trajectories together and a larger archive must be
retrieved, do not expand the research corpus: inspect only the already selected
trajectory identities.

Record:

- exact source locator;
- revision;
- artifact filename;
- source hash/checksum if available;
- locally computed SHA-256;
- byte size;
- extraction method;
- whether the artifact maps to an accepted Phase 1A provenance record.

Downloaded raw artifacts must remain local-only and untracked.

Do not commit the archive or extracted trajectory files.

Add a narrow ignore rule only if necessary.

## Integrity gate

Before using a raw trajectory as evidence, establish that it belongs to the
pinned accepted revision.

Verify as much of the following as available:

- dataset revision;
- manifest identity;
- trajectory ID;
- artifact/archive identity;
- file hash;
- accepted Phase 1A selection membership.

Do not mix raw content from another revision with the accepted fixture.

If identity cannot be established confidently, classify the artifact as
unverified and do not use it to justify importer changes.

## Privacy boundary

Raw artifacts may contain prompts, reasoning, assistant messages and tool
output.

Treat them as inspection inputs only.

Do not commit:

- prompt text;
- assistant reasoning;
- assistant response text;
- user text;
- tool output;
- reconstructed conversations;
- filesystem contents exposed by tasks;
- credentials;
- secrets;
- source-code payloads contained in trajectories.

Tracked evidence must contain only sanitized structural information such as:

- field names;
- type names;
- identifiers where safe;
- relationship shapes;
- counts;
- booleans;
- enum/value vocabularies where they are protocol metadata;
- hashes;
- path/schema locators;
- archive metadata.

Do not include raw string values merely to demonstrate a schema.

## Structural schema inventory

Inspect the actual raw trajectory object structure.

Produce a structural inventory covering:

- top-level fields;
- trajectory/session/task identity;
- stages;
- steps;
- message/events;
- roles;
- actions;
- observations;
- tool calls;
- tool results;
- IDs;
- reference/parent fields;
- timestamps/order fields;
- status/outcome fields;
- metadata objects;
- usage/token fields;
- explicit invalidation/supersession fields if any.

For each field relevant to Prefixity, record:

- field path;
- data type;
- whether always/optionally present;
- count/coverage across inspected accepted trajectories;
- whether it contains an explicit fact or merely content;
- safe evidence classification.

Do not commit field values containing raw trajectory content.

## Evidence taxonomy

Continue using the Phase 1B.2 taxonomy:

### `CAPTURED_EXPLICIT`

The exact raw artifact directly contains the fact.

### `DERIVED_STRUCTURAL`

The fact follows deterministically from explicit raw structure.

### `EVALUATION_ONLY`

The fact belongs to benchmark quality/evaluation metadata and must stay outside
planner inputs.

### `INFERRED_UNSAFE`

The fact could only be guessed or semantically interpreted.

### `ABSENT`

No defensible source for the evidence exists.

No `INFERRED_UNSAFE` field may be proposed as planner input.

## Questions to resolve

### Step and stage identity

Determine whether the raw trajectory contains explicit:

- stage IDs;
- step IDs;
- step ordering;
- parent-stage relationships.

If IDs exist, determine whether they correspond exactly to the existing
evaluation label stage/step IDs.

Do not infer equivalence from matching counts alone.

### Message/event identity

Determine whether raw messages/events contain explicit stable IDs or whether
Phase 1A's generated `message-####` identifiers are the only available
identity.

If explicit IDs exist, record their scope and uniqueness.

### Tool/action identity

Determine whether the source explicitly represents:

- action ID;
- tool-call ID;
- function/tool name;
- observation/result ID;
- result/reference ID;
- originating call reference;
- parent/action reference.

Distinguish:

- an explicit relationship;
- sequential adjacency;
- semantic resemblance.

Only explicit/deterministic relationships are admissible.

### Action -> observation linkage

Establish whether an exact link can be constructed from raw fields.

If the format contains an explicit call/result identifier pair, classify it
accordingly.

If linking requires "the next message probably belongs to this action", reject
that as unsafe unless the upstream format normatively defines that relation.

### Protocol structure

Determine whether the raw schema provides richer protocol structure than the
current normalized roles.

Investigate explicit representation of:

- system;
- user;
- assistant;
- tool;
- action;
- observation;
- reasoning/thought;
- visible assistant response;
- environment;
- control/protocol records.

Do not use natural-language content to assign a protocol type.

### Semantic zones

Determine which Prefixity zones could legitimately be produced from verified
raw protocol fields.

Create a proposed mapping:

`raw field/type -> Prefixity semantic zone -> evidence classification`

This remains design only.

Do not implement it.

### Dependencies

Search only for explicit structural dependency evidence:

- parent IDs;
- references;
- call/result relationships;
- graph edges;
- consumed-output identifiers;
- explicit prerequisites.

Do not infer semantic dependency from textual reference, chronology or task
logic.

Separate tool/protocol relations from semantic/load-bearing dependencies.

### `required`

Determine whether any explicit raw field establishes requiredness.

Do not treat:

- successful use;
- later reference;
- benchmark gold context;
- system role alone;
- inclusion in the original prompt

as proof of `required=true` unless that is the exact defined semantics of an
upstream field.

### `optional`

Determine whether any raw metadata explicitly establishes optionality.

Do not infer it.

### `stale` / invalidation / supersession

Search for explicit invalidation, replacement, supersession, versioning or
lifetime signals.

Age/order alone does not establish staleness.

### Provider usage

Determine whether the raw artifact carries actual model/provider usage:

- input tokens;
- cached tokens;
- output tokens;
- total tokens;
- model/provider identity;
- request usage payload.

Separate provider-reported usage from benchmark estimates.

### Evaluation join

This is a key gate.

Determine whether the raw schema permits an exact mapping:

`normalized request/block`
    ->
`raw trajectory event/action/message`
    ->
`raw stage/step ID`
    ->
`evaluation stage/step ID`

The mapping must be based on explicit identity or a normative deterministic
relationship.

Do not use position/count matching as proof.

If exact mapping is possible, describe the complete join key and provenance.

Do not implement it yet.

## Coverage

Where inexpensive, inspect structural fields across all 24 accepted
trajectories.

For detailed schema examples, use a deterministic sample.

Use the same solved/unsolved × short/medium/long deterministic sampling
principle already recorded in Phase 1B.2 unless raw archive structure makes
another deterministic rule necessary.

Record the rule before interpretation.

Do not cherry-pick trajectories containing richer metadata.

## Licence verification

Revisit the unresolved licence evidence at the exact pinned revision.

The current evidence records:

- dataset metadata declaring MIT;
- README declaring MIT;
- README referencing a `LICENSE`;
- no root `LICENSE` file observed at the exact checked revision.

For this task:

- inspect the exact pinned revision for the referenced licence material;
- inspect raw archive metadata/layout for bundled licence information if
  present;
- do not reconstruct or substitute licence text from another revision;
- do not copy a licence from current main and describe it as belonging to the
  pinned revision.

Classify the result as one of:

- `EXACT_LICENSE_FILE_VERIFIED`
- `METADATA_AND_README_ONLY`
- `CONFLICTING_LICENSE_EVIDENCE`
- `INSUFFICIENT_LICENSE_EVIDENCE`

This does not require a legal conclusion.

The purpose is provenance accuracy and determining whether redistribution
remains constrained.

## Proposed importer implications

Do not modify the importer.

For each useful verified raw field, state whether a future importer should:

- preserve it directly;
- derive a structural field from it;
- keep it evaluation-only;
- ignore it;
- leave the corresponding Prefixity field unknown.

A future importer must remain an evidence adapter, not a planner.

Do not convert raw structure into safety policy.

## Required output

Produce:

`docs/phase-1/PHASE_1B3_RAW_SCHEMA_VERIFICATION.md`

Optionally produce a compact sanitized audit:

`fixtures/phase-1a/codetracebench-mini-swe-v1/results/phase1b3-raw-schema-audit.json`

if this materially improves reproducibility.

The audit must contain no raw textual trajectory content.

The findings document must contain:

- acquisition/access result;
- exact source/revision identity;
- integrity checks;
- inspected trajectory coverage;
- structural schema inventory;
- evidence matrix;
- step/stage identity result;
- tool/action/observation relationship result;
- protocol/semantic-zone result;
- dependency result;
- required/optional/stale result;
- provider-usage result;
- evaluation-join result;
- licence result;
- importer implications;
- decision-gate answers;
- assessment;
- recommended next task.

## Decision gate

Answer explicitly:

1. Were exact raw artifacts for the pinned revision successfully accessed?

2. Were their identities verified strongly enough to use as evidence?

3. Does the raw schema contain materially more useful structure than the
   accepted Phase 1A derivative representation?

4. Which fields are `CAPTURED_EXPLICIT`?

5. Which useful facts are only `DERIVED_STRUCTURAL`?

6. Which desired fields remain `ABSENT`?

7. Does explicit tool-call/action -> observation/result linkage exist?

8. Does explicit dependency evidence exist beyond protocol/tool relationships?

9. Are `required`, `optional`, or `stale` represented explicitly?

10. Can an exact evaluation-step join be established without unsafe positional
    inference?

11. Does the raw schema contain provider usage evidence?

12. What is the exact licence-evidence classification?

13. Is a narrow importer/evidence-model revision now justified?

14. Would such a revision materially exercise at least one currently untested
    Phase 1B planner evidence path?

15. Should CodeTraceBench remain the Phase 1B corpus?

## Assessment outcomes

Choose one.

### `PASS`

The exact raw schema contains sufficient verified structural evidence to justify
a narrow importer/evidence-model revision and another Phase 1B characterization.

### `PASS WITH RECORDED LIMITATIONS`

Useful raw structure exists and justifies a narrow revision, but important
planner evidence classes remain unavailable.

### `PIVOT`

The exact raw schema can be inspected but still lacks enough planner-safe
evidence to justify meaningful Phase 1B progression. Recommend a separately
authorized corpus/evaluation strategy review.

### `STOP`

Exact artifacts cannot be verified/accessed, or provenance/licence/privacy
problems prevent this evidence path from being relied upon.

Do not choose an outcome based on desired intervention count.

## Stop conditions

Do not:

- change the importer;
- change the Phase 1B planner;
- alter decision rules or thresholds;
- invent missing IDs;
- infer tool links from adjacency unless normatively guaranteed;
- infer dependencies from text;
- fabricate required/optional/stale metadata;
- expose evaluation labels to planner inputs;
- commit raw trajectories or archives;
- broaden to another corpus;
- begin Phase 1C;
- replay or mutate prompts;
- make live provider/model calls;
- implement compression;
- add current provider pricing;
- start the recommended next task;
- commit or push.

## Checks

If an inspection/download script is created, keep it read-only and deterministic.

Run focused tests for new tooling where appropriate.

At minimum:

- validate any compact JSON audit;
- verify tracked evidence contains no raw trajectory content;
- verify downloaded/extracted artifacts remain ignored/untracked;
- run `git diff --check`.

Product tests are optional if Rust/product code is untouched.

## Completion record

On completion record:

- raw-artifact access result;
- exact revision/artifact identity;
- hashes/integrity result;
- inspection coverage;
- schema findings;
- evidence classifications;
- evaluation-join result;
- licence result;
- importer implication;
- decision-gate answers;
- checks;
- Phase 1B.3 assessment;
- limitations;
- recommended next task.

Do not begin the recommended next task.

## Phase 1B.3 completion record — 2026-08-09

### Raw-artifact access result

Exact raw artifacts were successfully accessed for all 24 already accepted
trajectories from `NJU-LINK/CodeTraceBench`, split `verified`, at revision
`aa213b84ffb6690fc37ca15766d6ca174ec36d4d`. Metadata, README, verified
manifest and all 24 selected `bench_artifacts/full/*.tar.zst` archives were
inspected read-only in system temp storage. No raw archive, extracted
trajectory, prompt, reasoning, assistant/user text, tool output or
reconstructed conversation was added to the repository.

### Revision, artifact and hash verification

- Exact metadata revision SHA matched
  `aa213b84ffb6690fc37ca15766d6ca174ec36d4d`.
- The exact manifest contained all 24 accepted selection IDs with no missing
  or extra selected records.
- All 24 archive locators resolved and were locally SHA-256 hashed.
- Each archive contained exactly one `.traj.json` member.
- All 24 raw trajectory-member SHA-256 values matched the existing Phase 1A
  `source_file_sha256` ledger.
- Raw message totals reconciled to 1,498; assistant-message totals reconciled
  to 719; manifest `step_count` totals reconciled to 719.
- The previously recorded README and manifest source hashes were malformed
  or truncated. They were corrected in
  `fixtures/phase-1a/codetracebench-mini-swe-v1/corpus-provenance.json` using
  the exact locally computed 64-character hashes.

### Inspection coverage and schema findings

The deterministic six-trajectory detailed sample rule from Phase 1B.2 was
fixed before interpretation. All 24 archives were subsequently walked
structurally. The raw object has `info`, ordered `messages` and
`trajectory_format`; messages have explicit `role`, opaque string `content`
and numeric `timestamp`; assistant messages have embedded response envelopes
with globally unique response IDs, model/response metadata and provider
usage. The top-level roles observed are only `system`, `user` and `assistant`.

The raw schema is materially richer than the Phase 1A derivative for
timestamps, response identity, provider/model response usage and bounded
evaluation source locators. `tool_calls` and `function_call` fields occur only
as null placeholders in the inspected response messages; no actual tool-call
object or result object was captured.

### Evidence classifications and linkage results

- `CAPTURED_EXPLICIT`: exact revision/manifest/artifact identity, raw format,
  roles, timestamps, response IDs, provider/model identity, provider usage and
  evaluation source-locator fields (the last remain evaluation-only).
- `DERIVED_STRUCTURAL`: message array indexes, `messages[n]` paths, generated
  source-event IDs/indexes, role-only coarse zones and explicit
  evaluation-line-locator to raw-message mapping.
- `EVALUATION_ONLY`: stage IDs, stage boundaries, step IDs, solved/outcome and
  incorrect/unuseful labels, `action_ref` and `observation_ref` metadata.
- `INFERRED_UNSAFE`: content-based protocol assignment, adjacency-based
  action/result pairing, semantic dependency from text/chronology and safety
  flags inferred from age/role/outcome. None are proposed as planner input.
- `ABSENT`: raw message IDs, actual action/tool-call IDs, observation/result
  IDs, call-result references, dependency edges, `required`, `optional`,
  `stale` and invalidation/supersession fields.

The evaluation sidecar has 60 labelled step records. Thirty-two records have
explicit action/observation source references, producing 63 non-null
path/line locators. All 63 locators map structurally to exactly one raw
`messages[]` object. Twenty-eight labelled records have no such references,
so the exact evaluation join is bounded/partial, not a complete positional
join. Labels remain outside planner inputs.

Provider usage is explicit on all 719 assistant response envelopes, with
provider-specific prompt/completion/total/input/output/cache/detail fields.
This is captured response telemetry, not the derivative surrogate estimate.

### Licence classification

At the exact pinned revision, metadata declares `mit` and the primary README
declares the MIT License and links `LICENSE`; the exact root tree has no
`LICENSE` file. No licence text was reconstructed or substituted.

Classification: `METADATA_AND_README_ONLY`.

### Importer implications and decision gate

No importer or Phase 1B planner change was made. A separately authorized
narrow evidence-adapter revision is justified for typed preservation of raw
response IDs, timestamps, provider usage and evaluation source locators, with
hash-only/privacy boundaries. Required/optional/stale, dependency, actual
tool/result linkage and complete evaluation joining remain unknown/absent.

The raw evidence can newly exercise the provider-evidence/usage-presence and
provenance reporting paths, but it does not justify a destructive planner
candidate or any planner-rule change. CodeTraceBench should remain the bounded
Phase 1B corpus for that separately authorized adapter/recharacterization
task. Phase 1C was not started.

Detailed evidence is in
[`PHASE_1B3_RAW_SCHEMA_VERIFICATION.md`](../phase-1/PHASE_1B3_RAW_SCHEMA_VERIFICATION.md)
and the compact sanitized audit is in
[`phase1b3-raw-schema-audit.json`](../../fixtures/phase-1a/codetracebench-mini-swe-v1/results/phase1b3-raw-schema-audit.json).

### Checks

- Compact audit JSON validation: passed with Python JSON parsing and
  assertions for 24 artifact records, 64-character hashes, reconciled counts,
  partial evaluation join and privacy flags.
- Tracked-evidence privacy scan: passed; no raw content fields were retained,
  no new findings/audit line has trailing whitespace, and no credential value
  was read or printed.
- Raw archive/extraction tracked or untracked check: passed; no `.tar.zst` or
  `.traj.json` path exists in the workspace. The downloaded material remains
  in system temp outside the repository.
- Exact README/manifest hashes: passed against the corrected
  `corpus-provenance.json` values.
- `git diff --check`: passed.
- Rust/product tests: not run; no product code changed.

Recommended next task: separately authorize a narrow evidence-adapter revision
and frozen Phase 1B recharacterization. Do not begin it as part of this task.
