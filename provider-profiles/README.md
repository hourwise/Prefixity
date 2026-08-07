# Provider profiles

Cost profiles are **data**, not hard-coded facts. Provider pricing and cache
rules change, so Phase 0 never embeds "current" prices as ground truth.

## Rules

1. Every profile in this directory is **SYNTHETIC** (`"synthetic": true`) and
   states so in its `notes`. None of them represent real, current provider
   pricing.
2. A real profile for a later, audited phase would be supplied externally and
   set `"synthetic": false` only after review.
3. All prices are per **one million tokens** in `currency` units.
4. Fields: `input_price_per_1m`, `cache_read_price_per_1m`,
   `cache_write_price_per_1m`, `output_price_per_1m`. Use `0.0` for prices
   that do not apply.

## Profiles provided

| File | Purpose |
| --- | --- |
| `synthetic-example.json` | Built-in default used by `prefixity simulate` when no profile is given. |
| `synthetic-openai-like.json` | Shape with cheap cache reads (synthetic only). |
| `synthetic-anthropic-like.json` | Shape with distinct read/write prices (synthetic only). |
| `synthetic-deepseek-like.json` | Shape with very low absolute prices (synthetic only). |
| `synthetic-cache-write-expensive.json` | Expensive cache writes; used by fixture 05 to show caching can be a net loss. |

## Usage

```
prefixity analyse fixtures/traces/04-large-tool-output.json --provider-profile provider-profiles/synthetic-example.json
```

See `docs/phase-0/TRACE_FORMAT.md` for the profile JSON schema.
