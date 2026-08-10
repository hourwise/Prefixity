# ContextBench Front-Half External Evidence

## 1. Outcome

**NO-GO — BENCHMARK ADMISSION/TRAJECTORY INSUFFICIENT**

The study stopped at the mandatory provenance/admission preflight. The pinned
ContextBench material contains gold-context/task metadata and an evaluator,
but it does not provide a permission-cleared external agent trajectory joined
to those gold instances. No adapter, sample, baseline, or gold-scored result
was created.

This is a benchmark-evidence stop, not a claim that Prefixity's deterministic
front half failed on a valid external sample.

## 2. Authoritative Prefixity checkpoint

- Repository: `hourwise/Prefixity`
- Checkpoint: `975a1ae12dcdfb590814b97711f54b39e2bafca1`
- CI #36: successful
- Production planner: unchanged
- Phase 1B.9 policy: `controlled-evidence-policy-v1`, research-only
- Phase 1C Stage 0: certified, but Stage 1 remains separately blocked

## 3. Benchmark identity and pins

- Repository: [`EuniAI/ContextBench`](https://github.com/EuniAI/ContextBench)
- Repository revision:
  `1436c28a8eb95496da4ea69ad458b9f8a8eb7d61`
- Paper: [arXiv 2602.05892](https://arxiv.org/abs/2602.05892)
- Dataset: [`Contextbench/ContextBench`](https://huggingface.co/datasets/Contextbench/ContextBench)
- Dataset revision: `c2855792b006af41c67202d33883fb9d46362853`

The corrected repository is the only ContextBench source used in this
review. No same-name alternative repository was used as a benchmark,
license, or provenance basis.

## 4. Provenance and admission results

The pinned repository declares Apache License 2.0. That license covers the
repository's own code and documentation as licensed works; it does not, by
itself, grant redistribution rights for third-party issue text, patches,
test patches, source files, repositories, or agent trajectories.

The observed Hugging Face dataset metadata exposes no dataset license
declaration. Its documented schema contains instance IDs, repository URLs,
base commits, `gold_context`, `patch`, `test_patch`, `problem_statement`,
source-family fields, and related benchmark metadata. It contains no
trajectory field. The public page also presents conflicting row summaries,
so the repository README's 1,136 published task count is used for the
bounded preflight count rather than treating the display summary as a new
revision.

The four source families require separate treatment:

| Family | Observed provenance basis | Local disposition |
| --- | --- | --- |
| `Verified` | SWE-bench Verified; the official SWE-bench project identifies an MIT code license and tasks from third-party open-source repositories | No raw redistribution inferred; no trajectory join |
| `Pro` | SWE-bench Pro family; task and source-repository rights are source-specific | No blanket permission inferred; no trajectory join |
| `Poly` | [Amazon Science SWE-PolyBench](https://github.com/amazon-science/SWE-PolyBench), whose project README identifies MIT licensing | Code license does not settle third-party task/source rights; no trajectory join |
| `Multi` | [Multi-SWE-bench](https://github.com/multi-swe-bench/multi-swe-bench), whose project README identifies Apache-2.0 project licensing and acknowledges SWE-bench provenance | Project license is not a blanket grant over every underlying repository/task; no trajectory join |

Therefore the admission classification remains:

`ADMISSIBLE WITH PROVENANCE RESTRICTIONS`

This means a bounded local study might be possible after a source-specific
provenance and trajectory review. It does not authorize copying, vendoring,
or redistributing the raw dataset or source repositories inside Prefixity.

## 5. Candidate, admitted, and excluded counts

- Repository-published task candidates: **1,136**
- Trajectory-joined candidates: **0**
- Admitted instances: **0**
- Excluded instances: **1,136**
- Target sample: **50**
- Minimum sample for an external claim: **25**
- Actual scored sample: **0**

The repository's selected-500 metadata CSVs were inspected only for bounded
source-family counts: Verified 174, Pro 54, Poly 116, and Multi 156. No raw
row body, patch, or source excerpt was copied into Prefixity.

## 6. Trajectory source and availability

The repository's evaluator documents external input formats including
MiniSWE `.traj.json`, SWE-agent checkpoint/trajectory formats, Agentless
outputs, Prometheus logs, and OpenHands output. Those are parser contracts,
not supplied trajectory evidence. A pinned-revision scan found no bundled
benchmark trajectory artifact, and the dataset schema has no trajectory-like
field.

The only way identified to create the missing material would be to run an
agent or otherwise obtain a separately supplied trajectory artifact. That
would require a separate permission/provenance decision and, for generated
trajectories, model/provider execution. It was not done.

## 7. Preregistration

The frozen design is documented in
[`CONTEXTBENCH_FRONT_HALF_PREREGISTRATION.md`](CONTEXTBENCH_FRONT_HALF_PREREGISTRATION.md).
It freezes eligibility, the metadata-only deterministic sampling rule, the
gold/evidence boundary, evidence categories, baselines, metrics, stop rules,
and retention/network limits. Since eligibility failed, no case was selected
and no gold-scored result was inspected.

## 8. Adapter and evidence semantics

No adapter was implemented because the preregistered trajectory-availability
gate failed. The intended adapter vocabulary remains:

- `CAPTURED_EXPLICIT`
- `DERIVED_STRUCTURAL`
- `EVALUATION_ONLY`
- `INFERRED_UNSAFE`
- `ABSENT`
- `UNKNOWN`

The intended protection decisions remain `PROTECT_EXPLICIT`,
`PROTECT_STRUCTURAL`, `UNKNOWN_RETAIN`, and `NO_POSITIVE_EVIDENCE`.
No semantic LLM inference or invented dependency edge was introduced.

## 9. Evaluation and metrics

No evaluation granularity was available because there was no joined
trajectory. The following are all **NOT RUN** rather than zero measurements:

| Measure | Result |
| --- | --- |
| `KEEP_ALL` | NOT RUN |
| `EXPLICIT_REFERENCE_ONLY` | NOT APPLICABLE — no trajectory |
| `RECENCY` | NOT RUN |
| Prefixity protection | NOT RUN |
| Gold-context protection recall | NOT RUN |
| File/symbol/span recall | NOT RUN |
| Precision and F1 | NOT RUN |
| `UNKNOWN_RETAIN` and selectivity | NOT RUN |
| Gold-context miss count/categories | NOT RUN |
| Join success | 0 / 1,136 |

There is no claim of signal, selectivity, safety, or failure from these
unrun metrics.

## 10. Leakage, retention, and determinism

- Gold labels used for evidence extraction: **no**; no extraction occurred.
- Raw benchmark rows in tracked Prefixity: **no**.
- Raw source repositories, patches, problem statements, and trajectories in
  tracked Prefixity: **no**.
- Temporary benchmark checkout: outside tracked Prefixity.
- Preflight determinism: **pass** for pinned revisions, source hashes, schema
  observations, and the zero-trajectory scan.
- Scoring determinism: **not applicable**.

The machine-readable preflight and report are:

- [`CONTEXTBENCH_FRONT_HALF_ADMISSION_PREFLIGHT.json`](CONTEXTBENCH_FRONT_HALF_ADMISSION_PREFLIGHT.json)
- [`CONTEXTBENCH_FRONT_HALF_REPORT.json`](CONTEXTBENCH_FRONT_HALF_REPORT.json)

Frozen artifact hashes:

- preregistration: `05bc1daa9e6b621cd370da2365106519487f71731391ffa825bcdb3bdd4b3180`;
- admission preflight: `d6f9616bed46b9f98a120081aee3fae2d41ef9841d4539ac31f19ad0844ead34`;
- combined preflight determinism hash: `1ba696638c21df135641dd5b5d7c2f10473f62ab1185272460fbb8b00d0a069a`;
- adapter: not created, therefore no adapter version or hash.

## 11. Model/provider and accounting boundary

- Provider/model/API calls: **0**
- Inference spend: **0**
- Credentials read or provisioned: **0**
- Live replay or prompt mutation: **0**
- Production planner changes: **0**

## 12. Gold misses and limitations

There are no gold misses to categorize because no trajectory was joined and
no score was run. The central limitation is external-evidence availability,
compounded by the absence of a declared dataset license and the need for
source-specific review of the underlying benchmark repositories and task
material.

This result does not establish that every ContextBench row is inadmissible
for all research. It establishes only that this bounded Prefixity study could
not honestly proceed from the pinned material without a separately cleared
trajectory source.

## 13. What this establishes

- The corrected ContextBench identity and pins are recorded.
- The repository code license, dataset-license uncertainty, and
  source-specific provenance boundary are explicit.
- The evaluator's trajectory formats are not evidence that trajectories are
  supplied.
- The no-go boundary prevents synthetic trajectories, model-generated traces,
  raw-data vendoring, and accidental gold leakage.

## 14. What this does not establish

- It is not Phase 1C Stage 1.
- It is not provider/model/API evidence.
- It is not a production safety result.
- It is not a planner-quality result.
- It is not evidence that Prefixity is worse than a baseline.
- It is not permission to redistribute ContextBench or underlying benchmark
  data.

## 15. Relocation constraint

The existing constraint remains unchanged: `RELOCATE_CANDIDATE` requires
stronger model-in-the-loop treatment in later separately authorized work.
This no-go study tested no positional model behavior and does not alter that
production risk tier.

## 16. Stage 1 readiness

`STAGE_1 REMAINS BLOCKED — BENCHMARK/PROVENANCE ISSUE`

The absence of a permission-cleared joined trajectory prevents this result
from justifying even a tiny Stage 1 provider-schema expenditure. This decision
does not authorize provider calls.

## 17. Exactly one recommended next task

**Obtain a permission-cleared, pinned external trajectory artifact with a
stable join to ContextBench, then reopen the admission review.**

No work on that recommendation is begun by this checkpoint.
