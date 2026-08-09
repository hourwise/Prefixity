# Active Task — Phase 1B.1 Offline Decision Characterization and Reporting Freeze

Status: ready for implementation.

## Objective

Characterize the frozen Phase 1B.0 conservative planner over the accepted
Phase 1A CodeTraceBench-derived traces.

Freeze a deterministic, compact reporting schema before interpreting the
corpus results, then audit the planner's real-workload decision distribution,
safety invariants and evidence coverage without changing planner rules in
response to the results.

This is an offline characterization task.

It does not authorize prompt mutation, planner tuning, Phase 1C replay or live
provider calls.

## Required context

Read only the relevant sections of:

- `../phase-1/PHASE_1_PLAN.md`
  - Phase 1B — Offline intervention planning
  - Phase boundaries
- `../phase-1/SUCCESS_CRITERIA.md`
  - Phase 1B pass
  - hard failure / pivot criteria
- `../phase-1/QUALITY_GATE.md`
  - evidence tiers
  - safety failures
  - fail-open behaviour
- `../phase-1/PHASE_1A_CORPUS_CLOSEOUT.md`
  - accepted CodeTraceBench corpus
  - deterministic 24-trajectory / 719-request slice
  - provenance and evaluation-label boundary
- `../phase-1/PHASE_1B_DECISION_CONTRACT.md`
  - authoritative decision contract
  - conservative baseline invariants
- `../SOURCE_OF_TRUTH.md`
  - current implemented state and limitations

Inspect the existing Phase 1A tooling and Phase 1B planner before adding any
characterization code.

## Frozen planner boundary

The Phase 1B.0 planner committed at the Phase 1B.0 checkpoint is the subject
of this characterization.

Do not modify its decision thresholds, decision rules, reason-code semantics,
dependency handling or intervention eligibility in response to corpus results.

In particular:

- do not increase intervention coverage;
- do not make optional/stale/dependency claims that are absent from input;
- do not reinterpret non-gold material as removable;
- do not lower safety requirements to create positive cases;
- do not add corpus-specific exceptions;
- do not use evaluation labels as planner inputs.

If characterization exposes a planner safety defect, record it and stop rather
than repairing and rerunning the planner within this task.

Narrow fixes to characterization/reporting infrastructure are allowed if they
do not change planner behaviour.

## Reporting schema freeze

Before interpreting aggregate corpus results, define a versioned
characterization-report schema.

Use one authoritative schema/version for this task.

At minimum record:

### Corpus identity

- corpus name;
- exact corpus revision;
- split;
- Phase 1A fixture identity;
- trajectory count;
- request-trace count;
- excluded/missing cases already established by Phase 1A.

### Planner identity

- intervention-plan contract version;
- Prefixity git/base checkpoint;
- planner mode/configuration;
- whether provider/economic/quality inputs were available.

### Execution

- traces attempted;
- plans produced successfully;
- validation/planning failures;
- first-pass aggregate hash;
- second-pass aggregate hash;
- deterministic match result.

### Decision distribution

Report separately:

- recommendation counts by all six contract classes;
- number of traces containing each class;
- traces with at least one non-no-op intervention;
- traces whose result is `DO_NOTHING`;
- traces with multiple intervention candidates;
- target-block counts by intervention class.

Do not convert these counts into savings claims.

### Evidence distribution

Record aggregate counts for:

- reason codes;
- evidence strengths;
- expected quality-risk values;
- provider-state-dependence values;
- provider evidence present/absent;
- economic evidence present/absent;
- quality evidence present/absent;
- dependency evidence states where represented.

Keep these evidence dimensions separate.

### Safety audit

Record explicit counts for at least:

- destructive recommendations targeting required blocks;
- destructive recommendations targeting protocol-critical blocks;
- destructive recommendations targeting current/user-request blocks;
- destructive recommendations violating known dependency closure;
- destructive recommendations made despite missing/cyclic dependency evidence;
- unsafe cross-zone/chronology relocation recommendations;
- `COMPRESS_CANDIDATE` emissions;
- `DO_NOTHING` coexisting with actual intervention recommendations;
- contradictory destructive recommendations for the same target;
- source-trace byte/hash changes before versus after planning.

Every count above should be zero unless the characterization has discovered a
planner defect.

### Deterministic examples

Select a small deterministic set of examples from emitted classes.

Use stable selection such as lexicographically first trace/request IDs rather
than manually choosing favourable examples.

Record only sanitized IDs, class, reason codes and compact evidence metadata.

Do not commit raw trajectory text.

## Characterization implementation

Prefer a small repository-native characterization runner rather than manually
invoking the CLI 719 times.

A standard-library Python tool under `tools/` is acceptable if that matches the
existing Phase 1A evidence tooling.

The runner should:

1. discover the accepted local Phase 1A traces deterministically;
2. verify the expected corpus/provenance identity;
3. execute the existing frozen planner offline;
4. collect the versioned report fields;
5. verify safety invariants;
6. write canonical deterministic output;
7. rerun sufficiently to establish deterministic output;
8. preserve source trace files unchanged.

Reuse the existing CLI/core interface rather than implementing decision logic
in the characterization tool.

The characterization runner must contain no alternative planner rules.

## Corpus availability

Use the existing local ignored traces under:

`fixtures/phase-1a/codetracebench-mini-swe-v1/traces/`

Expected accepted Phase 1A set:

- 24 trajectories;
- 719 request traces.

