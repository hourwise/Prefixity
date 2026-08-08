# Prefixity — Research and Prior Art

> Index and evidence record for research relevant to Prefixity.
>
> Research does not become product truth merely by appearing here.
> Accepted conclusions must be reflected separately in `SOURCE_OF_TRUTH.md`
> or an appropriate project decision record.

## Core hypothesis

For an observed LLM/agent workload, can a deterministic tool explain where
context cost is incurred, identify structural prefix divergence and unnecessary
context, model provider-specific cache economics, and simulate alternative
context policies before modifying live prompts?

The revised hypothesis explicitly allows `DO_NOTHING`. The Phase 1 version is
stronger and remains unvalidated: a provider-neutral observer should identify
context-management opportunities in realistic agent trajectories and recommend
an intervention that reduces avoidable context cost while preserving
load-bearing context and task quality.

The narrower implemented claim is only that Prefixity can perform deterministic
offline observation/simulation and can compare that structural view with
provider-reported usage when such usage is supplied.

## Evidence supporting the approach

- Offline fixture and integration tests exercise deterministic divergence,
  same-content/different-structure handling, provider-usage reconciliation,
  profile-dependent economics, no-op/already-optimal cases, required-block
  retention and non-mutation. This supports the harness mechanics; it is not
  evidence of real-world savings or task quality.
- The controlled DeepSeek sequence on 2026-08-07 produced a schema-smoke
  `MATCH`, a stable-prefix `MATCH`, an early-divergence `MATCH` with zero reuse,
  a first late-divergence `PARTIAL_MATCH`, and a later persistence-probe
  primary `MATCH`. The detailed values and limitations are in
  [PHASE_0B_FINDINGS.md](phase-0/PHASE_0B_FINDINGS.md) and
  [PHASE_0B_DEEPSEEK_CLOSEOUT.md](phase-0/PHASE_0B_DEEPSEEK_CLOSEOUT.md).
- In the stable-prefix observation, the repository records approximately
  99.8% structural reuse potential versus approximately 99.9% realized
  provider cache reuse; in the early-divergence control both were 0%; in the
  corrected late persistence probe the proportions were approximately 89.9%
  and 90.0%. These are single controlled observations on synthetic content,
  compared by proportion rather than by absolute token subtraction.
- The live pipeline's mock tests show that provider-shaped usage can be
  captured, normalized, reconciled and written without needing real network
  access or leaking credentials.

## Evidence that challenges or could invalidate it

- The prior-art notes call the reported `Prefixion` experience an important
  negative result: a multi-layer volatility/hash/provider-caching approach was
  reported not to materially reduce costs. This is repository-recorded prior
  art, not an independently reproduced benchmark.
- The first DeepSeek late-divergence pair was only `PARTIAL_MATCH`: structural
  reuse potential was about 89.9% while realized cache reuse was about 57.9%.
  This directly challenges any claim that structural potential predicts the
  exact provider cache-hit ratio.
- Prefixity's chars/4 estimate materially differed from provider token counts
  in the schema-smoke observation (563 estimated versus 1215 provider input
  tokens). No conversion factor is inferred.
- The controlled evidence uses synthetic requests, one observed sequence per
  scenario, one live-validated provider/model, and no task-quality or
  end-to-end cost measurement. It therefore cannot establish production value,
  cross-provider generality, causal benefit from the settle delay or provider
  determinism.
- The design is not differentiated by stable-first placement or cache
  divergence observation alone; the prior-art notes identify overlapping
  systems and require explanation, evidence and decision quality to carry the
  value proposition.

## Competing and overlapping approaches

The existing [prior-art record](research/PRIOR_ART.md) and [Phase 1 prior-art
decisions](phase-1/PRIOR_ART_DECISIONS.md) point to these areas. The labels
below are the repository's scope decisions, not new market or performance
claims.

| Area / reference | What the repository records | Prefixity boundary |
| --- | --- | --- |
| Provider-native caching | Different providers expose different caching mechanisms, pricing and reporting. | Normalize supplied usage and observe provider state; do not hard-code or reimplement provider cache layers. |
| CacheLane | Middleware/cache-lane approach relevant to the stable-prefix problem. | Prefixity is an evidence/decision layer, not another generic proxy. |
| Prefixion | Negative evidence motivating falsification. | Must be able to recommend `DO_NOTHING`. |
| Graft / Madar | Context-reduction work motivating unnecessary-context/tool-output cases. | Evaluate reduction against quality and end-to-end outcomes. |
| llmtrim and CAPC | Prompt trimming/compression and cache-aware compression concerns. | Compression is reserved until quality and cache effects can be evaluated. |
| VS Code Cache Explorer | Cache-divergence observability reference. | Observability alone is not the claimed differentiator. |
| LMCache / vLLM / CacheWise | Server-side KV/prefix management. | Different layer; do not build a serving-layer KV manager. |
| GPTCache | Semantic response caching. | Explicitly out of scope. |
| ContextBench | Preferred Phase 1A external workload/evaluation candidate. | Import only after exact version, licence and provenance are checked. |
| Squeez-style pruning, ACON, LaMR-style pruning | External pruning/evaluation ideas cited in Phase 1 decisions. | Benchmark or adapt where useful; do not reimplement specialist methods as core. |

