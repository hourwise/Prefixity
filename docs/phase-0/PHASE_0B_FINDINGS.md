# Phase 0B — Findings

Evidence collected from controlled live validation runs. Each entry is a
**single observation**; repeated, documented runs are required before any
claim.

## Terminology: structural potential vs realized provider cache

To avoid implying that Prefixity predicts exact provider cache-hit ratios,
Phase 0B distinguishes three concepts in every report:

- **`structural_reuse_ratio`** — the observed reusable-prefix **POTENTIAL**
  in Prefixity's structural model: what a provider cache *could* serve given
  perfect persistence of that prefix. It is **not** a prediction of the
  exact provider cache-hit ratio.
- **`provider_cache_reuse_ratio`** — the **REALIZED** provider cache reuse
  reported for that request (best-effort, asynchronous persistence may lag
  or exceed structural potential).
- **`reuse_ratio_difference`** — the **realization/alignment gap** between
  those two observations.

Classification meanings:

- **MATCH** — structural potential and realized provider reuse aligned
  closely for this observation.
- **PARTIAL_MATCH** — some provider reuse occurred but realized cache
  availability was materially below/above structural potential. It does
  **NOT** necessarily mean Prefixity's structural comparison was wrong;
  provider cache availability/state can differ.
- **NO_MATCH** — likewise must not be casually described as proof that
  structural analysis is incorrect; provider cache availability/state can
  differ.

Thresholds are research defaults and are **not** tuned per result.

## Finding 1 — DeepSeek schema-smoke (2026-08-07)
| Field | Value |
| --- | --- |
| Date | 2026-08-07 |
| Provider | DeepSeek |
| Model | `deepseek-v4-flash` |
| Scenario | `schema-smoke` |
| Requests | 1 |
| HTTP | 200 |
| Prefixity estimate | 563 estimated tokens (chars/4) |
| Provider total input | 1215 provider tokens |
| Provider cache read | 0 |
| Schema result | **MATCH** |

**Finding:** DeepSeek's real usage schema (`deepseek-chat-completions-v1`)
matched Prefixity's normalizer: `prompt_cache_hit_tokens` (0) +
`prompt_cache_miss_tokens` (1215) = total input (1215), cache read 0,
completion 1.

**Second finding:** Prefixity's generic chars/4 token estimate (563)
materially differed from the provider's tokenizer (1215) on the
deterministic synthetic corpus.

**Consequence:** cross-provider reconciliation must use explicitly labelled
measurement bases and **ratio-based comparison** rather than subtracting
absolute token counts from incompatible tokenizers. Prefixity's estimated
unit and the provider's token unit are preserved separately and never
silently converted into one another.

**Explicitly NOT concluded:** ONE observation does not establish a universal
tokenizer ratio (e.g. nothing like `estimated * 2.16` is derived or used).
No cost claims are made.

## Raw usage observed (sanitized)

The live `RawUsage` contained the DeepSeek cache fields plus
OpenAI-compatible accounting fields, all preserved verbatim:

```
prompt_cache_hit_tokens:   0
prompt_cache_miss_tokens:  1215
prompt_tokens:             1215
prompt_tokens_details.cached_tokens: 0
completion_tokens:         1
total_tokens:              1216
```

The DeepSeek normalizer reads only the cache hit/miss and completion fields;
the OpenAI-shaped fields are preserved verbatim but unused by that schema.
See the sanitized regression fixture
`fixtures/traces/17-deepseek-live-schema-smoke.json`.

## Cache settling control (design note, 2026-08-07)

Official DeepSeek Context Caching documentation describes cache construction
as asynchronous/best-effort and taking seconds, with common-prefix
persistence possibly established after multiple requests. To avoid a false
negative (C arriving before best-effort cache persistence completes), Phase
0B applies a conservative **10-second settle period after B and before C**
(`pre_request_delay_ms`: A=0, B=0, C=10000) for all DeepSeek B–D scenarios.

This is an experimental control, not a provider SLA or a required value, and
not a validated optimum. B is deliberately not delayed (A/B establish the
common prefix), and the experiment encodes no expectation that B must report
zero cache reuse. A zero cache hit after settling remains evidence to
investigate, not automatic proof that structural reuse is incorrect.

## Finding 2 — first live DeepSeek stable-prefix cache validation (2026-08-07)

| Field | Value |
| --- | --- |
| Date | 2026-08-07 |
| Provider | DeepSeek |
| Model | `deepseek-v4-flash` |
| Scenario | `stable-prefix` |
| Commit | `2f69dd6` |
| Requests | 3 (A/B/C), settle delay 10 s before C |
| HTTP | 200 |
| Conclusion | **MATCH** |

Important observations:

1. Request A was a complete cache miss (total 18061, read 0, fresh 18061).
2. Request B immediately reported **18048 cache-hit tokens / 13 cache-miss
   tokens** out of **18061 total input tokens** — a hit without any settle
   delay.
3. Request C, after the controlled 10-second settle, reported **exactly the
   same cache accounting**.
4. Prefixity independently identified **8048 / 8062 estimated tokens
   reusable = 99.826% structural reuse**.
5. DeepSeek reported **18048 / 18061 provider tokens cached = 99.928%
   provider cache reuse**.
