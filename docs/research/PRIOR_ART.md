# Prior art

Research that shaped the Prefixity hypothesis. This document records *what
was learned*, not a survey, and lists each reference so claims can be audited
later. "Reviewed" dates are when the notes were last updated; provider
pricing and product details change, so no entry here should be read as
current fact.

All references below were reviewed/updated on **2026-08-07** (Phase 0A.1
review) unless stated otherwise.

## Systems reviewed

- **Provider-native prompt/prefix caching** — OpenAI, Anthropic, DeepSeek and
  others expose prefix caching with different mechanisms, prices, TTLs and
  reporting. A core Phase 0 conclusion: cache economics must be **data**, not
  hard-coded fact.
  - Reference: provider API docs (OpenAI usage fields, Anthropic cache
    fields, DeepSeek cache hit/miss fields). Field-level semantics are
    captured as *data* in the trace format and normalized offline; see
    `docs/phase-0/TRACE_FORMAT.md`.
- **CacheLane** — a middleware approach to managing prompt caching lanes.
  Relevant because it assumes the stable-prefix problem is worth solving at
  the proxy layer.
  - Reference: github.com/bhancockio/cache-lane (reviewed 2026-08-07).
- **Prefixion** — **important negative evidence.** A developer reports having
  built a multi-layer volatility/hash/provider-caching architecture that did
  not materially reduce costs in practice. This is the single strongest
  reason Phase 0 exists to *test*, not to *assume*.
  - Reference: github.com/alabhyajindal/prefixion (reviewed 2026-08-07).
- **Graft / Madar** — context-reduction research suggesting that avoiding
  unnecessary context and tool calls may be more valuable than merely caching
  large prompts. This motivates fixtures 04/06 (volatile tool output and
  optional/stale material dominating fresh context).
  - References: Madar — github.com/aribornstein/Madar (reviewed 2026-08-07);
    Graft — as noted in prior review (2026-08-07).
- **llmtrim** — trimming/prompt-compaction tooling; evidence that simple
  token-count reduction is tempting and that correctness effects are the hard
  part (compression is therefore reserved, not implemented, in Phase 0).
  - Reference: github.com/argmaxinc/llmtrim (reviewed 2026-08-07).
- **VS Code Cache Explorer** — a UI for inspecting provider cache state;
  evidence that observability of caching is a real need, but also that
  observability alone is not a novel product.
  - Reference: marketplace item `qyurila.vscode-cache-explorer` (reviewed
    2026-08-07).
- **LMCache / vLLM automatic prefix caching** — server-side KV/prefix caching.
  Prefixity is deliberately *not* reimplementing these; they are the
  providers' and hosts' problem space.
  - References: github.com/LMCache/LMCache; vLLM automatic prefix caching
    (reviewed 2026-08-07).
- **GPTCache** — semantic response caching. Explicitly out of scope for
  Phase 0 (semantic caching can change answers; Prefixity does not do it).
  - Reference: github.com/zilliztech/GPTCache (reviewed 2026-08-07).