## Provider/platform behaviour on which the design depends

The trace format and code depend on the following existing repository notes and
implemented schema rules. They are not timeless provider guarantees; provider
documentation, endpoints and models must be rechecked before new live work.

- Usage is keyed by an explicit versioned API-surface schema, not provider name.
- The synthetic schema treats `input_tokens` as total input and derives fresh
  input from explicit read/write fields.
- The Anthropic Messages shape treats `input_tokens` as the uncached remainder
  and sums it with cache-read and cache-creation fields for total input.
- The DeepSeek Chat Completions shape derives total input from
  `prompt_cache_hit_tokens + prompt_cache_miss_tokens`; cache writes are not
  reported by that adapter.
- The OpenAI Chat Completions shape treats `prompt_tokens` as total input and
  reads nested `prompt_tokens_details.cached_tokens`; cache writes are not
  reported by that adapter.
- OpenAI Responses is reserved and is not silently interpreted as Chat
  Completions.
- The Phase 0B notes describe DeepSeek cache construction as asynchronous and
  best-effort and use a 10-second settle period as an experimental control.
  The closeout expressly does not establish that the delay is required or
  optimal, nor what caused the later persistence result.
- Provider tokenization/serialization is not represented exactly by the
  structural block model. The code therefore keeps Prefixity estimates and
  provider-reported units separate and compares live observations by ratio.
- All committed cost profiles are synthetic. No current provider pricing is a
  product fact in this repository.

## Unanswered questions

- Does structural observation identify useful opportunities in natural,
  multi-turn agent trajectories rather than designed synthetic traces?
- Can Prefixity preserve protocol, dependency and load-bearing context while
  identifying safe deferral or pruning opportunities?
- Do proposed interventions improve full-trajectory cost, fresh input,
  latency, tool calls and recovery behavior after quality is gated?
- How often is `DO_NOTHING` correct, and can the evaluator avoid aggregate
  scores hiding baseline-pass to intervention-fail regressions?
- How stable are cache boundaries and usage reports across providers, models,
  regions, time, request serialization, cache persistence and expiry?
- Do OpenAI and Anthropic live adapters match real responses, and what exact
  versioned schema would be required for additional API surfaces?
- What observable evidence can safely support `required`, `dependency_required`,
  `gold_required`, `optional` and `unknown` labels in a public workload?
- Which public corpus and exact revision can be imported under its licence,
  with provenance and evaluation labels kept separate from decision inputs?
- Does a provider-neutral decision layer materially outperform simpler native
  diagnostics or specialist external interventions at acceptable complexity?
- What overhead does collection and analysis add in a realistic deployment?

## Existing research and decision records

- [Prior-art notes](research/PRIOR_ART.md) - systems reviewed, conclusions that
  shaped scope, the negative Prefixion evidence and the revised hypothesis.
- [Phase 0 experiments](phase-0/EXPERIMENTS.md) - proposed cache-only,
  context-reduction and combined experiment groups; no benchmark result.
- [Phase 0 findings](phase-0/PHASE_0B_FINDINGS.md) - controlled live
  observations and limitations.
- [DeepSeek closeout](phase-0/PHASE_0B_DEEPSEEK_CLOSEOUT.md) - final Phase 0B
  decision and stopping rule.
- [Phase 1 plan](phase-1/PHASE_1_PLAN.md) - unimplemented real-workload and
  quality-gated research direction.
- [Phase 1 quality gate](phase-1/QUALITY_GATE.md), [success criteria](phase-1/SUCCESS_CRITERIA.md)
  and [workload corpus](phase-1/WORKLOAD_CORPUS.md) - proposed evaluation
  contract and provenance requirements.
- [Phase 1 prior-art decisions](phase-1/PRIOR_ART_DECISIONS.md) - explicit
  reuse/integrate/differentiate boundaries.

## Sources retained in the repository

- `docs/research/PRIOR_ART.md` lists the external project, marketplace, paper
  and provider-document pointers that were reviewed on 2026-08-07. It warns
  that those references are not current pricing facts or benchmark evidence.
- `docs/phase-0/TRACE_FORMAT.md` records the provider field semantics used by
  the offline normalizers.
- `docs/phase-0/PHASE_0B_LIVE_VALIDATION.md` records the live protocol,
  guardrails, scenario definitions, measurement-unit caveat and classification
  thresholds.
- Sanitized fixture traces under `fixtures/traces/` and local ignored run
  artifacts under `experiments/runs/` preserve reproducible shapes and the
  controlled DeepSeek observations without credentials or full private
  responses.

No new competitor, provider, licensing, pricing or performance claims were
added by this audit. Where evidence is missing, it remains an open question.
