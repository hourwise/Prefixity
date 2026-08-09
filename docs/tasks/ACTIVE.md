# Active Task — Phase 1B.4 Verified Evidence Adapter Revision and Frozen Recharacterization

Status: ready for implementation.

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