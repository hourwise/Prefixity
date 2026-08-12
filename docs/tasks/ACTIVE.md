# Active Task — Phase 0 Foundation Slice (P0-L4)

Status: complete; P0-L4 is implemented and validated. P0-L1 through P0-L3
remain complete.

## Completion record

- P0-L1 accepted the local-first product boundary in
  [`docs/phase-0/ADR-001-LOCAL-FIRST-PRODUCT-BOUNDARY.md`](../phase-0/ADR-001-LOCAL-FIRST-PRODUCT-BOUNDARY.md)
  and aligned the product definition in `docs/SOURCE_OF_TRUTH.md`.
- P0-L2 added versioned serde contracts in
  `crates/prefixity-core/src/observation.rs`: `ContextArtifact` v1,
  `CacheObservation` v1, and `RuntimeCacheCapabilities` v1.
- P0-L3 added focused validation tests and representative observation and
  capability fixtures. Local capability examples cover llama.cpp and Ollama;
  cloud/provider examples cover DeepSeek, Meta, Mistral, Alibaba Model Studio,
  and Z.AI / GLM. Unestablished capabilities remain unknown/unverified.
- Validation: focused observation-schema tests, workspace check, formatting,
  clippy, full workspace tests, documentation review, and `git diff --check`.

The contracts are observation-only. No runtime integration, cache probing,
automatic rewriting, cache routing, KV quantisation, benchmark scoring,
ContextBench integration, or P0-L5 work was started.

## P0-L4 completion record

- Extended the existing `prefixity-controlled-benchmark` crate with versioned
  provider-neutral `ConformanceExperiment`, `ConformanceCase`,
  `ConformanceRequest`, `ConformanceResult`, and `ConformanceRunner` types.
- Separated model-visible context from the request envelope and added
  deterministic ordered JSON-field handling and request/context fingerprints.
- Added baseline/repeat, content, structured-content, tool, model, reasoning,
  and response-format mutation classes without encoding cache outcomes.
- Added a deterministic in-process mock runner that records P0-L2
  `CacheObservation` values with explicit `not_observed` telemetry and no
  fabricated token, cache, latency, quality, or provider values.
- Added the synthetic coding-agent fixture at
  `fixtures/conformance/coding-agent-cache-conformance-v1.json` and focused
  validation/determinism tests.
- Validation included focused conformance tests, observation-schema tests,
  research-state consistency tests, workspace check, formatting, clippy,
  full workspace tests, and `git diff --check`.

P0-L5 runtime adapters, live inference, cache probing, ContextBench
integration, benchmark scoring, and optimization work remain deferred.

## Historical task: Phase 1B.5 Corpus and Evaluation Strategy Review

Status: complete; Phase 1B.5 closed with `PASS WITH RECORDED LIMITATIONS`.

The Phase 1B.4 task definition and completion record are retained below as
historical evidence. No Phase 1B.6 or Phase 1C work has started.

## Current task

Determine the next evidence source and evaluation strategy after the frozen
CodeTraceBench Phase 1B.4 result. Do not implement an importer, corpus
adapter, planner change, controlled benchmark, or Phase 1C work as part of
this task.

## Phase 1B.5 completion record

Completed the public-source corpus and evaluation strategy review in
[`docs/phase-1/PHASE_1B5_CORPUS_EVALUATION_STRATEGY.md`](../phase-1/PHASE_1B5_CORPUS_EVALUATION_STRATEGY.md).

- Reviewed tau2-bench, ToolSandbox, AppWorld, BrowserGym, WebArena, AgentDojo,
  and SWE-bench using their public repositories, source schemas/documentation,
  and licence files where available. No raw third-party dataset was
  downloaded and no provider/model call was made.
- Preserved the Phase 1B.4 facts: CodeTraceBench remains useful for natural
  structural observation, provenance, usage, timestamps, and partial joins;
  its frozen planner result remains 719/719 `DO_NOTHING` because the required
  safety evidence is absent.
- Classified candidate evidence as `CAPTURED_EXPLICIT`,
  `DERIVED_STRUCTURAL`, `EVALUATION_ONLY`, `INFERRED_UNSAFE`, or `ABSENT`.
  No candidate was credited with optional, stale, dependency, required, or
  removable semantics based on adjacency, age, repetition, or evaluation
  labels.
