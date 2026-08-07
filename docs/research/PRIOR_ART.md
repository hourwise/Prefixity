# Prior art

Research that shaped the Prefixity hypothesis. This document records *what
was learned*, not a survey. Dates are deliberately omitted; provider pricing
and product details change.

## Systems reviewed

- **Provider-native prompt/prefix caching** — OpenAI, Anthropic, DeepSeek and
  others expose prefix caching with different mechanisms, prices, TTLs and
  reporting. A core Phase 0 conclusion: cache economics must be **data**, not
  hard-coded fact.
- **CacheLane** — a middleware approach to managing prompt caching lanes.
  Relevant because it assumes the stable-prefix problem is worth solving at
  the proxy layer.
- **Prefixion** — **important negative evidence.** A developer reports having
  built a multi-layer volatility/hash/provider-caching architecture that did
  not materially reduce costs in practice. This is the single strongest
  reason Phase 0 exists to *test*, not to *assume*.
- **Graft / Madar** — context-reduction research suggesting that avoiding
  unnecessary context and tool calls may be more valuable than merely caching
  large prompts. This motivates fixtures 04/06 (volatile tool output and
  optional/stale material dominating fresh context).
- **llmtrim** — trimming/prompt-compaction tooling; evidence that simple
  token-count reduction is tempting and that correctness effects are the hard
  part (compression is therefore reserved, not implemented, in Phase 0).
- **VS Code Cache Explorer** — a UI for inspecting provider cache state;
  evidence that observability of caching is a real need, but also that
  observability alone is not a novel product.
- **LMCache / vLLM automatic prefix caching** — server-side KV/prefix caching.
  Prefixity is deliberately *not* reimplementing these; they are the
  providers' and hosts' problem space.
- **GPTCache** — semantic response caching. Explicitly out of scope for
  Phase 0 (semantic caching can change answers; Prefixity does not do it).
- **CAPC (Cache-Aware Prompt Compression)** — research on the interaction
  between compression and caching.

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
