# Synthetic fixtures — README

All files in `fixtures/traces/` are **synthetic**. They contain:
- no real credentials or API keys;
- no private source code;
- no real prompt content (blocks carry fabricated hashes and metadata only,
  consistent with the Phase 0 stance of avoiding complete prompt retention).

Hashes are well-formed 64-char hex strings but are **not** digests of real
content; they are deliberately patterned (e.g. `1111...`, `2a2a...`) so
fixtures are obviously synthetic and easy to eyeball. The exceptions are
fixtures **12** and **13**, which embed a tiny shared string with its *real*
SHA-256 digest to demonstrate structural-fingerprint behaviour, and
**17**, which is a **sanitized copy of the first real DeepSeek live
schema-smoke** (safe usage/accounting values only — no credentials, no
authorization headers, no provider request id, no full provider response).

## Scenario map

| File | Scenario | Commands that demonstrate it |
| --- | --- | --- |
| `01-stable-prefix.json` / `01-stable-prefix-turn2.json` | Large stable prefix remains identical between turns; only the user request changes | `compare` (observed reuse 9500 of 9700); `analyse` on either file |
| `02-early-timestamp-break.json` / `02-early-timestamp-break-turn2.json` | A timestamp near the start changes and destroys downstream prefix reuse | `compare` (divergence at position 0, observed reuse 0) |
| `03-tool-order-break.json` / `03-tool-order-break-turn2.json` | Same logical tools, different order (A,B,C → A,C,B) | `compare` (reordering detected at position 2) |
| `04-large-tool-output.json` | Large volatile tool result dominates fresh context | `analyse` (top fresh contributor = tool-result-large, 30,000 tokens) |
| `05-cache-write-not-economic.json` / `05-cache-write-not-economic-turn2.json` | Small reuse + large rewrite; caching is a net loss under an expensive-write profile | `compare --provider-profile provider-profiles/synthetic-cache-write-expensive.json` |
| `06-context-reduction-wins.json` | Removing optional/stale material saves more than cache placement | `simulate --policy stable-prefix` vs `simulate --policy defer-volatile` / `combined` |
| `07-already-optimal.json` | Existing layout is already effectively optimal | `analyse` (no structural change recommended); all `simulate` policies produce zero change |
| `08-unsafe-pruning-example.json` | A block marked optional+stale but required must never be removed | `simulate --policy defer-volatile` / `combined` (block retained, warning emitted) |
| `09-anthropic-usage-semantics.json` | Anthropic-shaped raw usage (`input_tokens` = uncached remainder) | `analyse` (normalized total = 5000, fresh = 500, read = 4000, write = 500) |
| `10-deepseek-usage-semantics.json` | DeepSeek-shaped raw usage (hit + miss = total) | `analyse` (normalized total = 5000, fresh = 1000, read = 4000, no write) |
| `11-openai-usage-semantics.json` | OpenAI-shaped raw usage (nested `cached_tokens`) | `analyse` (normalized total = 5000, fresh = 1000, read = 4000, no write) |
| `12-same-content-different-role.json` | Same text, same zone, different role | `analyse` — content hashes match but structural fingerprints differ |
| `13-same-content-different-zone.json` | Same text, different zone | `analyse` — content hashes match but structural fingerprints differ |
| `14-first-request-no-observed-reuse.json` | Single trace, no provider usage; high scores are candidates only | `analyse --provider-profile ...` (cache read = 0, all billed fresh; recommendation says a single trace cannot prove reuse) |
| `15-history-proves-prefix-reuse.json` / `15-history-proves-prefix-reuse-turn2.json` | History proves observed reuse (5200) while the provider reports a different figure (5000) | `compare` (observed reuse kept separate from provider-reported cache read) |
| `16-global-reorder-would-be-unsafe.json` | A naive global stable-first sort would cross zones / reorder messages | `simulate --policy stable-prefix` (no safe relocation; unsafe moves deferred) |
| `17-deepseek-live-schema-smoke.json` | **Sanitized** real DeepSeek live schema-smoke (2026-08-07, `deepseek-v4-flash`): hit 0 + miss 1215 = total 1215, cache read 0, output 1 | `analyse` (normalized total = 1215, fresh = 1215, read = 0, output = 1) |
| `18-deepseek-live-stable-prefix.json` | **Sanitized** real DeepSeek live stable-prefix, request B (2026-08-07, `deepseek-v4-flash`): hit 18048 + miss 13 = total 18061, output 1 | `analyse` (normalized total = 18061, fresh = 13, read = 18048); live reconciliation MATCH (structural 0.9983 vs provider 0.9993) |

## Quick command examples (run from the repository root)

```
cargo run -p prefixity-cli -- validate fixtures/traces/01-stable-prefix.json
cargo run -p prefixity-cli -- analyse fixtures/traces/04-large-tool-output.json
cargo run -p prefixity-cli -- analyse fixtures/traces/09-anthropic-usage-semantics.json --provider-profile provider-profiles/synthetic-example.json
cargo run -p prefixity-cli -- compare fixtures/traces/03-tool-order-break.json fixtures/traces/03-tool-order-break-turn2.json
cargo run -p prefixity-cli -- compare fixtures/traces/15-history-proves-prefix-reuse.json fixtures/traces/15-history-proves-prefix-reuse-turn2.json
cargo run -p prefixity-cli -- compare fixtures/traces/05-cache-write-not-economic.json fixtures/traces/05-cache-write-not-economic-turn2.json --provider-profile provider-profiles/synthetic-cache-write-expensive.json
cargo run -p prefixity-cli -- simulate fixtures/traces/16-global-reorder-would-be-unsafe.json --policy stable-prefix
```

Add `--json` to any command for stable, machine-readable output.