- Recommended hybrid strategy `E`: retain CodeTraceBench for natural-workload
  observational validation and add a separate controlled intervention/quality
  track using paired/ablated, provider-neutral traces and an independent
  evaluation sidecar. AppWorld is the strongest public seed/design reference;
  its public/plain-text portion is Apache-2.0, while protected task/app/API
  material is distributed in encrypted bundles under Apache-2.0 with an
  additional encrypted-redistribution requirement. Any future Prefixity use
  must pin and audit the exact material used before implementation, and should
  not copy protected raw data into the repository. ToolSandbox is a useful
  schema reference but not the preferred raw corpus.
- Licence/privacy decision: do not copy candidate raw trajectories, encrypted
  bundles, archives, prompts, screenshots, or model output into the
  repository. The next artifact must use immutable source manifests and the
  existing hash-only boundary.
- Repository implementation remained unchanged. The accepted CodeTraceBench
  corpus and counts remain 24 trajectories, 719 request traces, and 1,498
  source events; no planner rules or evaluation joins were changed.
- Assessment: `PASS WITH RECORDED LIMITATIONS`. A useful next strategy exists,
  but a small controlled benchmark and its privacy/licence/data-leakage audit
  are still unresolved.

Recommended next task:

> **Phase 1B.6 - Controlled intervention benchmark design and seed audit.**
> Pin the public environment/task reference, define the provider-neutral trace
> and evaluation schemas, specify paired/ablated cases and leakage/privacy
> rules, and approve the seed set before implementing the benchmark or adapter.

Stop here. Do not begin the recommended next task, change the planner, begin
Phase 1C, commit, or push as part of this completion.

## Historical Phase 1B.4 task record

## Objective

Implement the smallest provenance-preserving evidence-adapter revision justified
by Phase 1B.3, then re-run the frozen Phase 1B planner over the accepted
CodeTraceBench slice.

Preserve only raw facts or deterministic structural relationships that Phase
1B.3 verified at the exact pinned corpus revision.

Do not weaken or tune the planner.

This task asks:

> Does preserving the richer evidence actually present in the raw trajectories
> improve Prefixity's observation/audit model, while retaining conservative
> decision behaviour where intervention-safety evidence remains absent?

## Required context

Read only the relevant sections of:

- `../phase-1/PHASE_1A_CORPUS_CLOSEOUT.md`
- `../phase-1/PHASE_1B_DECISION_CONTRACT.md`
- `../phase-1/PHASE_1B1_CHARACTERIZATION.md`
- `../phase-1/PHASE_1B2_EVIDENCE_GAP_STUDY.md`
- `../phase-1/PHASE_1B3_RAW_SCHEMA_VERIFICATION.md`
- `../phase-1/QUALITY_GATE.md`
- `../SOURCE_OF_TRUTH.md`

Inspect the current trace model, usage model, Phase 1A importer, provenance
structures, evaluation sidecar and Phase 1B planner before changing anything.

Do not recursively read unrelated documentation.

## Frozen evidence source

Use only:

- `NJU-LINK/CodeTraceBench`
- revision `aa213b84ffb6690fc37ca15766d6ca174ec36d4d`
- split `verified`
- the same accepted 24 trajectories / 719 requests

The Phase 1B.3 raw-schema findings are the authority for what upstream fields
were actually verified.

Do not broaden the corpus.

## Frozen planner

The Phase 1B.0 planner behaviour remains frozen.

Do not change:

- intervention eligibility;
- thresholds;
- reason-code semantics;
- dependency-safety rules;
- relocation rules;
- `DO_NOTHING`;
- compression behaviour.

If richer evidence does not create a justified intervention, `DO_NOTHING`
remains the correct result.

## Evidence that MAY be added

Phase 1B.3 verified the following useful evidence classes.

### Raw message timestamp

Raw messages contain numeric timestamps.

Preserve timestamps only with explicit provenance.

Do not infer:

- staleness;
- lifetime;
- invalidation;
- supersession;
- removability

from timestamp age.

### Provider response identity

Assistant response envelopes contain explicit provider response IDs and
response/model/status metadata.

Preserve these as provider/source identity where compatible with the existing
model.

Do not treat provider IDs as dependency or safety evidence.

### Provider usage

All 719 assistant response envelopes contain explicit provider usage telemetry.

Preserve the raw provider-specific usage without converting fields across
providers when semantics differ.

Reuse the existing Prefixity usage-schema/version mechanism where possible.

Keep:

- raw provider usage;
- schema/provider identity;
- normalized fields only where the existing versioned normalizer has an exact
  supported interpretation.

Do not:

- invent universal token counts;
- introduce current provider pricing;
- interpret usage as intervention safety;
- claim cache savings merely because cache-related provider fields exist.

### Evaluation source locators

Phase 1B.3 established an exact bounded mapping for 32 of 60 labelled
evaluation steps through explicit path/line source locators.

Preserve enough upstream structural identity to reproduce this mapping in the
evaluation sidecar.

Evaluation metadata must remain external to planner inputs.

For the remaining 28 labelled steps, preserve the absence of an exact mapping.

Do not infer joins from position or count similarity.

### Provenance

Every new captured field must state whether it is:

- `source_explicit`;
- `derived_structural`;
- `unknown`.

Reuse an existing compatible provenance representation if one exists.

Do not create an unnecessary parallel provenance system.

## Evidence that MUST remain unknown

Phase 1B.3 verified that the exact raw schema does not provide usable explicit:

- action/tool-call IDs;
- observation/result IDs;
- call-result references;
- dependency edges;
- semantic/load-bearing dependencies;
- `required`;
- `optional`;
- `stale`;
- invalidation;
- supersession.

Do not derive any of these from:

- timestamps;
- message order;
- adjacency;
- provider response IDs;
- evaluation labels;
- content;
- repetition;
- token counts;
- model outcome;
- later use.

Current false/empty schema defaults must not be described as source evidence.

## Schema/model design

Prefer the smallest backward-compatible change.

Before extending `RequestTrace` or `ContextBlock`, inspect whether the verified
facts fit existing:

- metadata;
- provenance;
- usage;
- source-map;
- trace identity

structures.

If a schema extension is required:

- version it explicitly;
- preserve compatibility with existing Phase 0/Phase 1 fixtures;
- distinguish absent/unknown from explicit false;
- do not force historical fixtures to fabricate provenance;
- document migration/compatibility semantics.

Avoid turning safety-sensitive evidence into bare booleans when provenance
would be lost.

## Importer revision

Revise the Phase 1A CodeTraceBench adapter only as needed to preserve the
verified evidence.

The adapter remains an evidence adapter.

It must not contain planner policy.

At minimum consider:

1. explicit raw timestamp preservation;
2. provider response ID/model/status preservation;
3. provider-specific usage capture;
4. typed provenance for newly captured/derived evidence;
5. explicit source locator preservation needed for partial evaluation joins.

Do not change source textual privacy behaviour.

Raw prompts/reasoning/tool output must remain untracked.

## Re-import

Regenerate the accepted derivative fixture deterministically from the same
24 pinned raw trajectories.

Preserve:

- corpus identity;
- exact revision;
- trajectory selection;
- privacy/hash-only boundary;
- label isolation.

Do not silently change the accepted workload selection.

Record whether request/source-event counts remain:

- 24 trajectories;
- 719 request traces;
- 1,498 source events.

Any count change must be explained before continuing.

## Frozen recharacterization

After the importer/evidence revision is complete and tests pass, run the frozen
Phase 1B planner over the regenerated 719 traces.

Use the Phase 1B.1 characterization schema unless an evidence-only additive
schema revision is strictly required.

If the report schema changes, version it.

Record:

- decision distribution;
- evidence distribution;
- provider-evidence coverage;
- usage-schema coverage;
- timestamp coverage;
- evaluation-locator coverage;
- safety audit;
- determinism;
- source integrity.

Do not tune the planner after seeing results.

## Expected interpretation

This task does NOT require positive interventions.

Because Phase 1B.3 found no explicit optional/stale/dependency/tool-link
evidence, it is entirely plausible that the planner remains dominated by
`DO_NOTHING`.

That is not a failure by itself.

The useful question is whether Prefixity now preserves materially better
provider/provenance/evaluation evidence without weakening safety.

## Evaluation overlay

Evaluation labels remain post-hoc only.

After planner output is frozen:

- reproduce the 32/60 exact labelled-step mappings where possible;
- report the 28 unmapped steps explicitly;
- do not pass solved/incorrect/unuseful labels to planner execution;
- do not convert evaluation failures into prune/defer recommendations.

No causal quality claim is authorized.

## Tests

Add focused tests covering at minimum:

- newly captured source-explicit evidence carries provenance;
- timestamp presence does not imply `stale`;
- provider response ID does not create dependencies;
- provider usage round-trips without semantic field conflation;
- unsupported provider fields remain raw/uninterpreted;
- historical fixtures remain compatible;
- absent safety metadata remains unknown rather than fabricated;
- evaluation locators remain outside planner inputs;
- exact evaluation mapping reproduces only where explicit source locators exist;
- no positional fallback is used for the remaining labels;
- importer remains deterministic;
- planner output remains deterministic;
- source traces are not mutated.

