# Active Task — Phase 1A Corpus Gate Resolution and Rerun

Status: completed — PASS with recorded licence-file limitation.

## Objective

Establish a real agent-trajectory corpus whose licence and redistribution terms
are explicit for the actual artifact-bearing revision, then rerun the existing
Phase 1A protocol if a technically suitable corpus can be established.

This remains Phase 1A evidence gathering. It does not authorize Phase 1B or
Phase 1C work.

## Required context

Read only the relevant sections of:

- `../phase-1/PHASE_1_PLAN.md`
  - Phase 1A — Real-workload ingestion and observation
  - Phase boundaries
- `../phase-1/WORKLOAD_CORPUS.md`
  - corpus acceptance requirements
  - provenance requirements
  - evaluation leakage rule
  - Phase 1A exit condition
- `../phase-1/SUCCESS_CRITERIA.md`
  - Phase 1A pass
- `../phase-1/PHASE_1A_CORPUS_CLOSEOUT.md`
  - corpus/licence finding
  - existing technical spike
  - limitations
- `../phase-1/QUALITY_GATE.md`
  - evidence tiers
  - hard safety failures
  - fail-open principle

Use `../SOURCE_OF_TRUTH.md` and `../RESEARCH.md` only where needed.

Reuse the existing Phase 1A tools unless a narrowly scoped adapter change is
required.

## Existing evidence

The completed Tracebench spike established that the technical importer and
observer path works offline, but Phase 1A remained `INSUFFICIENT-EVIDENCE`
because redistribution terms for the artifact-bearing Tracebench revision were
not explicitly established.

The existing local Tracebench evidence remains ignored under:

`fixtures/phase-1a/tracebench-mini-swe-v1/`

Do not delete or commit it unless the corpus gate is explicitly resolved.

## Work

1. Recheck the artifact-bearing Tracebench source using authoritative upstream
   sources.

   Determine whether an explicit licence or redistribution statement applies
   to the actual trajectory artifacts at the checked revision.

   Record:
   - dataset/repository identity;
   - exact revision;
   - authoritative source;
   - retrieval date;
   - licence/redistribution statement;
   - whether the statement clearly applies to the trajectory artifacts.

2. Do not infer rights from:
   - the ContextBench repository licence;
   - another dataset owned by the same authors;
   - CodeTraceBench unless its licence explicitly applies to the exact
     artifact-bearing source being used;
   - similarity of project names, organisations or authors.

3. If Tracebench is explicitly cleared for the required use:

   - retain the existing pinned Phase 1A corpus path;
   - record any attribution, notice or redistribution requirements;
   - determine which generated evidence may safely be tracked;
   - rerun the existing Phase 1A protocol using the established source.

4. If Tracebench remains unresolved:

   - inspect no more than three credible public trajectory-corpus alternatives;
   - include current CodeTraceBench as a candidate if its current
     artifact-bearing revision is technically appropriate;
   - require an explicit licence/redistribution declaration applying to the
     actual trajectory artifacts;
   - select one corpus only.

5. A replacement corpus is acceptable only if it provides:

   - real multi-turn agent trajectories;
   - enough information to reconstruct ordered request/history context without
     fabricating absent fields;
   - stable task and trajectory identifiers;
   - an exact pinnable public revision;
   - explicit terms applicable to the artifacts used;
   - at least 20 suitable trajectories;
   - sufficient outcome/evaluation information for label-separated evaluation,
     where available.

6. If no corpus satisfies those conditions, stop and record
   `INSUFFICIENT-EVIDENCE`. Do not weaken the corpus requirements.

7. If a corpus is accepted:

   - use a deterministic 20–50 trajectory slice;
   - prefer the existing 24-case protocol when the source supports an
     equivalent deterministic selection;
   - preserve provenance;
   - keep evaluation/post-hoc labels outside observer inputs;
   - preserve unknown/absent context as unknown rather than fabricating it;
   - reuse `tools/phase1a_tracebench.py`,
     `tools/phase1a_run_observer.py`, and `tools/phase1a_report.py` where
     possible;
   - make only minimal corpus-adapter changes if needed.

