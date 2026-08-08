# Active Task

Status: final context-efficiency review complete; no product implementation
started and no Phase 1A work started.

## Recommended next task

Run the offline Phase 1A corpus-validation spike described by:

- [`phase-1/PHASE_1_PLAN.md`](../phase-1/PHASE_1_PLAN.md), section **Phase 1A -
  Real-workload ingestion and observation**, plus **Phase boundaries**;
- [`phase-1/WORKLOAD_CORPUS.md`](../phase-1/WORKLOAD_CORPUS.md), sections
  **Primary candidate: ContextBench**, **Initial slice**, **Corpus acceptance
  checklist**, **Provenance requirements**, **Evaluation leakage rule** and
  **Phase 1A exit condition**;
- [`phase-1/QUALITY_GATE.md`](../phase-1/QUALITY_GATE.md), sections **Evidence
  tiers**, **Hard safety failures** and **Fail-open principle**; and
- [`phase-1/SUCCESS_CRITERIA.md`](../phase-1/SUCCESS_CRITERIA.md), section
  **Phase 1A pass**.

The task should verify the exact corpus revision, licence and redistribution
terms; select a representative 20-50-task slice; preserve task/trajectory and
source-event provenance; keep evaluation labels out of decision inputs; run
the existing observer offline; and report positive, negative and `DO_NOTHING`
cases. Do not begin Phase 1B pruning/compression or Phase 1C replay.

## Current validation record

- `C:\Users\USER\.cargo\bin\cargo.exe fmt --all -- --check` passed.
- `C:\Users\USER\.cargo\bin\cargo.exe test --workspace` passed: all
  workspace suites passed with no failures.
- `C:\Users\USER\.cargo\bin\cargo.exe clippy --workspace --all-targets
  --all-features -- -D warnings` passed.
- Checks emitted only the non-fatal warning that `D:\Users\fleur` could not be
  canonicalized. No live provider command was run.