Run:

- formatting;
- workspace check;
- clippy;
- workspace tests;
- importer-specific tests;
- characterization checks;
- `git diff --check`.

No live provider/model calls are required.

## Required outputs

Produce:

- minimal evidence-adapter/model implementation;
- importer revision;
- focused tests;
- regenerated compact accepted evidence as appropriate;
- frozen recharacterization report;
- concise Phase 1B.4 findings document;
- completion record in this file.

Suggested findings document:

`docs/phase-1/PHASE_1B4_EVIDENCE_ADAPTER_RECHARACTERIZATION.md`

Do not commit raw trajectory archives/content.

## Decision gate

Answer explicitly:

1. Were all verified B.3 evidence fields preservable without weakening the
   privacy boundary?

2. Did the adapter remain deterministic?

3. Did the accepted corpus identity/counts remain stable?

4. Is captured versus derived evidence auditable?

5. How many requests now contain explicit provider usage?

6. Which provider usage schemas/fields were exactly interpretable?

7. How many messages contain explicit timestamps?

8. Did timestamp evidence alter any stale decision? It should not by itself.

9. Can the 32 exact evaluation joins be reproduced?

10. Are the remaining 28 still correctly unresolved?

11. Did any safety-sensitive field become known from legitimate source
    evidence?

12. What is the new Phase 1B decision distribution?

13. Did the hard safety audit remain clean?

14. Did deterministic repeated characterization match?

15. Does the richer evidence materially improve Prefixity's audit/evaluation
    capability even if intervention coverage remains zero?

16. Is another CodeTraceBench planner characterization justified, or has this
    corpus now exhausted its useful Phase 1B evidence?

## Assessment outcomes

Choose one:

### `PASS`

The narrow adapter revision truthfully preserves useful raw evidence,
recharacterization is deterministic/safety-clean, and the new evidence
materially improves Phase 1B evaluation or decision analysis.

### `PASS WITH RECORDED LIMITATIONS`

The evidence adapter improves provenance/provider/evaluation coverage and
remains safe, but intervention-relevant evidence remains substantially absent.

### `PIVOT`

The revision is technically sound but does not materially advance the central
Phase 1B decision hypothesis; recommend a separately reviewed corpus/evaluation
strategy change.

### `STOP`

The evidence revision introduces provenance ambiguity, privacy regression,
unsafe semantics, nondeterminism or another hard failure.

Do not choose the outcome based on intervention count alone.

## Stop conditions

Do not:

- tune or change the planner;
- fabricate optional/required/stale metadata;
- infer dependencies or tool links;
- infer staleness from timestamps;
- use evaluation outcomes as planner input;
- change the accepted trajectory selection;
- commit raw trajectory material;
- add current provider pricing;
- make live provider/model calls;
- begin Phase 1C;
- replay/mutate prompts;
- implement compression;
- start the next task;
- commit or push.

## Completion record

On completion record:

- model/schema changes;
- provenance design;
- importer changes;
- corpus/count integrity;
- provider usage coverage;
- timestamp coverage;
- evaluation-locator coverage;
- tests/checks;
- frozen recharacterization distribution;
- safety audit;
- determinism;
- decision-gate answers;
- Phase 1B.4 assessment;
- remaining limitations;
- recommended next task.

Do not begin the recommended next task.

## Phase 1B.4 completion record

Completed against the frozen source `NJU-LINK/CodeTraceBench`, revision
`aa213b84ffb6690fc37ca15766d6ca174ec36d4d`, split `verified`, and the existing
24-trajectory selection.

### Implementation

- Model/schema: retained trace format v2 compatibility and added optional
  evidence schema version 1. Added typed `EvidenceOrigin`, bounded
  `SourceLocator`, `EvidenceProvenance`, `ProviderFieldState`, and bounded
  `ProviderResponseMetadata`. `ContextBlock` now accepts a source-explicit
  numeric timestamp and block provenance; `RequestTrace` accepts evidence
  version, provider response metadata, and trace provenance. Historical
  fixtures remain readable without fabricating evidence.
- Provenance: every new captured/derived field is represented as
  `source_explicit`, `derived_structural`, or `unknown`, with bounded source
  locators. Generated message IDs, role-only source/zone projections, paths,
  and unique evaluation locator joins are marked derived. Provider response
  IDs are identity only, not dependency or tool relationships.
