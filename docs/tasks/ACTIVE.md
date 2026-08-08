# Active Task — Phase 1B.0 Intervention Decision Contract and Conservative Baseline

Status: ready for implementation.

## Objective

Implement the first Phase 1B offline decision layer.

Define a deterministic, auditable intervention-plan contract and a conservative
baseline planner that converts existing Prefixity observations into justified
context-management recommendations without mutating the source trace.

This task establishes the decision boundary. It is not a pruning/compression
implementation task and does not authorize replay or live provider calls.

## Required context

Read only the relevant sections of:

- `../phase-1/PHASE_1_PLAN.md`
  - Phase 1B — Offline intervention planning
  - Context decision model
  - Phase boundaries
- `../phase-1/SUCCESS_CRITERIA.md`
  - Phase 1B pass
  - Hard failure / pivot criteria
- `../phase-1/QUALITY_GATE.md`
  - evidence tiers
  - required/load-bearing context protection
  - fail-open behaviour
- `../phase-1/PHASE_1A_CORPUS_CLOSEOUT.md`
  - accepted corpus result
  - interpretation limits
- `../SOURCE_OF_TRUTH.md`
  - implemented architecture
  - invariants and deferred work

Inspect the existing core analysis, policy simulation, trace flags,
dependencies and CLI before adding new structures.

Reuse existing concepts where possible rather than creating a parallel policy
system.

## Phase 1A boundary

Phase 1A established deterministic ingestion and offline observation on the
accepted CodeTraceBench slice.

It did not establish that any of the 712 structural candidates are safe or
beneficial interventions.

Treat those candidates as observations only.

Do not derive safety from:
- absence from gold context;
- structural volatility alone;
- token size alone;
- repetition alone;
- low Prefixity score alone.

Unknown safety must default toward retention or `DO_NOTHING`.

## Decision classes

The Phase 1B contract must represent exactly these intervention classes:

- `KEEP`
- `DEFER`
- `PRUNE`
- `RELOCATE_CANDIDATE`
- `COMPRESS_CANDIDATE`
- `DO_NOTHING`

Supporting a class in the contract does not require the conservative baseline
to emit that class without sufficient evidence.

In particular, do not invent a compression heuristic merely to produce
`COMPRESS_CANDIDATE`.

## Decision record

Each non-trivial recommendation must be auditable.

Represent at minimum:

- recommendation class;
- target block ID(s), when applicable;
- deterministic reason codes;
- human-readable explanation;
- evidence strength;
- source evidence used;
- relevant dependencies;
- expected structural effect;
- expected quality risk;
- provider-state dependence;
- whether provider evidence is present or absent;
- whether economic evidence is present or absent;
- `hypothetical_only: true`.

Keep structural evidence, provider/cache evidence, economic evidence and
quality/dependency evidence distinguishable.

Do not manufacture unavailable evidence.

## Conservative baseline

Implement a deterministic offline baseline using only evidence already present
in Prefixity traces and analysis.

Rules must include these invariants:

1. Known required blocks are always retained.

2. Protocol-critical/current-request content must not be recommended for
   destructive intervention.

3. Unknown safety defaults to `KEEP` or contributes to `DO_NOTHING`.

4. A block must not be recommended for destructive intervention when doing so
   would violate recorded dependency closure.

5. `PRUNE` may only be emitted where existing explicit metadata provides a
   defensible safe case, such as an optional stale tool result with no retained
   dependency requiring it.

6. `DEFER` may only be emitted where explicit metadata supports optionality and
   deferral without violating dependencies or protocol/order requirements.

7. `RELOCATE_CANDIDATE` is hypothetical only and must obey existing semantic
   zone and chronology constraints. Do not actually reorder the trace.

8. `COMPRESS_CANDIDATE` must remain supported by the contract but need not be
   emitted by this baseline unless an already-established evidence rule
   justifies it.

9. If no intervention is sufficiently justified, emit `DO_NOTHING`.

