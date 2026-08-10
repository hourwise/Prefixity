# Phase 1C External Evidence and Front-Half Validation Gate

**Status:** research and authorization-gate document only

**Review date:** 2026-08-10

**Decision:** `STAGE_1_BLOCKED_FRONT_HALF_EXTERNAL_VALIDATION_REQUIRED`

This document completes the Phase 1C external-evidence gate inserted between
the certified offline Stage 0 checkpoint and any provider schema smoke. It
does not authorize provider/model/API calls, credentials, network replay,
prompt mutation, paid evaluation, production-planner changes, policy
promotion, Stage 1, Stage 2, or a ContextBench replay implementation.

## 1. Checkpoint and protected state

- Repository: `hourwise/Prefixity`, branch `main`.
- Authoritative Stage 0 commit:
  `4a11466d91d5f33818f89896c360580757c3a364`
- Stage 0 commit message: `feat: certify Phase 1C Stage 0 offline replay`.
- CI #35 was successful at the Stage 0 SHA: [GitHub Actions run #35](https://github.com/hourwise/Prefixity/actions/runs/31375105655).
- The unrelated local modification to `docs/tasks/ACTIVE.md` is excluded
  from this task. Its protected SHA-256 remains
  `D329C117BF346D65B2587B07EF9B13AA394E5796B580C623E71B1593853F17E2`.
- The only intended Phase 1C research changes are this document and the
  dated research addition to `docs/research/PRIOR_ART.md`.

## 2. Stage 0 result and scope

Stage 0 remains valid and is not amended by this gate. The certified fixture
set contains 17 synthetic tasks, frozen input/evidence hashes, deterministic
mock transport, and accounting assertions. The certification recorded zero
network calls, zero credential reads, and zero spend. It establishes that the
offline harness can validate request construction, policy decisions, mock
transport, and accounting.

It does not establish natural-trace evidence quality, model behavior,
provider cache behavior, provider-native compaction behavior, generalization,
or production readiness. No Stage 0 code, fixture, hash, result, or policy was
changed during this gate.

## 3. Review method and boundaries

The review used the Prefixity source-of-truth documents and implementation,
official provider documentation, primary papers/preprints, and read-only
public repository metadata. Claims were classified as one of:

1. **Observed in Prefixity:** local implementation or certified evidence.
2. **Reported by an external source:** a claim made by a cited paper,
   provider, or benchmark repository.
3. **Design proposal:** a future Prefixity experiment or threshold.
4. **Not established:** a claim for which this gate found no admissible
   evidence.

No external benchmark data was copied into the repository. No provider or
model inference endpoint was contacted. No credential was read or provisioned.

## 4. What the external review verified

### 4.1 The closest context-management systems

The reviewed systems overlap with parts of the Prefixity hypothesis, but their
decision/evidence boundaries differ:

| System | Verified mechanism | Boundary relevant to Prefixity |
| --- | --- | --- |
| ACON | Contrastive failure-driven optimization of learned context-compression guidelines | Useful success/failure counterfactual for evaluation; not deterministic planner evidence |
| ContextWeaver | LLM-mediated dependency parent selection, dependency summaries, ancestry traversal, and runtime validation summaries | Inferred dependency edges must not be treated as captured structural facts |
| AgentDiet | Separate reflection model, bounded sliding window, explicit Original/Random/Delete baselines | Demonstrates paired quality, trajectory-step, token, and cost accounting |
| AgentFold | Model-emitted granular condensation and deep-consolidation directives | Learned/model-directed context control, not provider-neutral policy |
| Context as a Tool / SWE-Compressor | Learned context-management tool behavior from reconstructed trajectories | Strong comparator for learned compression, outside this deterministic offline gate |
| SWE-Pruner | Task-aware neural skimmer intercepting file reads | Relevant to observation pruning; not auditable evidence admission |
| VISTA | Dashboard plus exact archive/recovery operations | Shows value of state visibility and recovery; the model still selects actions |
| ContextCite | Model-based randomized ablation attribution | Evaluation-only behavioral attribution, not planner input |