- Importer: corrected message classification to use role only; preserved raw
  numeric timestamps, response metadata, provider-specific raw usage, and
  hash-only source provenance; retained explicit evaluation locators and exact
  path/line-span joins. No content-marker, adjacency, timestamp-age,
  evaluation-label, or repetition inference creates safety evidence.
- Privacy: no raw prompts, reasoning, tool output, trajectory JSON, or archive
  was added to the repository. Decision-input traces remain hash-only.

### Evidence and recharacterization

- Corpus/count integrity: deterministic re-import produced 24 trajectories,
  719 request traces, 1,498 source events, and 724 derivative files. Two
  independent imports had identical file sets and SHA-256 hashes.
- Provider usage: 719/719 traces retain explicit raw usage. Schema counts are
  268 Anthropic custom, 118 DeepSeek custom, and 333 OpenAI Chat Completions.
  Existing exact normalization is claimed only for the OpenAI fields
  `prompt_tokens`, `prompt_tokens_details.cached_tokens`, `completion_tokens`,
  and `total_tokens`; unsupported Anthropic/DeepSeek fields remain raw.
- Provider response metadata: 719/719 traces preserve explicit response ID,
  model, `created`, and finish-reason metadata.
- Timestamp coverage: 1,498/1,498 source events preserve explicit numeric
  timestamps. Timestamp age was not used as staleness evidence and did not
  alter a planner decision.
- Evaluation join coverage: 60 labeled steps contain 63 explicit locator
  references; 32 steps have an exact bounded source-event join and 28 remain
  unresolved. No positional fallback was used. Labels were loaded only after
  both planner passes and were never planner inputs.
- Frozen planner distribution: 719/719 `DO_NOTHING`; `KEEP`, `DEFER`, `PRUNE`,
  `RELOCATE_CANDIDATE`, and `COMPRESS_CANDIDATE` each 0. This remains valid
  because no intervention-safety evidence was established.
- Safety audit: all failure counts are zero, including no source trace
  mutation, no destructive recommendation against current/required/protocol
  blocks, no missing/cyclic dependency-evidence violation, no unsafe
  relocation, no contradictory destructive recommendations, and no
  non-hypothetical recommendations.
- Determinism: both planner passes produced 719 plans, zero validation
  failures, and aggregate hash
  `5157f5a4a8b59d58d8898bf3df3fc4ad9bea60f08ccf9d920b87f41734e806fb`.
  Repeated characterization matched. No live provider/model calls were made.

### Checks and decision gate

Focused importer and characterization tests passed (`3` evidence-adapter
tests, `7` characterization tests, Python compilation). `cargo fmt --all
-- --check`, `cargo check --workspace`, `cargo clippy --workspace
--all-targets --all-features -- -D warnings`, and `cargo test --workspace`
passed (workspace test groups: 5, 59, 12, 2, 24, 12, 86, 22, plus zero-test
groups). The deterministic re-import produced identical hashes for both 724-
file outputs and preserved 24/719/1,498 counts; no raw material was present.
The final frozen characterization passed with 719 plans and zero validation
failures. Privacy/hash-only review found no raw archives or trajectory-content
files in the repository, and `git diff --check` passed.

Decision-gate answers: all verified B.3 fields were preservable without a
privacy regression; the adapter and planner were deterministic; corpus identity
and counts were stable; captured versus derived evidence is auditable; all 719
requests have provider usage; only the existing OpenAI schema fields listed
above are exactly interpretable; all 1,498 messages/events have timestamps;
timestamp evidence changed no stale decision; 32 exact evaluation joins are
reproducible and 28 remain unresolved; no safety-sensitive field became known;
the decision distribution is 719 `DO_NOTHING`; the hard safety audit is clean;
repeated characterization matches; audit/evaluation capability materially
improved even though intervention coverage remains zero; and another
CodeTraceBench planner characterization is not justified without new evidence.

Assessment: `PASS WITH RECORDED LIMITATIONS`. Remaining limitations are the
absence of explicit action/result/dependency/removability evidence, the 28
unresolved evaluation joins, lack of causal quality joins, and no replay,
savings, latency, or task-success evidence.

Recommended next task: separately review a corpus/evaluation artifact with
explicit action/result/dependency/removability identity and task-quality joins.
Do not begin that task or Phase 1C as part of this completion.

Findings: `docs/phase-1/PHASE_1B4_EVIDENCE_ADAPTER_RECHARACTERIZATION.md`.
