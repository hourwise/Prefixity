# Phase 1 Quality Gate

## Core rule

> An intervention is acceptable only when end-to-end task quality remains within the predeclared acceptable envelope and required/protocol constraints remain valid.

Token reduction is secondary.

## TaskSuccess model

```text
TaskSuccess
├── task_completed
├── expected_result_obtained
├── required_tests_or_checks
├── required_tool_outcomes
├── required_facts_or_state_retained
├── forbidden_regressions
├── protocol_valid
├── recovery_required
└── evaluator_confidence / review_status
```

Unknown remains unknown; it is never silently treated as pass.

## Evidence tiers

### Tier 0 — structural safety

Required blocks retained, dependency closure retained, chronology valid, tool relationships valid, no illegal relocation, request valid.

Tier 0 cannot prove task quality.

### Tier 1 — external gold-context evaluation

Measure gold-context recall/precision where meaningful plus protocol/dependency-required retention.

### Tier 2 — deterministic task checks

Prefer builds, tests, expected file changes, expected command results or benchmark scorers.

### Tier 3 — model/human semantic evaluation

Use only when deterministic checks are insufficient. Record evaluator prompt/version, use the same procedure for baseline/intervention, retain uncertainty, and do not rely on semantic judging alone for protocol/security-critical preservation.

## Hard safety failures

Any of these fails an intervention regardless of savings:

- required block removed;
- protocol invalidated;
- dependency closure broken;
- baseline pass becomes intervention fail;
- forbidden regression introduced;
- safety/security instruction removed contrary to policy;
- critical preservation cannot be determined.

## Intervention rules

`DO_NOTHING` is successful when evidence is weak, context is already efficient, quality risk outweighs predicted savings, or cache economics make mutation unattractive.

`DEFER` must account for later rereads.

`PRUNE` requires stronger evidence than `DEFER`; non-gold is not automatically removable.

`RELOCATE_CANDIDATE` must preserve zones, chronology, protocol and dependencies.

`COMPRESS_CANDIDATE` is highest risk and remains gated until simpler deterministic reductions are evaluated and cache disruption is modelled.

## Primary evaluation rule

```text
INTERVENTION_PASS =
    quality_within_gate
    AND protocol_valid
    AND required_context_preserved
    AND end_to_end_efficiency_improves
```

End-to-end efficiency includes fresh/total input, cache accounting, output, turns, tool calls, rereads, latency and cost.

## Avoid aggregate-score traps

Report baseline-pass → intervention-fail regressions individually, improvements, worst regression, savings distribution, negative-ROI cases, reread/recovery cases, `DO_NOTHING` cases and inconclusive cases.

## Early directional requirements

Before Phase 1C numerical thresholds are frozen:

- zero known required/protocol-block removals in offline tests;
- baseline-pass → intervention-fail is critical evidence;
- savings claims must be end-to-end;
- quality must be non-inferior within a predeclared task-specific tolerance.

Freeze numerical thresholds **before** controlled replay.

## Replay controls

Predeclare task subset, baseline, intervention, model/provider/settings, request/turn limits, spend limits, timeout, evaluator/scorer, thresholds, retries, cache assumptions and stopping conditions.

Do not repeat experiments just to obtain preferable numbers.

## Fail-open principle

Original context must remain recoverable; transformations disable cleanly; Prefixity failure must not destroy the original request; low-confidence decisions default toward retention/no-op.