The bulky Phase 1A traces remain local-only.

Do not change their ignore status merely for this task.

If the expected local evidence is missing or fails provenance checks, stop and
record the problem rather than substituting another corpus.

## Evaluation-label overlay

Planner execution must finish and its deterministic outputs must be fixed
before evaluation labels are consulted.

After that point only, the existing evaluation-only labels may be joined as a
separate post-hoc audit overlay.

They must never influence:

- planner inputs;
- intervention eligibility;
- reason codes;
- evidence strength;
- thresholds;
- deterministic example selection.

If a reliable existing mapping permits it, report separately:

- decision distribution for solved versus unsolved trajectories;
- overlap between recommendations and externally labelled
  incorrect/unuseful steps.

Any such result is correlation/diagnostic evidence only.

It is not evidence that an intervention would improve quality.

If an exact join is not available, record that fact rather than manufacturing
one.

## Interpretation rules

The characterization may establish:

- what the current conservative planner recommends on this corpus;
- how often it declines to intervene;
- which evidence/rules dominate recommendations;
- whether safety invariants hold;
- whether the current trace representation contains enough explicit evidence
  to exercise the planner.

It may not establish:

- that any recommendation is quality preserving;
- realised token savings;
- realised cache reuse;
- provider cost reduction;
- latency improvement;
- task-success improvement;
- causal benefit.

An all- or mostly-`DO_NOTHING` result is valid evidence.

Do not weaken planner rules if that occurs.

If the current Phase 1A representation lacks the explicit safety metadata
needed for useful non-no-op recommendations, record that as a result and
recommend an evidence/modeling task rather than fabricating metadata.

## Evidence storage

Keep bulky per-trace planner outputs local-only.

Commit only compact sanitized evidence sufficient to audit/reproduce the
characterization, such as:

- schema/version metadata;
- aggregate characterization report;
- deterministic hashes;
- sanitized representative IDs/reason codes;
- post-hoc label summary if performed.

Do not commit raw trajectory text, reconstructed prompts, model reasoning,
credentials or duplicated 719-plan output sets.

Add narrowly scoped ignore rules if required.

## Tests and checks

Add focused tests for the characterization/reporting layer where appropriate.

At minimum verify:

- report serialization is deterministic;
- schema/version is explicit;
- all six decision classes are represented in the reporting schema even when
  count is zero;
- count totals reconcile;
- deterministic reruns match;
- safety-audit fields cannot silently disappear;
- planner output is not altered by label availability;
- raw labels are not passed into planner execution;
- source traces remain unchanged;
- existing workspace tests continue to pass.

Run the normal formatting, check, clippy and workspace test suite.

## Required outputs

Produce:

- versioned characterization/reporting schema;
- deterministic offline characterization runner;
- compact sanitized CodeTraceBench characterization evidence;
- concise Phase 1B.1 findings document or update to the existing Phase 1B
  documentation;
- completion record in this file.

Update `SOURCE_OF_TRUTH.md` only if the characterization materially changes
what the repository can claim.

## Acceptance criteria

This task is complete when:

- the reporting schema is explicit and versioned;
- the accepted 719-trace Phase 1A set is characterized without planner rule
  changes;
- all available traces either produce plans or failures are individually
  accounted for;
- a deterministic second pass reproduces the same results;
- decision and evidence distributions are recorded;
- hard safety invariants are explicitly audited;
- source traces remain unchanged;
- `DO_NOTHING` remains a legitimate result;
- evaluation labels remain isolated from planner inputs;
- any post-hoc label analysis is clearly separated;
- no corpus-specific planner tuning occurs;
- only compact sanitized evidence is prepared for commit;
- relevant tests/checks pass;
- results are sufficient to decide the next Phase 1B research task.

A high intervention rate is not an acceptance criterion.

A low or zero intervention rate is not a failure by itself.

## Assessment outcomes

Choose one:

### `PASS`

Characterization is deterministic, safety audit is clean and the current
corpus representation provides useful coverage of the frozen planner.

### `PASS WITH RECORDED LIMITATIONS`

Characterization is deterministic and safety-clean, but coverage/evidence is
limited — including a result dominated by conservative retention or
`DO_NOTHING`.

### `PIVOT`

The characterization shows that the current trace/evidence representation is
not sufficient to exercise the decision hypothesis meaningfully without a
separately designed evidence-model change.

### `STOP`

A hard safety failure, unreproducible decision behaviour or other result makes
continued Phase 1B work unjustified until reviewed.

Do not choose an assessment based on intervention count alone.

## Stop conditions

Do not:

- change Phase 1B planner rules based on corpus results;
- tune thresholds;
- mutate prompts or traces;
- begin Phase 1C;
- perform replay;
- make live provider calls;
- implement automatic compression;
- derive new safety labels from benchmark outcome labels;
- use solved/incorrect/unuseful labels as decision inputs;
- add current provider pricing;
- claim realised efficiency or quality gains;
- broaden the corpus;
- start the recommended next task;
- commit or push.

## Completion record

On completion update this file with:

- reporting schema/version;
- corpus and planner identity;
- implementation completed;
- traces/plans processed;
- decision distribution;
- evidence distribution;
- safety-audit results;
- determinism result;
- post-hoc label audit, if any;
- tests/checks run;
- interpretation and limitations;
- Phase 1B.1 assessment;
- recommended next task.

Do not begin the recommended next task.