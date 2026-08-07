# Phase 0B — Controlled Live Validation

Status: **harness under development / controlled live validation**. No paid
provider call may be made without explicit opt-in (see below).

## Purpose

Phase 0B is a **falsification exercise**. It does not optimise real agent
workloads. It answers one question:

> Does Prefixity's offline model correspond to what real providers report?

Concretely, can Prefixity:

1. send a tightly controlled synthetic request;
2. preserve the exact request structure used;
3. capture the raw provider response usage;
4. normalize that usage correctly;
5. compare consecutive requests structurally;
6. distinguish **prefixity candidates**, **observed structural prefix reuse**,
   and **provider-reported cache reuse**;
7. explain disagreement between those quantities?

A result showing Prefixity's predictions do **not** correspond to provider
behaviour is valuable evidence. Phase 0B does **not** need to show cost
savings and makes **no pricing claims**.

## Falsification criteria

Stop provider testing if:

- schema-smoke cannot normalize a real response;
- credentials appear in any artifact or log;
- the request-count guard fails;
- a dry run causes network traffic;
- provider redirect/auth behaviour becomes unsafe.

Stop broader Prefixity development for review if:

- structural observed reuse repeatedly has little relationship to provider
  cache behaviour;
- provider usage cannot be normalized reliably;
- provider-native diagnostics already provide all useful information;
- Prefixity overhead begins to distort the experiment materially.

## Architecture

- `prefixity-core` remains authoritative for normalization, structural
  comparison, analysis, cost modelling and policy simulation.
- `crates/prefixity-live` is **disposable experimental infrastructure**: it
  owns controlled HTTP requests, credential acquisition, trace-v2
  conversion, and sanitized artifact writing. It contains no analysis logic.
- HTTP uses `reqwest` (blocking) with `rustls`, TLS verification enabled,
  redirects disabled, an explicit timeout, and **no automatic retry**.
- Provider base URLs are hard-coded/allowlisted in the adapters. No URL is
  ever taken from a fixture or argument.

## Provider differences (known shapes, subject to schema-smoke)

Each adapter emits an **explicit versioned API-surface identifier** in the
trace `RawUsage.provider_schema` (and records it, alongside the concrete
endpoint URL, in the experiment manifest). The provider name alone is not
sufficient — a single provider can expose different usage semantics across
different endpoints — so the schema id, not the provider name, drives
normalization.

| Provider | API-surface schema | Raw fields | Notes |
| --- | --- | --- | --- |
| OpenAI | `openai-chat-completions-v1` | `prompt_tokens`, `completion_tokens`, `prompt_tokens_details.cached_tokens` | `prompt_tokens` = total input; cached tokens nested. No explicit cache control in baseline. |
| Anthropic | `anthropic-messages-v1` | `input_tokens`, `cache_read_input_tokens`, `cache_creation_input_tokens` | `input_tokens` = uncached remainder; total is the sum of the three. Explicit `cache_control` on the large prefix block. |
| DeepSeek | `deepseek-chat-completions-v1` | `prompt_cache_hit_tokens`, `prompt_cache_miss_tokens` | Cache construction may require a prior completed request; the stable-prefix scenario plans three requests and preserves observed behaviour. |

The OpenAI Responses API surface (`openai-responses-v1`) is **reserved** but
not implemented: no live adapter emits it yet, and no unknown `openai-*`
schema is ever interpreted as Chat Completions.

These shapes are assumptions to be **falsified** by schema-smoke, not facts.

## Credentials

- Read **only** from environment variables: `OPENAI_API_KEY`,
  `ANTHROPIC_API_KEY`, `DEEPSEEK_API_KEY`.
- Never accepted as CLI arguments; never persisted; never serialized; never
  included in traces; never logged; never in errors; never printed (even
  partially).
- Never sent to a provider other than the matching adapter.
- `.env` files remain gitignored; no dotenv loading.

## Spend / request guardrails

- No command makes a call without `--execute-live`.
- `--max-requests` default 3, hard ceiling 10 (values above 10 are
  rejected).
