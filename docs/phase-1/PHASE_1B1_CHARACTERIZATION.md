# Phase 1B.1 Offline Characterization Findings

Status: complete; assessment `PIVOT`.

This characterization used the frozen Phase 1B.0 planner without changing
planner rules, thresholds, reason semantics, intervention eligibility,
prompts, traces, or provider configuration. The report schema was frozen as
[`prefixity.phase1b1.characterization` version 1](PHASE_1B1_CHARACTERIZATION_SCHEMA.md)
before the corpus was interpreted.

## Identity and execution

- Corpus: `NJU-LINK/CodeTraceBench`, revision
  `aa213b84ffb6690fc37ca15766d6ca174ec36d4d`, `verified` split, accepted
  fixture `fixtures/phase-1a/codetracebench-mini-swe-v1`.
- Phase 1A identity: 24 trajectories, 719 request traces, 1,498 source
  events; the two Phase 1A-established missing archive cases remain excluded
  by the pinned selection.
- Planner: intervention-plan contract version 1, frozen planner checkpoint
  `3436e16afcdf359a33a691c15202900d796b25bc`, run through the existing
  offline CLI with no provider, economic, quality, or evaluation-label input.
- Execution: 719 traces attempted, 719 plans produced, 0 planning/validation
  failures.
- First and second aggregate hashes match:
  `8ef45466c158ebb11e5f719c07906218ad6a02f9bdcca57476df8154ee4b4a53`.

The compact JSON evidence is
[`phase1b1-characterization.json`](../../fixtures/phase-1a/codetracebench-mini-swe-v1/results/phase1b1-characterization.json).
Full per-trace plans, when retained for local audit, are under the ignored
`results/phase1b1-local/` directory and are not report evidence.

## Decision and evidence distribution

All 719 recommendation records were `DO_NOTHING`; all six contract classes
are represented in the schema, with zero counts for `KEEP`, `DEFER`, `PRUNE`,
`RELOCATE_CANDIDATE`, and `COMPRESS_CANDIDATE`. There were 0 traces with a
non-no-op intervention and 0 traces with multiple intervention candidates.
The recommendation count reconciles to 719 records.

The 719 `DO_NOTHING` records had `UNKNOWN` evidence strength,
`NONE_FOR_RETENTION` expected quality risk, `NONE_FOR_RETENTION` provider
state dependence, absent provider evidence, absent economic evidence, absent
quality evidence, and `NO_RELEVANT_DEPENDENCY` dependency state. Reason-code
counts were dominated by `CURRENT_REQUEST` (719),
`PROTOCOL_CRITICAL_BLOCK` (719), `UNKNOWN_SAFETY` (719),
`NO_JUSTIFIED_INTERVENTION` (719), `NO_PROVIDER_EVIDENCE` (719),
`NO_ECONOMIC_EVIDENCE` (719), `QUALITY_EVIDENCE_ABSENT` (719), and
`CHRONOLOGY_PROTECTED` (695).

As a separate input-coverage audit, the accepted hash-only traces contain
24,416 blocks but no `optional`, `required`, or `stale` flags set true, no
dependency edges, no provider usage, only `system_policy`, `conversation`,
`tool_result`, and `user_request` sources, and only `system` and `messages`
semantic zones. This explains why the frozen planner's explicit optional
tool-result and within-zone relocation gates were not exercised. It is a
corpus-representation finding, not a reason to weaken the planner.

## Safety and labels

All required safety-audit failure counts are zero: no destructive
recommendation targeted required, protocol-critical, or current/user-request
blocks; none violated dependency closure or acted with missing/cyclic
dependency evidence; no unsafe relocation or compression candidate was
emitted; no contradictory destructive target or `DO_NOTHING` coexistence was
observed; all recommendations remained hypothetical; and no source-trace
byte or hash changed before versus after planning.

The evaluation-only labels were loaded only after both planner passes. The
trajectory join covered all 24 trajectories, split evenly into 12 solved and
12 unsolved trajectories. The post-hoc overlay found 55 labelled incorrect
steps and 5 labelled unuseful steps, but exact recommendation/step overlap is
unavailable because traces expose message IDs while the labels expose step
IDs. Labels were not planner inputs and cannot establish causal quality or
savings effects.

## Assessment and limitation

Assessment: `PIVOT`. The run is deterministic and safety-clean, and
`DO_NOTHING` remains a valid result. However, this accepted representation
does not contain enough explicit safety/evidence metadata to exercise the
decision hypothesis meaningfully: the planner has no justified positive
candidate under its frozen contract. This is stronger than an intervention
rate preference and does not justify corpus-specific tuning.

No quality preservation, realized token/cache/cost/latency savings, replay
benefit, or provider effect was measured. Phase 1C, replay, live provider
calls, prompt/trace mutation, and compression were not performed.

Recommended next task: design a separately reviewed Phase 1B.2
evidence/modeling gap study and importer revision plan for explicit optional,
stale, dependency, semantic-zone, chronology, and evaluation-join metadata,
with privacy/licence and label-isolation gates. Do not begin that task as part
of this closeout.