8. Run the existing Prefixity observer offline over the accepted slice.

9. Record:

   - validation/analysis success;
   - deterministic observer structural candidates;
   - `DO_NOTHING` cases;
   - negative/non-useful post-hoc diagnostic cases where labels permit;
   - provenance and selection evidence;
   - limitations.

10. Do not interpret structural candidates as validated safe interventions,
    realised provider cache reuse, provider-token savings, cost savings or
    quality-preserving reductions.

## Required outputs

Produce repository-native evidence sufficient to determine whether the Phase 1A
corpus gate is now satisfied.

At minimum record:

- corpus decision and exact revision;
- authoritative licence/redistribution evidence;
- accepted/rejected candidate reasoning;
- deterministic slice definition if a corpus is accepted;
- importer/adapter changes if any;
- observer results if rerun;
- provenance and evaluation-boundary checks;
- concise Phase 1A assessment.

Update existing Phase 1 documentation rather than creating a parallel
documentation system.

Corpus-derived material must remain local-only unless its terms clearly permit
repository redistribution.

## Acceptance criteria

This task is complete when one of the following is established.

### PASS path

- one artifact-bearing trajectory corpus is identified at an exact revision;
- explicit licence/redistribution terms apply to the artifacts used;
- any attribution/notice obligations are recorded;
- a deterministic 20–50 trajectory slice is reproducible;
- provenance survives import;
- evaluation labels cannot influence observer decisions;
- the intended traces validate and analyse offline;
- observer evidence includes structural candidates and `DO_NOTHING`, plus
  negative/non-useful evaluation cases where supported;
- evidence and interpretation remain separate;
- only material permitted by the corpus terms is tracked;
- relevant tests/checks pass.

### INSUFFICIENT-EVIDENCE path

If no candidate has explicit applicable terms or no legally suitable candidate
can technically support the protocol:

- record the sources checked and exact reason each failed;
- retain existing local evidence;
- do not change the acceptance criteria;
- stop without beginning another phase.

## Stop conditions

Do not:

- begin Phase 1B pruning/compression;
- begin Phase 1C replay;
- perform live provider calls;
- contact dataset authors, open issues, or make external writes;
- infer licensing from related projects;
- redistribute corpus artifacts without explicit applicable terms;
- broaden into a general benchmark survey;
- redesign the Prefixity runtime;
- tune corpus selection to improve Prefixity results;
- claim realised cache/cost/quality benefits from structural observations;
- begin another task after completing this one;
- commit or push.

## Completion record

On completion, update this file with:

- sources/revisions checked;
- corpus decision;
- licence/redistribution evidence;
- work completed;
- corpus and slice used, if any;
- tests/checks run;
- evidence produced;
- Phase 1A `PASS`, `INSUFFICIENT-EVIDENCE`, or `PIVOT` assessment;
- remaining uncertainties;
- recommended next task.

Do not begin the recommended next task.

### Completion record — gate resolution

#### Sources and revisions checked

- `Contextbench/Tracebench`, revision
  `7da2e4f45b330be8b6e8f1cff835247723cb3341`: upstream metadata has no
  declared licence; rejected for redistribution.
- `NJU-LINK/CodeTraceBench`, revision
  `aa213b84ffb6690fc37ca15766d6ca174ec36d4d`, retrieved 2026-08-08: exact
  dataset metadata declares `mit`; the exact primary README describes
  `bench_artifacts/full/*.tar.zst` as compressed trajectory artifacts and
  states that the dataset is released under the MIT License. The exact root
  tree has no README-linked `LICENSE` file; this is recorded as a limitation,
  not filled by inference.
- The earlier CodeTraceBench revision
  `914de38100105c1ac21d9eb64a8134e32602d63c` was rejected for this rerun
  because its checked tree did not contain the artifact archives.