- **CAPC (Cache-Aware Prompt Compression)** - research on the interaction
  between compression and caching.
  - The older pointer to arXiv `2503.08158` was not located during the
    2026-08-10 review and is not silently equated with the current paper.
  - Reference checked during the current review: [Cache-Aware Prompt
    Compression: A Two-Tier Cost Model for LLM API Caching](https://arxiv.org/abs/2607.15516).

## Phase 1C external-evidence review (2026-08-10)

This section records the primary-source review performed for the Phase 1C
external-evidence and front-half validation gate. It is a research record,
not a claim that Prefixity has reproduced any external result. Provider
documentation and preprints are time-sensitive; the cited URLs and the review
date are part of the provenance.

### Context-management systems and the evidence boundary

- **ACON** ([paper](https://arxiv.org/abs/2510.00615),
  [code](https://github.com/microsoft/acon)) uses failure-driven,
  contrastive optimization to improve learned context-compression guidelines.
  Its successful-full-context versus failed-compressed-context comparison is
  useful as an evaluation-time counterfactual signal. It is not a
  provider-neutral deterministic evidence source, and an outcome label from
  the same evaluation cannot be used as planner-time evidence for that
  evaluation.
- **ContextWeaver** ([paper](https://arxiv.org/abs/2604.23069)) constructs a
  dependency graph over executed steps, but parent selection is described as a
  reasoning task and dependency summaries are produced by an LLM. Its runtime
  validation summary is valuable evidence to retain, while its inferred edges
  must remain distinguishable from captured structural relations. This is a
  direct reason to keep evidence admission separate from the deterministic
  Prefixity decision layer.
- **AgentDiet** ([paper](https://arxiv.org/abs/2509.23586)) uses a separate,
  cheaper reflection model and a bounded sliding window to remove redundant,
  expired, or useless trajectory content. The paper reports explicit Original,
  Random, and Delete baselines and measures pass rate, steps, input/output
  tokens, and cost. This supports paired quality-plus-accounting evaluation;
  it does not establish that a deterministic structural rule can safely make
  the same removability decisions.
- **AgentFold** ([paper](https://arxiv.org/abs/2510.24699)) has the agent emit
  folding directives, including granular condensation and deep consolidation,
  for long-horizon web tasks. The model-directed control and training regime
  make it a comparator for learned context management, not a drop-in
  provider-neutral policy oracle.
- **Context as a Tool / SWE-Compressor** ([paper](https://arxiv.org/abs/2512.22087))
  trains context-management behavior from reconstructed trajectories and
  compares against unmodified ReAct and threshold compression. Its results
  reinforce that context management and task quality must be measured jointly;
  learned summaries and tool calls are outside the current deterministic,
  offline Prefixity gate.
- **SWE-Pruner** ([paper](https://arxiv.org/abs/2601.16746),
  [code](https://github.com/Ayanami1314/swe-pruner)) places a task-aware neural
  skimmer between a coding agent and file-reading operations. It reports
  token and interaction-round reductions alongside success measures. It is
  relevant to the front-half question of what a tool observation contains,
  but it is learned middleware rather than auditable captured evidence.
- **VISTA** ([paper](https://arxiv.org/abs/2606.30005)) exposes a dashboard of
  per-block size, recency, and archive state and lets an agent archive and
  recover exact payloads. It demonstrates why observability, recovery, and
  position matter, but the agent still chooses the archive/recovery actions.
  The dashboard is a useful measurement pattern, not evidence that a model
  will use a changed context safely.
- **ContextCite** ([paper](https://arxiv.org/abs/2409.00729),
  [code](https://github.com/MadryLab/context-cite)) attributes an answer to
  sources through randomized context ablations. It requires model inference
  and behavioral outcomes, so it belongs in future evaluation or attribution
  work, not in the current planner evidence path.
- **Sufficient Context** ([paper](https://arxiv.org/abs/2411.06037)) separates
  whether a provided context is sufficient from whether a model fails to use
  it. This supports Prefixity's distinction between evidence sufficiency and
  model behavior, but it is not an agent-trace admission format.

### Provider-native semantics and cache economics

The current official provider documentation confirms that cache and context
management are provider-specific comparators, not interchangeable primitives:

- [Anthropic context editing](https://platform.claude.com/docs/en/build-with-claude/context-editing)
  can clear old tool results or thinking blocks while preserving placeholders
  or selected recent content; the response reports applied edits and cleared
  tokens. [Anthropic compaction](https://platform.claude.com/docs/en/build-with-claude/compaction)
  creates a provider-generated summary and requires usage accounting across
  compaction iterations. [Anthropic prompt caching](https://platform.claude.com/docs/en/build-with-claude/prompt-caching)
  documents ordered prefixes, cache breakpoints, minimum cacheable sizes, and
  cache-read/cache-write usage fields.
- [OpenAI prompt caching](https://developers.openai.com/api/docs/guides/prompt-caching)
  documents exact prefix matching, explicit cache breakpoints, cache keys, and
  model-specific usage fields. [OpenAI compaction](https://developers.openai.com/api/docs/guides/compaction)
  documents provider-generated compaction items and the stateless chaining
  rules around them.
- [Gemini caching](https://ai.google.dev/gemini-api/docs/generate-content/caching)
  and the current [Interactions caching documentation](https://ai.google.dev/gemini-api/docs/caching)
  document implicit and explicit caching, token thresholds, TTLs, and usage
  metadata. The reviewed documentation does not establish a generic
  provider-native context-editing or compaction comparator equivalent to the
  Anthropic/OpenAI mechanisms.

The current CAPC preprint ([arXiv 2607.15516](https://arxiv.org/abs/2607.15516))
is the closest reviewed economic analysis. It measures Anthropic Sonnet 4.6,
compares vanilla, cache-only, query-aware compression, and cache-aware
compression, and reports a cost crossover that depends on cache write/read
prices and the fraction of a cached prefix that compression mutates. It also
reports a 16/16 LongBench-v2 dominance grid and end-to-end workload results,
but those are the paper's own paid experiments, current-version measurements,
and quality comparisons. Prefixity should reuse the measurement shape -
unmodified baseline, cache-only/provider-native comparator, intervention,
cache reads/writes, model calls, and net cost - rather than importing its
numeric thresholds as facts.

### External benchmark and licensing audit

- **ContextBench** ([paper](https://arxiv.org/abs/2602.05892),
  [repository](https://github.com/EuniAI/ContextBench/tree/1436c28a8eb95496da4ea69ad458b9f8a8eb7d61),
  [documentation](https://euniai.github.io/ContextBench/),
  [Hugging Face dataset](https://huggingface.co/datasets/Contextbench/ContextBench))
  is the corrected source for the coding-agent context-retrieval benchmark.
  At the observed `main` commit `1436c28a8eb95496da4ea69ad458b9f8a8eb7d61`,
  the repository declares Apache License 2.0 and contains the evaluator,
  source metadata CSVs, and parquet benchmark files. The README identifies
  1,136 issue-resolution tasks across 66 repositories and eight languages,
  with human-annotated gold contexts and trajectory recall/precision/efficiency
  evaluation.
  The repository's Apache-2.0 license is evidence for the repository's own
  code and documentation, not automatic permission to redistribute every
  dataset row, issue, patch, test patch, repository excerpt, or vendored agent
  framework. The Hugging Face card is public and ungated, but its current card
  metadata does not declare a dataset license; it exposes `gold_context`,
  `patch`, `test_patch`, `problem_statement`, `repo_url`, `base_commit`, and
  source fields. The underlying four source benchmark families and each
  referenced repository require separate provenance/license review. The
  corrected disposition is **admissible with provenance restrictions** for a
  bounded local study, not for copying or vendoring raw data into Prefixity.
  The earlier `cioutn/context-bench` URL was a different same-name project
  and is not a ContextBench license or provenance basis.
- **AppWorld** ([repository](https://github.com/StonyBrookNLP/appworld),
  [site](https://appworld.dev/),
  [paper](https://aclanthology.org/2024.acl-long.850.pdf)) is a controlled
  multi-app world with programmatic state and collateral checks. The public
  repository is Apache-2.0, but its README describes encrypted bundles with
  distinct terms and asks users not to publish raw or derived benchmark
  material. It is suitable as a future pinned adapter/reference, not as a
  reason to copy the protected bundles into this repository.
- **tau2-bench** ([repository](https://github.com/sierra-research/tau2-bench),
  [license](https://github.com/sierra-research/tau2-bench/blob/main/LICENSE))
  is MIT-licensed and provides simulated domains and programmatic grading, but
  generated trajectories require model/provider configuration and the README
  warns that release grading changes make scores non-comparable across
  versions. Any future use must pin an exact release/commit and record the
  provider/model boundary.
- **CodeTraceBench** remains the accepted hash-only observational slice at
  revision `aa213b84ffb6690fc37ca15766d6ca174ec36d4d` in Prefixity's own
  source-of-truth. The current public Hugging Face page reports a different
  revision and a license claim; that does not retroactively change the
  provenance, contents, or limitations of the accepted pinned slice.

### Position, stochasticity, and the resulting design constraint

[Lost in the Middle](https://arxiv.org/abs/2307.03172) reports a U-shaped
relationship between relevant information position and model performance.
That result, together with the provider documentation's exact-prefix cache
semantics, means relocation is not semantically neutral merely because the
bytes are unchanged. A 2024 follow-up, [Found in the Middle](https://arxiv.org/abs/2406.16008),
also treats positional attention bias as a measurable factor.

Repeated-run research likewise warns against treating one hosted-model result
as deterministic proof: [Quantifying non-deterministic drift](https://arxiv.org/abs/2601.19934)
reports variability at temperature 0.0, and [Necessary but Not Sufficient](https://arxiv.org/abs/2606.26185)
reports residual grader flips under forced greedy settings. The Phase 1C
consequence is a fixed, pre-registered confirmation policy with model/version,
prompt, usage, and grader disagreement recorded; one live run cannot certify a
quality threshold.

> No Prefixity benchmark claims are made by this research update. External
> measurements are reported as claims made by the cited sources and are not
> evidence that Prefixity has reproduced them.

## Conclusions that shaped the design

1. **A simple "stable/semi-stable/volatile prompt proxy" is not a novel
   product.** Similar implementations exist. If Prefixity is to be more than
   a toy, its value must be in *explanation* and *evidence*, not in "put the
   stable stuff first".
2. **Prefixion's negative result is central.** It means "cache hit %" and
   "prefix stability" are not obviously worth engineering effort. Phase 0
   must be able to *recommend doing nothing*.
3. **Context reduction may beat caching.** Graft/Madar-style evidence
   suggests removing unnecessary context and tool calls can save more than
   cache placement. Fixture `06-context-reduction-wins` encodes this.
4. **Compression and caching interact badly.** A compressor that changes an
   early prefix can destroy a valuable provider cache. Compression is
   therefore reserved behind a policy interface and never auto-applied.
5. **Provider cache economics and mechanisms differ.** Hence `CostProfile`
   as externally supplied data, and a theoretical economics evaluator that
   can show caching is a net loss under some profiles (fixture 05).
6. **Maximising "cache hit %" is NOT Prefixity's objective.** The objective
   is total economic cost, fresh input processed, latency, correctness and
   tool calls (see `docs/phase-0/EXPERIMENTS.md`).

## The revised hypothesis

> For an observed LLM/agent workload, can a deterministic tool explain where
> context cost is being incurred, identify prefix divergence and unnecessary
> context, model provider-specific cache economics, and simulate alternative
> context policies before modifying live prompts?

A useful Prefixity result may be **"do nothing"**. That is an acceptable and
desirable result, and Phase 0 is designed so it can be produced honestly.