- `--max-input-tokens` is a local safety ceiling; the run refuses **before
  any call** if the estimated input would exceed it.
- Requests within an experiment are sequential; there is no concurrency.
- No automatic retry. A timeout or server error returns an error and **stops**.
- These are research guardrails, not a billing guarantee.

## Scenarios (A–D only)

| Id | Scenario | Requests | What it measures |
| --- | --- | --- | --- |
| A | `schema-smoke` | 1 | Does one real response match our usage schema? STOP for that provider if not. |
| B | `stable-prefix` | 2 (3 for DeepSeek) | Provider behaviour when consecutive requests share the same large prefix. |
| C | `early-divergence` | 2 | Change a block near the beginning of request B; Prefixity predicts sharply reduced structural reuse. |
| D | `late-divergence` | 2 | Keep the whole large prefix; change only a small tail; Prefixity observes a large structurally identical prefix. |

Cache-expiry/TTL experiments are **not** part of Phase 0B baseline.

## Timing

Recorded per request: start time, time to response headers, time to first
body byte (approximate with the blocking client), and total time. No
artificial concurrency.

## Artifacts

Live runs write to `experiments/runs/<experiment-id>/` (gitignored):

```
manifest.json
request-01.trace.json
request-02.trace.json
provider-raw-usage-01.json
...
result.json
```

Never stored: authorization headers, API keys, full HTTP header dumps,
private data. "Raw usage" means usage/accounting fields, not unrestricted
raw HTTP traffic. Review artifacts before deliberately committing a
sanitized result later.

## Results and classification

`result.json` contains per-request results, per-pair comparisons, and a
conservative conclusion:

- **MATCH** — provider behaviour broadly corresponds to the structural
  prediction.
- **PARTIAL_MATCH** — some reuse appears but differs materially.
- **NO_MATCH** — provider behaviour contradicts the structural prediction.
- **INCONCLUSIVE** — not enough provider data.
- **SCHEMA_MISMATCH** — the response does not fit our normalizer.

One successful run proves very little: providers vary by model, region,
cache state, and time. Repeated, documented runs across providers are
required before any claim.

## Manual procedure for the first live test (schema-smoke)

1. Set the provider credential in your shell, e.g.:
   `$env:OPENAI_API_KEY = "<your key>"` (never typed into a fixture or file).
2. Dry-run first (zero network, prints exactly what would be sent):
   ```
   cargo run -p prefixity-live -- dry-run --provider openai --model <model> --scenario schema-smoke
   ```
3. Run the single request with explicit opt-in:
   ```
   cargo run -p prefixity-live -- run --provider openai --model <model> --scenario schema-smoke --execute-live
   ```
4. Inspect `experiments/runs/<experiment-id>/result.json`. The conclusion
   must be `MATCH` (or `SCHEMA_MISMATCH`, which means **stop** for that
   provider and report the discrepancy — do not silently adapt unknown
   fields during a paid run).
5. Repeat steps 1–4 for `anthropic` and `deepseek` before running scenarios
   B–D.

### How to abort

- Send `Ctrl-C` in the terminal. The in-flight request may still complete
  (a single request); no further requests are sent because the process has
  stopped.
- To abort before any spend: do not pass `--execute-live`, or set
  `--max-requests 0`-equivalent is invalid, so use `--max-requests 1` and
  schema-smoke only.
- Any error stops the run immediately (no retries, no loop).

### How to inspect artifacts

- `manifest.json` — what was planned (model, scenario, seed, limits).
- `request-NN.trace.json` — the Prefixity v2 trace (structure + hashes +
  raw usage).
- `provider-raw-usage-NN.json` — the verbatim provider usage object.
- `result.json` — reconciliation and conclusion.

## Do not

- Do not run scenarios B–D before each provider's schema-smoke passes.
- Do not enable OpenAI explicit caching in the baseline; we first observe
  native/default behaviour.
- Do not add Anthropic 1-hour TTL or cache-diagnostics requirements yet.
- Do not record real prices or claim savings; pricing remains data and
  synthetic-only for now.