10. Never convert Phase 1A structural-candidate counts directly into Phase 1B
    intervention recommendations.

## Implementation

Prefer implementation in `prefixity-core` and the existing CLI rather than a
new crate.

Add the smallest API surface necessary for:

- intervention-plan data structures;
- deterministic planner execution;
- JSON serialization;
- concise human-readable explanation;
- offline CLI access if consistent with the existing command structure.

A likely CLI shape is:

`prefixity plan <trace> --json`

but inspect the current CLI conventions and use the repository-native form.

The planner must not mutate its input trace.

Do not remove or replace the existing Phase 0 policy simulator. Reuse its
validated safety logic where appropriate, while keeping simulation and Phase
1B recommendation concepts distinct.

## Tests

Add focused tests covering at minimum:

- deterministic identical output for identical input/config;
- all six decision classes serialize through the contract;
- required block -> never `PRUNE` or `DEFER`;
- unknown-safety block -> conservative retention/no-op;
- optional stale tool-result safe case -> `PRUNE` where dependencies permit;
- same case with retained dependency -> no destructive recommendation;
- defensible optional volatile case -> `DEFER` where supported;
- safe structural relocation -> `RELOCATE_CANDIDATE`, never actual mutation;
- unsafe cross-zone or chronological relocation -> rejected/no-op;
- no justified intervention -> `DO_NOTHING`;
- compression class exists without inventing a compression implementation;
- structural/provider/economic/quality evidence remains separately represented;
- planner never mutates the original trace.

Reuse existing synthetic fixtures where possible.

Do not tune rules against the Phase 1A corpus to increase the number of
non-no-op recommendations.

## Optional characterization

If the ignored Phase 1A CodeTraceBench traces are locally available, the new
planner may be run over them after implementation as a non-gating
characterization.

If performed:

- keep bulky outputs local-only;
- record only compact aggregate evidence if useful;
- report the distribution of Phase 1B decisions;
- do not tune the rules in response to that distribution;
- do not treat corpus outcome labels as planner inputs;
- do not claim quality or savings.

Absence of the local corpus must not cause tests or CI to fail.

## Required outputs

Produce:

- implementation of the intervention-plan contract;
- conservative baseline planner;
- focused tests;
- CLI exposure if appropriate;
- documentation of decision semantics/invariants;
- completion record in this file.

Update `SOURCE_OF_TRUTH.md` only if implementation materially changes the
authoritative implemented-state description.

## Acceptance criteria

The task is complete when:

- all six Phase 1B decision classes exist in one authoritative contract;
- recommendations are deterministic and auditable;
- evidence dimensions remain distinguishable;
- required/protocol-critical context cannot receive destructive
  recommendations;
- dependency closure is respected;
- non-gold or unknown context is not automatically treated as removable;
- weak/insufficient evidence defaults to retention or `DO_NOTHING`;
- the conservative baseline demonstrates at least one defensible non-no-op
  case using existing explicit metadata;
- `DO_NOTHING` remains reachable;
- no trace is mutated;
- no compression implementation is invented;
- focused tests and existing workspace tests pass;
- documentation clearly labels recommendations as offline/hypothetical.

This task does not need to satisfy the complete Phase 1B pass criteria.
It establishes the Phase 1B decision contract and conservative baseline on
which subsequent Phase 1B characterization can build.

## Stop conditions

Do not:

- mutate real prompts or traces;
- begin Phase 1C replay;
- make live provider calls;
- implement automatic compression;
- build a learned classifier/pruner;
- use evaluation outcomes as decision inputs;
- tune rules to maximize intervention count or estimated token reduction;
- infer that non-gold context is removable;
- add current provider pricing;
- redesign the runtime;
- start the next task;
- commit or push.

## Completion record

On completion, update this file with:

- implementation completed;
- decision contract added;
- conservative rules implemented;
- tests/checks run;
- optional corpus characterization, if performed;
- limitations and unsupported decisions;
- Phase 1B.0 assessment;
- recommended next task.

Do not begin the recommended next task.