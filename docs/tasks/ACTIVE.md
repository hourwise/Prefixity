# Active Task — Phase 1A Corpus Validation Spike

Status: ready for implementation.

## Objective

Test whether Prefixity's existing offline observer produces useful and
defensible observations on a small slice of a real workload corpus.

This is an evidence-gathering task, not a runtime/product-expansion task.

## Required context

Read only the relevant sections of:

- `../phase-1/PHASE_1_PLAN.md`
  - Phase 1A — Real-workload ingestion and observation
  - Phase boundaries
- `../phase-1/WORKLOAD_CORPUS.md`
  - Primary candidate: ContextBench
  - Initial slice
  - Corpus acceptance checklist
  - Provenance requirements
  - Evaluation leakage rule
  - Phase 1A exit condition
- `../phase-1/QUALITY_GATE.md`
  - Evidence tiers
  - Hard safety failures
  - Fail-open principle
- `../phase-1/SUCCESS_CRITERIA.md`
  - Phase 1A pass

Follow `../SOURCE_OF_TRUTH.md` and `../RESEARCH.md` only where needed to
resolve project-state or evidence questions.

## Work

1. Verify and record the exact selected corpus revision.
2. Verify licence and redistribution terms before importing corpus material.
3. Select a representative 20–50 task slice using a deterministic,
   documented selection method.
4. Preserve provenance from corpus source through task, trajectory and
   source-event representation.
5. Keep evaluation labels and other post-hoc information out of Prefixity
   decision inputs.
6. Import/represent the selected trajectories deterministically.
7. Run the existing Prefixity observer offline over the selected slice.
8. Record examples covering:
   - positive intervention candidates;
   - negative/non-useful candidates;
   - `DO_NOTHING` decisions.
9. Report observed evidence without converting structural reuse potential
   into claims of realised provider cache reuse or monetary savings.

## Required outputs

Produce repository-native evidence sufficient to reproduce the spike,
including:

- corpus identity/revision and licence/provenance record;
- deterministic slice definition;
- imported trajectory/source-event representation;
- observer output/results;
- concise Phase 1A findings.

Reuse existing project structures where suitable rather than creating
parallel documentation systems.

## Acceptance criteria

The task is complete when:

- the selected corpus and exact revision are identifiable;
- licence/redistribution status is explicitly recorded;
- selection of the 20–50 task slice is reproducible;
- provenance survives import;
- evaluation labels cannot influence observer decisions;
- the existing observer successfully processes the intended slice offline;
- results include positive, negative and `DO_NOTHING` cases;
- evidence and interpretation are clearly distinguished;
- relevant tests/checks pass; and
- results are sufficient to assess the documented Phase 1A pass criteria.

If the corpus cannot legally or technically satisfy the required conditions,
record that as a Phase 1A result rather than weakening the requirements.

## Stop conditions

Do not:

- begin Phase 1B pruning/compression;
- begin Phase 1C replay;
- perform live provider calls;
- claim realised cache savings from structural observations;
- tune corpus selection to improve Prefixity results;
- redesign the Prefixity runtime;
- begin another phase after completing this task.

## Completion record

On completion, update this file with:

- work completed;
- corpus and slice used;
- tests/checks run;
- evidence produced;
- Phase 1A pass/fail/insufficient-evidence assessment;
- remaining uncertainties;
- recommended next task.

Do not begin the recommended next task.