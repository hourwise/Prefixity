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