- No rights were inferred from ContextBench, Tracebench, CodeTracer, authors,
  organisations or related repositories.

#### Corpus/licence decision

CodeTraceBench at `aa213b84ffb6690fc37ca15766d6ca174ec36d4d` is accepted for
the Phase 1A derived-evidence rerun. Its own artifact-bearing dataset revision
contains the trajectory archives and makes the explicit MIT declaration. Only
sanitized derived evidence is retained in the repository; raw archives and
trajectory text are not tracked. The missing linked `LICENSE` file remains a
documented uncertainty.

#### Work completed

- Parameterized the existing
  [`tools/phase1a_tracebench.py`](../../tools/phase1a_tracebench.py) selector
  and importer for an explicit corpus, revision and split, preserving the
  existing turn mapping, provenance and label boundary.
- Reused [`tools/phase1a_run_observer.py`](../../tools/phase1a_run_observer.py)
  unchanged. The report tool was narrowly parameterized so its evaluation
  source label records the accepted corpus while preserving report behavior.
- Added exact source hashes and licence findings to
  `fixtures/phase-1a/codetracebench-mini-swe-v1/corpus-provenance.json`.
- Updated the Phase 1A report with the historical Tracebench rejection and
  the accepted CodeTraceBench rerun.

#### Corpus and slice used

- `NJU-LINK/CodeTraceBench`, revision
  `aa213b84ffb6690fc37ca15766d6ca174ec36d4d`, `verified` split.
- 1,000 verified manifest rows; 150 artifact-bearing `mini-SWE-agent` rows.
- Two missing-`.traj.json` rows were excluded and recorded; after
  recomputing rank bands, 24 trajectories were selected, four per solved ×
  short/medium/long cell.
- 1,498 source events and 719 one-request-per-assistant-turn traces.

#### Evidence/results produced

- `fixtures/phase-1a/codetracebench-mini-swe-v1/selection.json`
- `corpus-provenance.json`, `import-report.json`, `provenance/`, `traces/`
  and `evaluation/labels.json`
- `results/validation.json`, `results/analyses.json` and `results/report.json`
- Observer result: 719/719 validation and analysis successes; 712
  deterministic observer/adapter structural candidates, 7 `DO_NOTHING`, and
  0 review/errors.
- The 712 classifications are candidates only—not validated safe
  interventions, provider cache savings, provider-token savings, cost
  savings or quality-preserving reductions.

#### Tests/checks run

- Python compilation passed for all three Phase 1A tools.
- The existing offline observer processed all 719 accepted traces.
- Local report generation passed and reproduced the 712/7 result split.
- `C:\Users\USER\.cargo\bin\cargo.exe test --workspace` passed with 205
  tests and 0 failures.
- Deterministic selection/import evidence and raw-marker scan passed; labels
  remained outside observer inputs.
- `git diff --check` and final ignore/status checks passed.
- No live provider calls, replay, mutation or provider profile was used.

#### Phase 1A assessment

`PASS` for the corpus-gate resolution task, with the missing README-linked
`LICENSE` file retained as a licence-evidence limitation. The accepted source
has explicit primary dataset-level MIT terms applicable to the artifact-bearing
dataset; the tracked evidence is sanitized derived material only.

#### Remaining uncertainties

- The exact CodeTraceBench revision’s README links to a `LICENSE` file that is
  absent from the root tree; its text was not reconstructed or inferred.
- No raw artifact redistribution is performed by this repository.
- Structural candidates remain heuristic observer output. No provider cache
  reuse, cost, latency, replay quality or causal intervention benefit was
  measured.
- The two missing source-format rows remain excluded by the deterministic
  selection definition.

#### Recommended next task

Separately authorize Phase 1B decision-layer design/review against the Phase 1
plan, treating the 712 candidates as structural observations only. Do not
begin Phase 1B as part of this completion.
