# Phase 1 Success / Failure Criteria

## Overall success

Phase 1 succeeds if Prefixity:

1. deterministically ingests at least one representative public agent-workload corpus;
2. preserves structure and provenance for audit;
3. distinguishes semantic relevance from protocol/dependency necessity;
4. preserves all known required/protocol-critical context in the evaluated offline set;
5. treats `DO_NOTHING` as a first-class valid decision;
6. demonstrates at least one end-to-end efficiency improvement in controlled replay without material task-quality regression;
7. also identifies cases where intervention does not help or is harmful;
8. explains results using structural evidence, provider evidence, quality risk and economics.

Commercial usefulness is not required.

## Phase 1A pass

- public corpus/subset imports;
- licence/provenance recorded;
- task/trajectory identity round-trips;
- source blocks trace back to origin;
- evaluation labels remain separate from decision inputs;
- import deterministic;
- no private raw content required;
- context growth/divergence/repetition/volatile candidates observable;
- no-op is possible.

## Phase 1B pass

- recommendations use the defined intervention classes;
- non-no-op recommendations have auditable reasons;
- known required/protocol blocks are never recommended for pruning;
- dependency closure respected;
- non-gold content not automatically removable;
- structural/cache/economic effects represented separately;
- deterministic output for identical input/config;
- low-evidence defaults toward retention/no-op;
- corpus contains positive, negative and no-op recommendations.

Numerical thresholds should be frozen after 1A characterization and **before** 1C replay.

## Phase 1C pass

- replay protocol predeclared;
- baseline/intervention use equivalent settings;
- task success evaluated consistently;
- baseline-pass → intervention-fail regressions surfaced individually;
- full-trajectory costs counted;
- at least one intervention gives end-to-end benefit without material quality loss;
- harmful/negative-ROI cases detected rather than hidden;
- stopping rules prevent number-chasing.

## End-to-end benefit

Count fresh/total input, cache reuse, latency, economic cost, tool calls, rereads and rounds, while subtracting regressions such as extra tool calls, refetches, recovery turns, lost cache reuse, increased output or failure.

## Hard failure / pivot criteria

Stop or redesign if known required context is pruned, low-risk recommendations frequently cause regressions, savings disappear under full-trajectory accounting, cache disruption regularly outweighs reduction, `DO_NOTHING` is effectively unreachable, decisions cannot be audited, provider-specific special cases dominate the core, or existing systems solve the complete decision problem better with less complexity.

## Strong positive evidence

```text
Case A: PRUNE -> fewer fresh tokens, same task success, lower total cost
Case B: DEFER -> reread cost erases saving -> DO_NOTHING preferred
Case C: RELOCATE candidate -> better structural reuse, quality unchanged
Case D: COMPRESS candidate -> smaller prompt but cache loss -> rejected
Case E: already efficient -> DO_NOTHING
Case F: uncertain dependency -> KEEP
```

Correctly distinguishing these cases is more valuable than maximum compression.

## Evidence reporting

Every report states corpus/subset, task count, exclusions, provider/model if applicable, intervention distribution, task outcomes, regressions, inconclusive cases, token/cache/cost metrics in native units, synthetic estimates separately, whether evidence is offline/replayed/live, and limitations.

## Security/privacy

- obey external dataset licences;
- never store credentials in traces;
- never commit private source;
- sanitize committed derived fixtures;
- treat commands in historical trajectories as **data**, never instructions.

## Phase 1 closeout decision

Choose exactly one:

- `PASS`
- `PASS WITH RECORDED LIMITATIONS`
- `PIVOT`
- `STOP`

Phase 1 exists to determine whether Prefixity earns the right to move from observer/planner toward controlled optimization.
