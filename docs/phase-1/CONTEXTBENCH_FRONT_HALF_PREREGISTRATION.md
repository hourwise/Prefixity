# ContextBench Front-Half External Evidence Preregistration

Status: **FROZEN — NOT EXECUTED**

This preregistration freezes the intended bounded front-half study before any
gold-scored result is inspected. The study was stopped during the mandatory
provenance preflight because the pinned ContextBench material did not include
an admissible external trajectory joined to the gold annotations. Consequently
no sample was selected, no adapter was implemented, and no score was produced.

## 1. Authoritative inputs

- Prefixity checkpoint: `975a1ae12dcdfb590814b97711f54b39e2bafca1`
- ContextBench repository: `https://github.com/EuniAI/ContextBench`
- ContextBench repository revision:
  `1436c28a8eb95496da4ea69ad458b9f8a8eb7d61`
- Paper: `https://arxiv.org/abs/2602.05892`
- Hugging Face dataset: `Contextbench/ContextBench`
- Hugging Face dataset revision:
  `c2855792b006af41c67202d33883fb9d46362853`
- Existing admission classification: `ADMISSIBLE WITH PROVENANCE RESTRICTIONS`

The repository revision and the dataset revision are separate inputs. The
Apache-2.0 repository license is not treated as a license for third-party
dataset rows, issue text, patches, source repositories, or trajectories.

## 2. Research question

From natural coding-agent context/trajectory evidence that Prefixity did not
author, can a deterministic research adapter identify and protect human-
labelled useful code context, distinguish explicit structural evidence from
uncertainty, and provide information beyond trivial baselines without
consulting the gold answer key?

This is an evidence-admission study. It is not provider replay, model
inference, prompt mutation, production-planner validation, or proof that
non-gold context is removable.

## 3. Eligibility and admission gate

An instance may enter the scored sample only when all of the following are
available and permission-cleared for bounded local research:

1. a stable ContextBench instance identity;
2. human gold-context labels kept evaluation-only;
3. an external agent trajectory or equivalent chronological context sequence;
4. a deterministic join between the trajectory and the gold instance;
5. source/repository revision and provenance metadata; and
6. no requirement to copy or redistribute raw third-party material in the
   tracked Prefixity repository.

If the external trajectory is absent or its permission is not clear, the
instance is excluded. If the joined admitted population is below 25, the
study returns `INSUFFICIENT ADMISSIBLE SAMPLE` and does not claim external
validation. If no usable trajectory source exists for the benchmark, the
study returns:

`NO-GO — EXTERNAL TRAJECTORY EVIDENCE NOT ADMISSIBLE/AVAILABLE`

The preflight found this benchmark-level stop condition.

## 4. Sampling rule

No case was selected because the eligibility gate failed before sampling.
If the gate is later reopened with a permission-cleared trajectory artifact,
the frozen rule is:

1. construct candidate metadata without reading gold difficulty or Prefixity
   results;
2. exclude rows failing the eligibility gate;
3. sort by UTF-8 byte order of `(source, repo_url, base_commit, instance_id)`;
4. take the first 50 admitted rows;
5. if fewer than 50 but at least 25 are admitted, use all admitted rows; and
6. stop without an external validation claim below 25 rows.

The selection rule is deterministic and does not use gold spans, patches,
task success, or Prefixity performance.

## 5. Adapter-visible and evaluation-only fields

The research adapter may receive only non-evaluation trajectory evidence:

- ordered message/instruction events;
- file reads/views and explicit line ranges;
- search results and symbol lookups when present in the trace;
- tool action/result identity;
- observations and chronologically legitimate actions;
- task identity, source repository, and source revision metadata.

The adapter must never receive `gold_context`, gold symbols/spans/files,
patches, test patches, final correct diffs, task-success labels, evaluator
classifications, or any field derived from those values. Gold annotations are
evaluation-only sidecar data.

If a trajectory includes later edits, only evidence at the preregistered
decision point may be used. Later actions must not be projected backward.

## 6. Evidence vocabulary and outputs

The frozen research vocabulary is:

- `CAPTURED_EXPLICIT`
- `DERIVED_STRUCTURAL`
- `EVALUATION_ONLY`
- `INFERRED_UNSAFE`
- `ABSENT`
- `UNKNOWN`

The primary protection decision is one of:

- `PROTECT_EXPLICIT`
- `PROTECT_STRUCTURAL`
- `UNKNOWN_RETAIN`
- `NO_POSITIVE_EVIDENCE`

`NO_POSITIVE_EVIDENCE` never means safe to prune. The adapter may not invent
semantic dependency edges or introduce an LLM dependency extractor.

## 7. Granularity and metrics

Score at the finest granularity supported by both sides, in this order:
span, symbol, then file. File-only traces must not be awarded fabricated span
precision.

Primary metric:

- gold-context protection recall among gold context actually exposed by the
  admitted trajectory, reported separately for file, symbol, and span where
  available.

Secondary metrics:

- protection precision and F1;
- `UNKNOWN_RETAIN`, explicit, structural, and no-positive-evidence rates;
- coverage/selectivity beyond `KEEP_ALL`;
- provenance completeness and join success;
- admitted/excluded counts; and
- individually inspectable gold-context misses by opaque task ID.

## 8. Frozen baselines

- `KEEP_ALL`: protect every observed context item.
- `EXPLICIT_REFERENCE_ONLY`: protect only context with an explicit structured
  reference at the decision point; report `NOT APPLICABLE` if the trace does
  not expose such references.
- `RECENCY`: protect the latest fixed 20 context-bearing events, with no
  tuning after gold inspection.

The comparison asks whether Prefixity protects at least as much gold context
as the strongest applicable selective baseline while retaining less than
`KEEP_ALL`. No numeric production-safety threshold is created by this study.

## 9. Interpretation and stop rules

The only allowed final labels are:

- `PASS — EXTERNAL FRONT-HALF SIGNAL DEMONSTRATED`
- `PASS WITH LIMITED EXTERNAL SIGNAL`
- `NO-GO — FRONT-HALF EVIDENCE INSUFFICIENT`
- `NO-GO — BENCHMARK ADMISSION/TRAJECTORY INSUFFICIENT`

Any gold miss remains in the report. The scored sample may not be patched and
rerun after inspecting misses; a changed rule requires a new preregistration
and development/evaluation split.

The current frozen result is the fourth label above because no joined,
permission-cleared external trajectory was available in the pinned material.

## 10. Retention and network boundary

Tracked artifacts may contain only opaque IDs, hashes, repository/source
identifiers, aggregate counts, license/provenance metadata, bounded
structural coordinates, and non-textual gold-match metadata. They must not
contain third-party source bodies, patches, test patches, full problem
statements, raw trajectories, or cloned repositories.

Raw inputs, if separately authorized in the future, must stay in a local
ignored scratch directory outside the repository. Public network access is
limited to the pinned benchmark/repository/dataset material. Provider/model
calls, inference clients, credentials, replay, and paid APIs are outside this
preregistration.

Frozen accounting values for this run:

- model/provider calls: `0`;
- inference spend: `0`;
- production planner mutations: `0`.
