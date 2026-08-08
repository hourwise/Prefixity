# Phase 1A Tracebench Corpus Validation Spike

Status: `INSUFFICIENT-EVIDENCE` for Phase 1A acceptance.

This report records the completed offline spike. It does not authorize Phase
1B pruning/compression or Phase 1C replay.

## Corpus and licence record

The preferred primary candidate was checked first:

| Item | Exact record |
| --- | --- |
| ContextBench source | [`EuniAI/ContextBench`](https://github.com/EuniAI/ContextBench/tree/1436c28a8eb95496da4ea69ad458b9f8a8eb7d61) at current `main` commit `1436c28a8eb95496da4ea69ad458b9f8a8eb7d61`, rechecked 2026-08-08 |
| ContextBench dataset card | [`Contextbench/ContextBench`](https://huggingface.co/datasets/Contextbench/ContextBench/tree/c2855792b006af41c67202d33883fb9d46362853) at dataset-card revision `c2855792b006af41c67202d33883fb9d46362853` |
| ContextBench licence | Apache-2.0, as declared by the upstream repository/card |
| ContextBench shape at the checked revision | Task/gold-context table: task identifiers, repository/base commit, problem statement, patch/test patch and gold context; it does not provide the released trajectory objects needed for this trace importer |
| Tracebench artifact-bearing source | [`Contextbench/Tracebench`](https://huggingface.co/datasets/Contextbench/Tracebench/tree/7da2e4f45b330be8b6e8f1cff835247723cb3341) at dataset revision `7da2e4f45b330be8b6e8f1cff835247723cb3341` |
| Tracebench checked status | The upstream dataset metadata/card describes trajectory artifacts and the selected verified manifest. The checked dataset metadata has no declared `license` field. |
| Separate MIT claim checked | [`NJU-LINK/CodeTraceBench`](https://huggingface.co/datasets/NJU-LINK/CodeTraceBench/tree/914de38100105c1ac21d9eb64a8134e32602d63c) at revision `914de38100105c1ac21d9eb64a8134e32602d63c` declares MIT, but this checked revision contains manifests/reports rather than the Tracebench artifact archives. Its MIT declaration is not treated as a licence for the Tracebench archives. |
| Retrieval date | 2026-08-08 |

The primary ContextBench repository is explicitly licensed, but its released
dataset shape did not meet the trajectory-ingestion requirement. The earlier
`b3b9236db44383739f31d21a06492df0cb7da927` value was the last main-branch SHA
visible during the initial check; it was not intentionally pinned as the
source revision for the HF dataset-card revision. The HF revision
`c2855792b006af41c67202d33883fb9d46362853` is an independent dataset-card
snapshot (last modified 2026-01-23), while `b3b9236` was committed 2026-02-11
and current `main` is `1436c28` from 2026-06-12. No direct repository-commit to
HF-dataset-revision correspondence was established, so the repository and
dataset-card revisions are recorded independently.

Tracebench was technically usable after local extraction, but the
artifact-bearing revision did not state redistribution terms. Therefore no
legal permission to commit or redistribute raw Tracebench content is inferred.
The generated Tracebench evidence remains available locally but is ignored by
Git under `/fixtures/phase-1a/tracebench-mini-swe-v1/`; only the importer,
reports and documentation are candidates for commit. Raw archives and
extracted trajectory files remain outside the repository.

## Slice and transformation

The selected slice is the `verified` split from Tracebench, filtered to
`mini-SWE-agent` rows with both `artifact_path` and `source_relpath`. The
adapter sorts by `(step_count, traj_id)`, assigns equal-ranked `short`,
`medium` and `long` bands, then chooses four evenly spaced trajectories in each
`solved × length-band` cell:

- population after artifact-format preflight: 148 rows;
- selected: 24 trajectories, four per each of six cells;
- exclusions recorded in the selection: `miniswe-Anthropic__Claude-Sonnet-4-20250514-Thinking-hf-model-inference-f200d460` and `miniswe-Anthropic__Claude-Sonnet-4-20250514-Thinking-play-zork-easy-8c77a548`, because their selected archives did not contain the expected `.traj.json` source representation;
- source events: 1,498;
- observer request traces: 719, one per recorded assistant turn, with the response itself excluded from that request context.

The deterministic selector and importer are [`tools/phase1a_tracebench.py`](../../tools/phase1a_tracebench.py).
Each source event becomes an ordered hash-only block. The adapter records the
trajectory ID, task, model, original message index, source path, content hash,
byte count, role, source classification and transformation metadata. Content is
not retained. `system` becomes `system_policy`; ordinary user content becomes
`user_request`; assistant history becomes `conversation`; mini-SWE shell
observations identified by source-format `<returncode>`/`<output>` markers
become `tool_result`. The adapter does not manufacture `required`, `optional`,
`stale` or dependency assertions.

The imported evidence is under
[`fixtures/phase-1a/tracebench-mini-swe-v1`](../../fixtures/phase-1a/tracebench-mini-swe-v1):

- [`selection.json`](../../fixtures/phase-1a/tracebench-mini-swe-v1/selection.json) — pinned slice and exclusions;
- [`import-report.json`](../../fixtures/phase-1a/tracebench-mini-swe-v1/import-report.json) — counts and input-boundary assertions;
- [`provenance/trajectory-summaries.json`](../../fixtures/phase-1a/tracebench-mini-swe-v1/provenance/trajectory-summaries.json) — trajectory-level provenance and turn counts;
- [`provenance/source-events.jsonl`](../../fixtures/phase-1a/tracebench-mini-swe-v1/provenance/source-events.jsonl) — one sanitized provenance row per source event;
- [`evaluation/labels.json`](../../fixtures/phase-1a/tracebench-mini-swe-v1/evaluation/labels.json) — evaluation-only solved/stage/step IDs and labels, with action/observation text omitted;
- `traces/<trajectory-id>/turn-*.json` — hash-only request/turn traces.

## Offline observer evidence

The unchanged CLI observer was built and run locally using
[`tools/phase1a_run_observer.py`](../../tools/phase1a_run_observer.py). The
machine-readable outputs are:

- [`results/validation.json`](../../fixtures/phase-1a/tracebench-mini-swe-v1/results/validation.json);
- [`results/analyses.json`](../../fixtures/phase-1a/tracebench-mini-swe-v1/results/analyses.json);
- [`results/report.json`](../../fixtures/phase-1a/tracebench-mini-swe-v1/results/report.json), generated by [`tools/phase1a_report.py`](../../tools/phase1a_report.py).

Measured observer results:

| Measure | Result |
| --- | ---: |
| Request traces processed | 719 |
| Validation successes | 719/719 |
| Analysis successes | 719/719 |
| Deterministic observer/adapter `INTERVENTION_CANDIDATE` classifications | 712 |
| `DO_NOTHING` recommendations | 7 |
| `REVIEW`/error results | 0 |
| Estimated input tokens | 13,704,473 |
| Estimated volatile tokens | 6,888,133 |
| Estimated stable-prefix candidate tokens | 6,816,340 |

The token numbers are the adapter's deterministic surrogate
`ceil(canonical_event_chars / 4)` counts, not provider-token usage. The
recommendation text itself retains the core's warning that a single-trace
analysis cannot prove cache reuse.

Examples are recorded in `results/report.json`:

- deterministic observer/adapter candidates: 712 requests were classified
  from the existing observer recommendation; representative examples include
  the first three `build-pmars` turns. These are not validated safe
  interventions, provider cache savings, or quality-preserving reductions;
  they are offline structural candidates only;
- negative/non-useful diagnostic cases: representative intervention candidates
  in `install-windows-xp` occur in a trajectory with three post-hoc
  `incorrect` step IDs. This is an evaluation-only correlation, not evidence
  that Prefixity caused the outcome;
- no-op cases: seven requests received the existing
  `no structural change recommended` recommendation, including later
  `git-workflow-hack` turns.

The report explicitly records that evaluation labels were not passed to the
observer, provider cache evidence is absent, realised provider savings are
unmeasured, and no live provider calls were made. No mutation or replay was
performed. The 712 classifications must not be interpreted as validated safe
actions or quality-preserving reductions.

## Acceptance assessment

| Criterion | Assessment | Evidence |
| --- | --- | --- |
| Exact corpus revision identifiable | Met | Selection and source table above |
| Licence/redistribution status explicit | Met as a finding; not cleared | Tracebench artifact revision has no declared licence, so redistribution remains unresolved |
| Reproducible 20–50 task slice | Met | 24 rows, six deterministic cells, exclusions recorded |
| Provenance survives import | Met | Sanitized source-event ledger and trajectory summaries |
| Labels cannot influence observer decisions | Met by boundary | Labels are a separate file; traces contain `evaluation_labels_excluded: true` |
| Existing observer processes slice offline | Met | 719/719 validation and analysis successes |
| Positive, negative/non-useful and `DO_NOTHING` cases | Met as observer/evaluation evidence | Counts and examples in `results/report.json`; candidates are not validated interventions and negative cases are post-hoc only |
| Evidence separated from interpretation | Met | Report distinguishes measured counts, heuristic estimates and limits |
| Legal/public-corpus condition for a Phase 1A pass | Not established | Tracebench artifact redistribution terms are unresolved |

Overall Phase 1A assessment: `INSUFFICIENT-EVIDENCE`. The technical spike is
reproducible and the observer produces all three requested case categories,
but the corpus licence gate is not satisfied. This result does not weaken the
acceptance requirement and does not support claims about realised caching,
cost, latency or task-quality improvement.

## Reproduction

The following commands assume the pinned manifest and extracted artifacts have
been obtained under terms that permit local use. The manifest and raw archives
are intentionally not checked in:

```text
python tools/phase1a_tracebench.py select --manifest <verified-manifest.jsonl> --out <selection.json> --count-per-cell 4 --exclude-traj-id <id> --exclude-traj-id <id>
python tools/phase1a_tracebench.py import --manifest <verified-manifest.jsonl> --selection <selection.json> --raw-root <indexed-extracted-root> --archive-root <archive-root> --out-dir fixtures/phase-1a/tracebench-mini-swe-v1 --replace
python tools/phase1a_run_observer.py --binary target/debug/prefixity --trace-root fixtures/phase-1a/tracebench-mini-swe-v1/traces --out-dir fixtures/phase-1a/tracebench-mini-swe-v1/results
python tools/phase1a_report.py --analyses fixtures/phase-1a/tracebench-mini-swe-v1/results/analyses.json --labels fixtures/phase-1a/tracebench-mini-swe-v1/evaluation/labels.json --out fixtures/phase-1a/tracebench-mini-swe-v1/results/report.json
```

Recommended next task, not started: resolve the public trajectory corpus
licence/redistribution gate (or select a trajectory corpus with explicit terms)
and rerun the same Phase 1A protocol, retaining the no-op and post-hoc negative
case checks.
