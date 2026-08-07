# Phase 0B — Findings

Evidence collected from controlled live validation runs. Each entry is a
**single observation**; repeated, documented runs are required before any
claim.

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
