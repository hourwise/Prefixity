# Synthetic fixtures — README

All files in `fixtures/traces/` are **synthetic**. They contain:
- no real credentials or API keys;
- no private source code;
- no real prompt content (blocks carry fabricated hashes and metadata only,
  consistent with the Phase 0 stance of avoiding complete prompt retention).

Hashes are well-formed 64-char hex strings but are **not** digests of real
content; they are deliberately patterned (e.g. `1111...`, `2a2a...`) so
fixtures are obviously synthetic and easy to eyeball.

## Scenario map

| File | Scenario | Commands that demonstrate it |
| --- | --- | --- |
| `01-stable-prefix.json` / `01-stable-prefix-turn2.json` | Large stable prefix remains identical between turns; only the user request changes | `compare` (reuse 9500 of 9700); `analyse` on either file |
| `02-early-timestamp-break.json` / `02-early-timestamp-break-turn2.json` | A timestamp near the start changes and destroys downstream prefix reuse | `compare` (divergence at position 0, reuse 0) |
| `03-tool-order-break.json` / `03-tool-order-break-turn2.json` | Same logical tools, different order (A,B,C → A,C,B) | `compare` (reordering detected at position 2) |
| `04-large-tool-output.json` | Large volatile tool result dominates fresh context | `analyse` (top fresh contributor = tool-result-large, 30,000 tokens) |
| `05-cache-write-not-economic.json` / `05-cache-write-not-economic-turn2.json` | Small reuse + large rewrite; caching is a net loss under an expensive-write profile | `compare --provider-profile provider-profiles/synthetic-cache-write-expensive.json` |
| `06-context-reduction-wins.json` | Removing optional/stale material saves more than cache placement | `simulate --policy stable-prefix` vs `simulate --policy defer-volatile` / `combined` |
| `07-already-optimal.json` | Existing layout is already effectively optimal | `analyse` (no structural change recommended); all `simulate` policies produce zero change |
| `08-unsafe-pruning-example.json` | A block marked optional+stale but required must never be removed | `simulate --policy defer-volatile` / `combined` (block retained, warning emitted) |

## Quick command examples (run from the repository root)

```
cargo run -p prefixity-cli -- validate fixtures/traces/01-stable-prefix.json
cargo run -p prefixity-cli -- analyse fixtures/traces/04-large-tool-output.json
cargo run -p prefixity-cli -- compare fixtures/traces/03-tool-order-break.json fixtures/traces/03-tool-order-break-turn2.json
cargo run -p prefixity-cli -- compare fixtures/traces/05-cache-write-not-economic.json fixtures/traces/05-cache-write-not-economic-turn2.json --provider-profile provider-profiles/synthetic-cache-write-expensive.json
cargo run -p prefixity-cli -- simulate fixtures/traces/06-context-reduction-wins.json --policy combined --provider-profile provider-profiles/synthetic-example.json
```

Add `--json` to any command for stable, machine-readable output.
