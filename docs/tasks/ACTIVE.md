# Active Task — Phase 1A Corpus Gate Resolution and Rerun

Status: ready for implementation.

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
- `../phase-1/PHASE_1A_TRACEBENCH_SPIKE.md`
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