6. The absolute tokenizer counts are very different (8048 vs 18048), but the
   reuse **proportions differ by only about 0.10 percentage points**
   (ratio difference ≈ 0.0010), so the ratio-based reconciliation is MATCH.

Interpretation:

This is strong evidence that Prefixity's structural-prefix observation can
correspond closely to real provider cache reuse for this controlled stable
prefix. It is **one** observation, not proof across models, providers, or
workloads.

Also recorded:

- B already hit without the 10-second settle, so the delay was **not
  required** for this particular stable-prefix run.
- Retaining the delay for C remains a valid conservative experimental
  control.
- We do **not** claim that 10 seconds is necessary.

See the sanitized regression fixture
`fixtures/traces/18-deepseek-live-stable-prefix.json` (request B shape) and
the reconciliation values under `experiments/runs/deepseek-stable-prefix-01/`
(gitignored).

## Finding 3 — first live DeepSeek early-divergence break (2026-08-07)

| Field | Value |
| --- | --- |
| Date | 2026-08-07 |
| Provider | DeepSeek |
| Model | `deepseek-v4-flash` |
| Scenario | `early-divergence` |
| Commit | `c894861` |
| Requests | 3 (A/B/C), settle delay 10 s before C |
| HTTP | 200 |
| Conclusion | **MATCH** |

Observed values:

1. A → B: **stable pair** — structural reuse potential ~99.8%
   (8049/8064 estimated tokens) vs realized provider cache reuse ~99.9%
   (18048/18061 provider tokens), realization gap ≈ 0.0011 → **MATCH**.
2. B → C: the **early header was changed** — structural reuse potential
   **0.0** (0/8066 estimated tokens) vs realized provider cache reuse
   **0.0** (0/18064 provider tokens) → **MATCH** (consistent no-reuse
   observations).

Interpretation:

The early prefix break destroyed **both** structural reuse potential and
realized provider cache reuse. This is a single observation and, like the
stable-prefix run, uses ratio-based reconciliation with distinct measurement
bases (Prefixity chars/4 vs provider tokens); the absolute counts are never
subtracted from each other.

See the sanitized regression fixture
`fixtures/traces/19-deepseek-live-early-divergence.json` (request C shape)
and the reconciliation values under
`experiments/runs/deepseek-early-divergence-01/` (gitignored).

## Finding 4 — first live DeepSeek late-divergence PARTIAL_MATCH (2026-08-07)

| Field | Value |
| --- | --- |
| Date | 2026-08-07 |
| Provider | DeepSeek |
| Model | `deepseek-v4-flash` |
| Scenario | `late-divergence` |
| Commit | `c894861` |
| Requests | 3 (A/B/C), settle delay 10 s before C |
| HTTP | 200 |
| Conclusion | **PARTIAL_MATCH** |

Observed values:

1. A → B: **stable pair** — structural reuse potential ~99.8%
   (8048/8063 estimated tokens) vs realized provider cache reuse ~99.9%
   (18048/18063 provider tokens), realization gap ≈ 0.0010 → **MATCH**.
2. B → C: the **late suffix was changed** — structural reuse potential
   **~89.9%** (7245/8063 estimated tokens, header + stable core) vs
   **realized** provider cache reuse **~57.9%** (10496/18115 provider
   tokens), realization gap ≈ **0.3191** → **PARTIAL_MATCH**.

This **PARTIAL_MATCH is valuable evidence, not a failed experiment**:
some provider reuse occurred (10496 provider tokens served from a shorter,
already-available cache unit) but realized cache availability was materially
below Prefixity's structural reuse potential. Per DeepSeek's documented
cache semantics, cache prefixes are independent complete persisted units;
C changed the late suffix, so the previously persisted long prefix unit
could no longer fully match, and C itself may have caused DeepSeek to
detect/persist the common stable core.

Consequence (design): the DeepSeek `late-divergence` scenario is revised to
**four requests** (A/B/C/D) in Phase 0B.3 — D carries a **second distinct
suffix variant** (different from both the original and C's variant 1) so it
cannot simply hit C's request-boundary cache, and it tests whether the
common stable core persisted after C. The conservative settle period moves
to **after C and before D**. The hypothesis is that D MAY show provider
cache reuse closer to Prefixity's structural potential; if it does not,
that evidence is preserved too. No expectation of MATCH is encoded.

Thresholds were **not** tuned: the live B → C PARTIAL_MATCH (0.8985 vs
0.5794, gap 0.3191) remains PARTIAL_MATCH under the unchanged
`REUSE_RATIO_MATCH_TOLERANCE = 0.10`.

See the sanitized regression fixture
`fixtures/traces/20-deepseek-live-late-divergence.json` (request C shape)
and the reconciliation values under
`experiments/runs/deepseek-late-divergence-01/` (gitignored).

## Remaining scientific uncertainty

- These are **single observations** per scenario; providers vary by model,
  region, cache state, and time. Repeated, documented runs are required
  before any claim.
- `structural_reuse_ratio` is structural **potential**, not a prediction of
  the exact provider cache-hit ratio; provider cache persistence is
  best-effort and asynchronous, so realized reuse can lag potential.
- The 10-second settle delay is an experimental control, not a provider SLA
  or a validated optimum.
- No provider tokenizer is implemented; no token multiplier is inferred from
  the observed absolute count differences (e.g. 8063 vs 18115). Only
  proportions are compared.
