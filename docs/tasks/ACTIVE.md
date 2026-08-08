# Active Task — Phase 1A Corpus Validation Spike

Status: completed — insufficient evidence for Phase 1A pass.

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

### Work completed

- Added a standard-library-only Tracebench selector/importer at
  [`tools/phase1a_tracebench.py`](../../tools/phase1a_tracebench.py).
- Added reproducible offline observer and evidence-summary runners at
  [`tools/phase1a_run_observer.py`](../../tools/phase1a_run_observer.py) and
  [`tools/phase1a_report.py`](../../tools/phase1a_report.py).
- Preserved a complete sanitized source-event ledger while emitting one
  hash-only request trace per recorded assistant turn; response messages are
  excluded from their own request context.
- Kept evaluation labels in a separate sanitized file containing IDs/labels
  only. No evaluation label is present in the observer trace input.
- Added the concise evidence report at
  [`docs/phase-1/PHASE_1A_TRACEBENCH_SPIKE.md`](../phase-1/PHASE_1A_TRACEBENCH_SPIKE.md)
  and indexed it in [`docs/INDEX.md`](../INDEX.md).

### Corpus and slice used

- Primary ContextBench check: current repository `main` commit
  `1436c28a8eb95496da4ea69ad458b9f8a8eb7d61`, rechecked 2026-08-08;
  dataset-card revision `c2855792b006af41c67202d33883fb9d46362853`;
  Apache-2.0. The previously recorded
  `b3b9236db44383739f31d21a06492df0cb7da927` was only the last SHA visible
  during the initial check, not an intentional pin for the HF dataset
  revision. No direct repository/dataset-revision correspondence was
  established, so the revisions are recorded independently. The checked
  dataset shape is a task/gold-context table, not the required released
  trajectory objects.
- Technical secondary slice: `Contextbench/Tracebench`, verified split,
  dataset revision `7da2e4f45b330be8b6e8f1cff835247723cb3341`, retrieved
  2026-08-08. The artifact-bearing revision has no declared licence field.
- `mini-SWE-agent` only; 148 artifact-bearing rows after two recorded
  missing-`.traj.json` preflight exclusions; 24 selected tasks, four per
  solved × short/medium/long cell; 1,498 source events and 719 request traces.
- Raw archives/extracted trajectory files were not added to the repository.
  Only hashes, metadata, provenance, sanitized evaluation IDs/labels and
  observer outputs were retained.

### Tests/checks and evidence

- `python -m py_compile tools/phase1a_tracebench.py tools/phase1a_run_observer.py tools/phase1a_report.py`
- `C:\Users\USER\.cargo\bin\cargo.exe build -p prefixity-cli`
- `C:\Users\USER\.cargo\bin\cargo.exe test --workspace` — all workspace tests passed.
- A second import regenerated 724 non-result evidence files with the same file
  set and SHA-256 hashes as the committed fixture output.
- Existing CLI `validate --json`: 719/719 succeeded offline.
- Existing CLI `analyse --json`: 719/719 succeeded offline.
- Observer result: 712 deterministic observer/adapter `INTERVENTION_CANDIDATE`
  classifications, 7 `DO_NOTHING`, 0 `REVIEW`/errors. The 712 are structural
  candidates only—not validated safe interventions, provider cache savings,
  or quality-preserving reductions.
- Evidence files: `fixtures/phase-1a/tracebench-mini-swe-v1/selection.json`,
  `import-report.json`, `provenance/`, `evaluation/labels.json`,
  `results/validation.json`, `results/analyses.json` and `results/report.json`.
- No live provider calls, provider profile, replay, mutation or cache/cost
  result was used. Token figures are deterministic surrogate estimates, not
  provider usage.
- `git diff --check` passed, and a raw-marker scan found no retained
  `action_ref`, `observation_ref`, `THOUGHT`, `qemu-img` or
  `reasoning_content` trajectory text in the committed fixture/report paths.

### Phase 1A assessment

`INSUFFICIENT-EVIDENCE`. The technical import, provenance, label boundary,
offline observer processing, deterministic slice and all three requested
observer/evaluation case categories were produced. The 712 intervention
classifications are deterministic observer/adapter candidates only; they are
not validated safe interventions, provider cache savings, or
quality-preserving reductions. The acceptance gate for a public corpus with
verified redistribution terms is unresolved because the Tracebench artifact
revision does not declare a licence; the separate CodeTraceBench MIT
manifest/report revision does not establish rights for those artifacts. This
is recorded as a result rather than a relaxation of the acceptance criteria.

### Remaining uncertainties

- Whether Tracebench artifact redistribution is permitted under terms not
  visible in the pinned dataset metadata.
- The observer produces heuristic structural candidates only; no provider
  cache reuse, monetary savings, latency, replay quality or causal harm was
  measured.
- Negative/non-useful examples are post-hoc manifest-label correlations and
  were not observer inputs.
- The two missing selected artifact representations are excluded and the
  exclusion is part of the reproducible slice definition.

### Recommended next task

Resolve the trajectory-corpus licence/redistribution gate (or select a public
trajectory corpus with explicit terms) and rerun this same Phase 1A protocol,
including no-op and post-hoc negative-case coverage. Do not begin that task as
part of this completion.

### Pre-commit correction review

- Rechecked the public ContextBench ref with `git ls-remote`: current
  `main` is `1436c28a8eb95496da4ea69ad458b9f8a8eb7d61`. The former
  `b3b9236db44383739f31d21a06492df0cb7da927` was not intentionally tied to
  HF revision `c2855792b006af41c67202d33883fb9d46362853`; the two revisions
  are now documented as independent snapshots.
- Added a Git ignore rule for
  `/fixtures/phase-1a/tracebench-mini-swe-v1/`. The 727-file generated local
  evidence remains present and was not deleted; it is not prepared for
  commit while Tracebench redistribution terms are unresolved.
- Clarified in the report and this record that the 712 classifications are
  deterministic observer/adapter candidates only—not validated safe
  interventions, provider cache savings, or quality-preserving reductions.
- Re-ran Python compilation, the local report summary, and
  `cargo test --workspace` (205 passed, 0 failed); `git diff --check` and the
  ignore-rule/status checks also passed.
