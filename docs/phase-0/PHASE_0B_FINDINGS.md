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
