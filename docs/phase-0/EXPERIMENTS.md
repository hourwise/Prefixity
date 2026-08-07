# Phase 0 experiments

Three future experimental groups are documented for a later phase (0B live
harness and beyond). Phase 0 builds the offline machinery these experiments
will use; it does not run them.

## EXPERIMENT A — Cache only

**Question:** does stable-prefix organisation of an unchanged context reduce
cost and latency?

- **Baseline request** vs **stable prefix organisation** of the same context.

**Measure:**

- input tokens
- cache read
- cache write
- fresh tokens
- TTFT (time to first token)
- total latency
- cost

**Phase 0 support:** `prefixity analyse` / `prefixity compare` with a
provider profile; fixture `01-stable-prefix` and `03-tool-order-break`.

## EXPERIMENT B — Context reduction

**Question:** does removing/deferring unnecessary context beat cache
placement, and does quality-gated compression help further?

- **Baseline**
- vs **deferred/pruned optional context**
- vs **future quality-gated compression**.

**Measure:** the same as Experiment A **plus task correctness**.

**Phase 0 support:** `prefixity simulate --policy defer-volatile` /
`prune-stale-tool-output`; fixture `06-context-reduction-wins`. Compression
is deliberately **not** implemented (reserved policy name `compression`);
quality cannot be inferred from token counts.

## EXPERIMENT C — Combined

**Question:** what does a full strategy actually achieve?

- **Native caching**
- vs **Prefixity strategy**
- vs **context reduction**
- vs **combined**.

**Primary outcomes are NOT maximum cache hit percentage.** Primary outcomes:

- total economic cost
- fresh input processed
- latency
- correctness / task success
- tool calls

## Principles for all experiments

- Provider-reported cache usage outranks Prefixity's theoretical estimate.
- A lower token count is not automatically a better result if correctness
  degrades.
- Every experiment must be runnable with audited, externally supplied
  provider profiles; nothing in Phase 0 claims real-world prices.

## Phase 0B live harness constraints (design notes only)

A later live harness may submit controlled requests to OpenAI, Anthropic and
DeepSeek. It must ensure:

- API keys never enter fixtures;
- API keys are never logged;
- API keys are never committed;
- `.env` files are gitignored.

Phase 0 does not implement credentials or provider calls.
