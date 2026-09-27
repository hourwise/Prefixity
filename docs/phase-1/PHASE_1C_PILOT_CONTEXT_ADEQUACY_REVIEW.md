# Phase 1C pilot context adequacy review

Status: `PILOT_CONTEXT_INADEQUATE` — offline review complete

```text
LOCAL_9B_FEASIBILITY_RESULT_ACCEPTED
LOCAL_9B_FEASIBILITY_PASSED
LOCAL_9B_INSTRUMENT_ACCEPTED
EXISTING_DEPENDENCY_SAFETY_BATTERY_RETAINED
EXISTING_CLAIM_2_PILOT_WORKLOAD_REJECTED
PILOT_CONTEXT_INADEQUATE
CLAIM_2_CONTEXT_PRESSURE_WORKLOAD_DESIGN_ONLY
```

This review follows the accepted local Qwen3.5-9B feasibility result. It
assesses the already specified six-case pilot for meaningful Claim-2 context
pressure. It records a workload adequacy finding; it does not redesign or
implement a workload, prepare a scored pilot, or authorize inference.

## Evidence

### Measuring instrument

The accepted instrument is Qwen3.5-9B Q4_K_M with reasoning off, context
8192, `max_tokens` 1024, temperature 0, `top_p` 1, and seed 1. The sealed
feasibility gate passed all three frozen structural probes and one h001
BASELINE request. The h001 request returned task success, required-context
recall 1.0, a valid dependency protocol, and zero critical regressions. Its
measured input was 391 tokens. The gate result accepts this model as a
competent Claim-2 measuring instrument for the frozen probes; it is not a
Prefixity efficacy or context-reduction result.

Instrument competence and workload adequacy answer separate questions. The
accepted 9B result does not establish that the proposed pilot gives Prefixity
enough accumulated context to reduce, and the workload finding below does not
reverse the instrument result.

### Existing pilot specification and implementation

The nominal pilot has six cases, each assigned `BASELINE → NO_OP →
INTERVENTION`:

| Case | Frozen action | Target | Existing role |
| --- | --- | --- | --- |
| h001 | `PRUNE` | e002 | Exact duplicate removal with a required explicit dependency |
| h004 | `PRUNE` | e003 | Duplicate with a distractor |
| h006 | `DO_NOTHING` | — | Hidden load-bearing dependency control |
| h007 | `DEFER` | e001 | Explicit supersession with protocol order |
| h009 | `RELOCATE_CANDIDATE` | e002 | Same-zone referenced-result relocation |
| h010 | `DO_NOTHING` | — | Protocol/dependency conflict control |

Only h001 is currently materialized as a runnable, model-visible scored task.
It is one prompt and one inference request per arm. The other five cases are
structural traces; they do not have implemented multi-turn model trajectories.
The declared `MAX_TURNS = 3` is an upper bound. The manifest's 54-request
formula (`6 cases × 3 arms × 1 replicate × 3 turns`) is arithmetic over that
ceiling, not an implemented three-turn efficacy workload.

The positive cases still provide useful dependency and policy checks across
PRUNE, DEFER, RELOCATE_CANDIDATE, and DO_NOTHING behavior. Retain all six as a
`DEPENDENCY_SAFETY_BATTERY`. They do not form an adequate primary Claim-2
efficacy workload in their current form.

### Context pressure in the runnable task

The measured 391-token h001 input contains a short task and three event
records. Its intervention removes one small duplicate event record while
preserving the explicitly referenced event. This is a local duplicate-removal
check; it does not represent meaningful long-horizon accumulated context.

h009 relocates a referenced item without removing it, so the operation removes
zero context. It is useful for checking dependency-safe relocation, but is not
primary context-reduction evidence. Any unmeasured reduction quantities
remain estimates; this review supplies no additional measured reduction
figures.

## Interpretation

The existing pilot has no implemented multi-turn context accumulation and no
material long-horizon context growth. Its currently runnable scored task
offers only a small duplicate-removal opportunity. Therefore the pilot's
context is inadequate to test whether Prefixity reduces accumulated context
while preserving task quality, even though the measuring instrument passed
its competence gate.

The current Claim-2 efficacy workload is rejected. The six fixed cases are
retained as a dependency-safety battery, not discarded or re-scored as
efficacy evidence. No model capability conclusion follows from the workload
finding.

## Boundary for the next task

The only next authorized task is:

```text
CLAIM_2_CONTEXT_PRESSURE_WORKLOAD_DESIGN
```

That design task may consider plausible accumulated task history, including
repeated identical tool or file results, superseded state snapshots, stale
results from abandoned branches, and reference information legitimately
deferred until later. It must avoid meaningless prompt padding. No workload
design or implementation is performed here.

The LOW/MODERATE/HIGH pressure thresholds, a 5/6 rule, an 80% rule, a minimum
of eight positive cases, and any final BASELINE competence threshold remain
future design inputs. This review freezes none of them. No scored pilot is
prepared or authorized, and no inference is authorized.

## Records reviewed

- `docs/phase-1/PHASE_1C_LOCAL_9B_FEASIBILITY_GATE_EXECUTION_RECORD.md` —
  accepted instrument, sealed gate result, and measured h001 input count.
- `docs/phase-1/PHASE_1C_CAPABLE_MODEL_DESIGN_DECISION.md` — Claim-2 instrument
  rationale and deferred adequacy criteria.
- `docs/phase-1/PHASE_1C_SCORED_PILOT_MANIFEST_V1.json` and
  `docs/phase-1/PHASE_1C_SCORED_PILOT_MANIFEST_V2.json` — six-case roster,
  three-arm order, turn ceiling, and request formula.
- `docs/phase-1/PHASE_1C_H001_SCORED_TASK_V1.json` and
  `docs/phase-1/PHASE_1C_H001_ARM_PROJECTIONS_V1.json` — runnable h001 task
  and its duplicate-removal intervention.
- `crates/prefixity-controlled-benchmark/src/phase1c_h001.rs` — current h001
  runner boundary and one-request-per-arm accounting.

No model server was started, no port 8080 or token-count endpoint was
contacted, no inference was performed, and no sealed evidence or runtime
implementation was changed.