References and the detailed evidence notes are in
[`docs/research/PRIOR_ART.md`](../research/PRIOR_ART.md).

The conservative conclusion is that generic context management, compression,
summarization, caching, and archive/recovery are established areas. Prefixity
cannot claim novelty from any one of those mechanisms. The still-unverified
candidate distinction is an auditable, provider-neutral evidence-to-decision
layer that can decline to intervene and can account for quality and cache
economics end to end.

### 4.2 Provider-native context and cache semantics

The official documentation establishes materially different comparators:

- Anthropic documents server-side context editing for clearing tool results or
  thinking blocks, plus server-side compaction and prompt-cache breakpoints.
  Edits can invalidate cache prefixes; compaction requires accounting across
  its additional iteration(s). See [context editing](https://platform.claude.com/docs/en/build-with-claude/context-editing),
  [compaction](https://platform.claude.com/docs/en/build-with-claude/compaction),
  and [prompt caching](https://platform.claude.com/docs/en/build-with-claude/prompt-caching).
- OpenAI documents exact-prefix prompt caching with explicit breakpoints and
  provider-generated Responses compaction. See [prompt caching](https://developers.openai.com/api/docs/guides/prompt-caching)
  and [compaction](https://developers.openai.com/api/docs/guides/compaction).
- Gemini documents implicit and explicit caching and usage metadata, but the
  reviewed documentation does not establish a generic context-editing or
  compaction facility equivalent to the Anthropic/OpenAI comparators. See
  [GenerateContent caching](https://ai.google.dev/gemini-api/docs/generate-content/caching)
  and [current caching documentation](https://ai.google.dev/gemini-api/docs/caching).

Therefore a later replay must compare Prefixity against the provider-native
mechanism actually available for the pinned provider/model/API version. A
single small first replay should select one meaningful provider-native
comparator, not silently treat different providers' mechanisms as equivalent.

### 4.3 CAPC and the economics of doing nothing

The current paper reviewed as CAPC is
[Cache-Aware Prompt Compression: A Two-Tier Cost Model for LLM API Caching](https://arxiv.org/abs/2607.15516).
The older local pointer to arXiv `2503.08158` was not located during this
review and is not treated as the same work.

The current paper reports Anthropic Sonnet 4.6 measurements for vanilla,
cache-only, query-aware compression, and query-agnostic cache-aware
compression. It reports a cost crossover that depends on cache write/read
prices, prefix size, and the fraction of the cached prefix mutated by the
compressor. It also reports a 16/16 LongBench-v2 dominance grid and
multi-round workload studies. These are externally reported measurements, not
Prefixity evidence; the paper itself reports paid API use and current-version
pricing/behavior.

The useful design lesson is stronger than "compression saves tokens":

- cache writes, cache reads, invalidation, compression overhead, and quality
  must be measured together;
- a changing early prefix can make an intervention lose money;
- the same query-aware strategy can save on one prompt composition and lose on
  another because the mutated fraction differs;
- a no-op or provider-managed baseline can already capture part of the benefit.

The later accounting manifest must therefore retain `DO_NOTHING`, the
unmodified baseline, provider-native/cache-only comparator where available,
intervention overhead, input/output/cache tokens, physical calls, retries,
latency, and total spend. CAPC numeric thresholds must not be hard-coded into
Prefixity.

## 5. The unresolved front half

The current Prefixity policy can reason deterministically over neutral
structural facts when trusted evidence supplies them. The central missing
piece is evidence admission from a natural external trace:

```text
external record
  -> identity/provenance join
  -> captured or derived relation
  -> candidate required-context set
  -> admissible evidence class
  -> conservative decision
```

The accepted CodeTraceBench slice is an observational, hash-only source. It
does not by itself supply a complete action/result/dependency/removability
join for each candidate. External context benchmarks may supply human gold
context, execution traces, or inferred relationships, but those are different
evidence classes and have different licensing and leakage risks.

The front-half question is therefore not "which compressor is best?" It is:

> Can a permission-cleared, reproducible external record be mapped to the
> Prefixity candidate identity and protected-required set without silently
> converting a human gold label, inferred semantic dependency, or behavioral
> outcome into a captured planner fact?

Until that question is answered, a successful controlled-policy replay would
not establish natural-trace safety.

## 6. Phase 1B.9 reinterpretation

Phase 1B.9 remains a valid controlled result under the frozen research-only
policy `controlled-evidence-policy-v1`, scope `CONTROLLED_ONLY`. Its held-out
set reported four selected positives, four true positives, and zero false
positives, false negatives, unsafe actions, or regressions under the frozen
controlled construction.

That result establishes:

- a deterministic blinded structural mapping on the authored controlled set;
- one consistent evidence-to-decision rule;
- fail-open behavior for unsupported or ambiguous evidence;
- useful bounded oracle facts for later test design.

It does not establish population precision/recall, natural workload
generalization, provider/model quality, economics, or live readiness. The
answer-coded identifiers and other construction details are leakage risks for
future experiments. The policy is not promoted and remains research-only.

For the future evidence taxonomy, retain the existing production-facing
classes (`CAPTURED_EXPLICIT`, `DERIVED_STRUCTURAL`, `EVALUATION_ONLY`,
`INFERRED_UNSAFE`, and `ABSENT`). A research adapter may use a sidecar
classification, without expanding the production schema in this task:

- `EXTERNAL_GOLD_LABEL`;
- `INFERRED_SEMANTIC_DEPENDENCY`;
- `OBSERVED_BEHAVIORAL_DEPENDENCY`;
- `UNKNOWN_OR_UNRESOLVED`.

Only evidence admitted by a pre-registered rule may affect a future candidate;
gold labels and behavior after intervention remain evaluation-only.

## 7. Corrected ContextBench identity and admission audit

The prior staged draft used `cioutn/context-bench` as the ContextBench source.
That was a material same-name repository identification error. It is removed
as a license or provenance basis. The corrected benchmark source is:

- Repository: [EuniAI/ContextBench](https://github.com/EuniAI/ContextBench)
  at pinned revision
  `1436c28a8eb95496da4ea69ad458b9f8a8eb7d61`.
- Paper: [arXiv 2602.05892](https://arxiv.org/abs/2602.05892).
- Documentation: [euniai.github.io/ContextBench](https://euniai.github.io/ContextBench/).
- Dataset card: [Contextbench/ContextBench on Hugging Face](https://huggingface.co/datasets/Contextbench/ContextBench).
- Observed `main` at audit time: the requested SHA
  `1436c28a8eb95496da4ea69ad458b9f8a8eb7d61`.

### 7.1 Code license

The pinned repository contains a root `LICENSE` declaring Apache License 2.0.
The README also describes the project as Apache-2.0 licensed. This supports
use, modification, and redistribution of the repository's own code and
documentation subject to Apache-2.0 conditions. It does not by itself settle
the rights to third-party benchmark material or all code under
`agent-frameworks/`; those components require their own attribution and
license inspection.

No separate data license, dataset-specific permission statement, or `NOTICE`
file was identified at the pinned repository root. The root license must not
be interpreted as a grant from the authors of every underlying repository or
issue artifact.

### 7.2 Distributed dataset and metadata

The pinned repository includes source metadata CSVs (`Verified.csv`,
`Multi.csv`, `Poly.csv`, and `Pro.csv`), selected-instance metadata, and
parquet files including full and verified splits. Its README directs users to
the public `Contextbench/ContextBench` Hugging Face dataset and identifies
1,136 issue-resolution tasks from 66 repositories across eight languages,
human-annotated gold contexts, and trajectory recall/precision/efficiency
evaluation.

The currently observed Hugging Face dataset revision is
`c2855792b006af41c67202d33883fb9d46362853`. The dataset is public and
ungated. Its card metadata exposes fields including `gold_context`, `patch`,
`test_patch`, `problem_statement`, `repo_url`, `base_commit`, and `source`.
The card metadata does not declare a dataset license. Public availability and
an ungated download path establish technical access, not blanket permission
to copy or redistribute the dataset.

### 7.3 Underlying task provenance and restrictions

The paper states that ContextBench pools tasks from SWE-bench Verified,
Multi-SWE-bench, SWE-PolyBench PB500, and SWE-bench Pro, then deduplicates,
selects, and annotates them. It records repository URLs and base commits and
distributes gold-context spans, patches, test patches, and problem/task
metadata. The paper also describes human dependency tracing and LLM/test-based
context verification.

The source families do not create one universal license for all resulting
rows. For example, the official [SWE-bench repository](https://github.com/SWE-bench/SWE-bench)
declares MIT, the [Multi-SWE-bench repository](https://github.com/multi-swe-bench/multi-swe-bench)
declares Apache-2.0, and the [SWE-PolyBench repository](https://github.com/amazon-science/SWE-PolyBench)
declares MIT. These are licenses for those projects and do not automatically
grant rights over every upstream repository, issue discussion, patch, test
patch, or code excerpt represented in their datasets. [SWE-bench Pro](https://arxiv.org/abs/2509.16941)
and every referenced repository/task need an item-level review before
redistribution. The ContextBench root Apache-2.0 license cannot override
those third-party terms.

The result is four distinct questions:

1. **ContextBench framework/code:** Apache-2.0 at the pinned repository.
2. **ContextBench distributed dataset:** public and ungated, but no explicit
   dataset license was found in the observed Hugging Face card metadata.
3. **Underlying tasks/repositories:** mixed provenance inherited from the
   four source benchmark families and the individual GitHub repositories;
   unresolved in aggregate.
4. **Local research versus redistribution:** a bounded local study is
   technically feasible after a per-source provenance/license preflight, but
   that does not authorize copying, vendoring, or redistributing raw rows or
   source excerpts inside Prefixity.

### 7.4 Admission classification

**`ADMISSIBLE WITH PROVENANCE RESTRICTIONS`**.

This is more permissive than the mistaken prior `NOT ADMISSIBLE` disposition
because the corrected source is identifiable and pinned, its framework code is
Apache-2.0, and a public/ungated dataset release makes a bounded local
evaluation technically feasible. It is not unrestricted admission because the
dataset card has no explicit license and the underlying task/repository rights
are mixed and not resolved by the ContextBench repository license.

The allowed research shape is a local, read-only, permission-cleared slice or
metadata/hash-only adapter with a provenance manifest. Prefixity must not
check in raw ContextBench rows, gold-context source text, issue text, patches,
test patches, cloned repositories, or vendored ContextBench data. Each future
slice must record the pinned ContextBench revision, Hugging Face revision if
used, source benchmark, underlying repository, base commit, applicable
license/permission, transformations, and retention/distribution decision.

AppWorld and tau2-bench remain possible future controlled-world references,
subject to their own release pinning and data boundaries. AppWorld's public
repository is Apache-2.0 but its encrypted bundles have distinct terms and
the project asks users not to publish raw or derived benchmark material.
tau2-bench is MIT-licensed, but generated trajectories require a configured
model/provider boundary and its README warns that grading changes make scores
across releases non-comparable. Neither benchmark was imported or executed.

## 8. Proposed bounded front-half experiment (not authorized here)

The selected next task remains **ContextBench bounded front-half adapter +
external evidence study**. It must begin with a provenance/license preflight
against the corrected `EuniAI/ContextBench` revision and the exact dataset
revision used. The experiment must use only a permission-cleared local slice,
or metadata/hashes and independently licensed source artifacts. It must not
send prompts to a provider, alter a live prompt, or copy raw ContextBench data
into Prefixity.

### Inputs and outputs

The adapter would ingest only the minimum permitted structural records after
the provenance preflight:

- stable source/revision identifiers and content hashes;
- bounded task/trajectory identifiers;
- event order and typed tool/observation identity where licensed;
- explicit reference/action/result joins when captured by the source;
- gold required-context locators only in an evaluation-only sidecar;
- provenance, license, transformation, retention, distribution, and
  unknown-field records.

It would emit a sidecar candidate map and evidence class. It would not emit a
production policy mutation, natural-language summary, semantic dependency
claim without provenance, or outcome-derived planner label.

### Offline comparison arms

The pre-registered comparison should contain:

1. `KEEP_ALL`: retain all admitted context; the safety baseline.
2. `EXPLICIT_REFERENCE_ONLY`: use only source-captured references, when
   present; otherwise fail open.
3. `RECENCY_ONLY`: a diagnostic baseline only when source order is explicit;
   it must not be presented as a quality baseline when order is unavailable.
4. `PREFIXITY_ADMISSION`: apply only the deterministic evidence classes
   admitted by the frozen rule; unknowns become `KEEP`/`DO_NOTHING`.

The comparison is over evidence admission and candidate classification. It is
not a model-quality or provider-cost experiment.

### Metrics and hard thresholds

Safety is evaluated before efficiency:

- protected required-context recall: **100% required for a safety pass**;
- zero unexplained false negatives in the protected/gold subset;
- all false negatives, unknowns, incomplete joins, and provenance gaps listed
  individually;
- precision and F1 reported only alongside recall and coverage;
- evidence provenance completeness and identity/relation join rate reported;
- intervention-eligible coverage reported separately from safe retained
  context, so low coverage cannot masquerade as high precision.

If protected recall or provenance completeness fails, the result is
`INCONCLUSIVE_OR_BLOCKED`, not a quality success. Efficiency is considered
only after the safety threshold: the existing Phase 1C efficiency gate is a
fresh-input reduction of at least 10% or billed-cost reduction of at least 5%,
with no increase in output tokens, rounds, tools, rereads, recovery actions,
physical calls, or safety failures. Those thresholds are proposals for a
future authorized experiment, not results of this gate.

## 9. Relocation risk

Recommendation: **structurally selectable but require model-in-the-loop
validation before application**.

Relocation can preserve bytes and explicit identifiers, so it remains useful
as a structural candidate. It is not semantically free: position can change
retrieval/use quality, and moving content can change the stable provider prefix
and cache accounting. The position evidence in [Lost in the Middle](https://arxiv.org/abs/2307.03172)
and [Found in the Middle](https://arxiv.org/abs/2406.16008) supports this
caution. No relocation should be applied in a later live arm unless the
unmodified and relocated prompts are paired, the model-quality outcome is
measured, cache-prefix effects are recorded, and any failure resolves to
`KEEP`/`DO_NOTHING`.

## 10. Replication and nondeterminism policy

Recommendation: **deterministic first pass plus fixed confirmation**.

The offline front-half adapter should be deterministic and hash-checked. A
later live experiment may use one first pass as a wiring/preflight pass, but
its scored cohort and arms must then receive a fixed confirmation replicate
plan chosen before observing results. Replicates must not be added only to
favorable or disputed cases. A disagreement is an evaluation signal and must
not be hidden by selecting a preferred run.

Every later live replicate must pin and record provider, model/version,
endpoint/API version, prompt/input hashes, tool definitions, sampling
settings where supported, cache settings, request IDs, usage fields, physical
call count, retries, latency, and grader/version. Temperature zero or a seed,
where available, is a reproducibility aid rather than a byte-identical
guarantee. This follows the reviewed repeated-run evidence in
[Quantifying non-deterministic drift](https://arxiv.org/abs/2601.19934) and
[Necessary but Not Sufficient](https://arxiv.org/abs/2606.26185).

## 11. Impact on the existing Phase 1C design

- **Stage 0:** no impact. Its offline certification remains valid with its
  original hashes, fixtures, mock transport, and accounting.
- **Stage 1 manifest:** must add an explicit external-evidence admission
  record, provenance/licensing manifest, a provider-native comparator choice,
  a fixed replicate plan, and a no-op/control arm before schema smoke is
  considered.
- **Stage 2 replay/evaluation:** must separate front-half evidence quality from
  model behavior, preserve baseline/pass-to-fail gates, measure position and
  cache invalidation, and include full cost/usage accounting.
- **Production:** no planner change, prompt mutation, credential addition, or
  promotion of `controlled-evidence-policy-v1` is authorized by this gate.

## 12. Readiness, novelty, and positioning

### Stage 1 readiness

The exact readiness decision is:

> **BLOCKED - do not begin Phase 1C Stage 1 schema smoke.**

The blocker is not Stage 0 or the deterministic controlled harness. It is the
absence of an admissible, provenance-complete natural/external evidence path
that can support the front half of the decision. A Stage 1 provider boundary
test before resolving that gap would validate transport/schema wiring, not the
central evidence hypothesis, and would risk spending authority on an
under-specified evaluation.

### Conservative novelty assessment

- Research novelty: **medium and conditional**. The component mechanisms have
  substantial prior art; the evidence-admission plus conservative decision and
  accounting combination remains an unvalidated hypothesis.
- Architecture novelty: **medium and conditional**. Provider-neutrality,
  provenance, fail-open behavior, and explicit `DO_NOTHING` could differentiate
  the architecture if independently demonstrated, but provider-native and
  learned systems overlap the context-management surface.
- Product differentiation: **conditional, not established**. The strongest
  possible position is auditable justification for changing context, not a
  generic claim of compression, caching, summarization, or lower token count.

Proposed conservative positioning:

> Prefixity is a provider-neutral, auditable context-decision layer that
> records when changing accumulated agent context is justified and can decline
> to intervene. It does not yet claim natural-trace correctness, provider
> superiority, universal savings, or production readiness.

## 13. Exact next task selected by this gate

The single next task is:

> **ContextBench bounded front-half adapter + external evidence study**

The corrected scope is a permission-cleared, hash-only or metadata-only
adapter at the pinned `EuniAI/ContextBench` revision, followed by a bounded
offline front-half study against the gold/structural reference. It must carry
complete source/task provenance and leakage controls and must not vendor raw
dataset material.

This task must not begin automatically. It requires separate direct
authorization after review of this gate. It must not include provider/model
calls, credentials, schema smoke, paid replay, prompt mutation, production
planner changes, or policy promotion.

## 14. Limitations and stop condition

- This is a literature/provider-document review, not an independent
  reproduction of any external benchmark or paper result.
- Several closest systems are preprints and use learned/model-mediated
  decisions; their reported numbers are not interchangeable with Prefixity's
  deterministic controlled evidence.
- Provider cache, pricing, compaction, model, and API semantics are versioned
  external facts and must be re-measured at any later authorized replay.
- ContextBench is admitted only with provenance restrictions: no raw data was
  downloaded into Prefixity, and no dataset or source-task redistribution
  permission was inferred from the repository's Apache-2.0 code license.
- Gold labels, inferred dependencies, and post-intervention outcomes must not
  be allowed to leak into the evaluated planner.
- No provider/model/API call, credential access, prompt mutation, replay,
  production change, or policy promotion occurred in this gate.

The research gate is complete. The authorized stopping point is after
validation and staging of the two research documents. No commit or push is
authorized by the attached request; direct authorization is required for the
exact proposed commit message `docs: add Phase 1C external evidence gate`.
