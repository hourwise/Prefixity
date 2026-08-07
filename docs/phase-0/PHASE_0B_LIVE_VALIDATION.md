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
7. reconcile those quantities **by proportion** while keeping their distinct
   measurement bases explicit (see "Measurement units" below)?

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
| DeepSeek | `deepseek-chat-completions-v1` | `prompt_cache_hit_tokens`, `prompt_cache_miss_tokens` | **First live provider for the current validation sequence.** Model: `deepseek-v4-flash` (never the retired `deepseek-chat` / `deepseek-reasoner` aliases). Thinking is explicitly disabled (`thinking.type=disabled`) so the run measures prompt/cache behaviour, not reasoning; temperature stays 0. Cache construction is async/best-effort and may need a prior completed request, so B–D each prime with A then B and apply a 10 s settle delay before the measured third request (see "Cache settling"). |
| OpenAI | `openai-chat-completions-v1` | `prompt_tokens`, `completion_tokens`, `prompt_tokens_details.cached_tokens` | `prompt_tokens` = total input; cached tokens nested. No explicit cache control in baseline. No `thinking` field is sent. |
| Anthropic | `anthropic-messages-v1` | `input_tokens`, `cache_read_input_tokens`, `cache_creation_input_tokens` | `input_tokens` = uncached remainder; total is the sum of the three. Explicit `cache_control` on the large prefix block. |

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
- `--max-estimated-input-tokens` is a conservative **local Prefixity
  estimate** (chars/4) safety ceiling; the run refuses **before any call**
  if the estimated input would exceed it. It is **NOT** a provider
  billing/tokenizer guarantee — the first live run measured 563 Prefixity
  estimated tokens against 1215 provider tokens for the same request.
- Requests within an experiment are sequential; there is no concurrency.
- No automatic retry. A timeout or server error returns an error and **stops**.
- These are research guardrails, not a billing guarantee.

## Measurement units

Absolute token counts from different tokenizers are **not directly
comparable**. Prefixity preserves both systems explicitly and never silently
converts one into the other:

- **Prefixity estimated unit** — Prefixity's chars/4 estimate
  (fields such as `prefixity_estimated_tokens`,
  `observed_structural_reuse_estimated_tokens`).
- **Provider token unit** — tokens reported by the provider
  (fields such as `provider_reported_total_input_tokens`,
  `provider_reported_cache_read_tokens`).

Pair reconciliation therefore compares **proportions**, each relative to its
own denominator. For request B:

```
structural_reuse_ratio =
    observed_structural_reuse_estimated_tokens
    / prefixity_estimated_input_tokens_for_request_B

provider_cache_reuse_ratio =
    provider_reported_cache_read_tokens
    / provider_reported_total_input_tokens_for_request_B
```

Reports phrase this as e.g. "structural reuse 97.8% of Prefixity-estimated
request context" and "provider cache reuse 96.9% of provider-reported input
tokens" — never as "7,800 Prefixity tokens equals 16,900 provider tokens".

**Remaining limitation:** ratio comparison is better than absolute
cross-tokenizer comparison but is still not exact. Provider total input may
include serialization/tokenization overhead not represented by Prefixity's
three structural blocks, and the two proportions are still measured in
different units. This is documented Phase 0B research, not a validated
metric (see `PHASE_0B_FINDINGS.md`).

## Scenarios (A–D only)

| Id | Scenario | Requests | What it measures |
| --- | --- | --- | --- |
| A | `schema-smoke` | 1 | Does one real response match our usage schema? The endpoint schema's defining fields must be derivable — for `deepseek-chat-completions-v1` that is `total_input_tokens`, `fresh_input_tokens` and `cache_read_tokens` (i.e. hit + miss input semantics). Completion/output tokens **alone are not a match**. STOP for that provider if not. |
| B | `stable-prefix` | 2 (OpenAI/Anthropic); 3 (DeepSeek) | Provider behaviour when consecutive requests share the same large prefix. DeepSeek primes with A then B and measures the third request (C). |
| C | `early-divergence` | 2 (OpenAI/Anthropic); 3 (DeepSeek) | Change a block near the beginning. OpenAI/Anthropic change the header at B. DeepSeek keeps the header unchanged through A and B (they establish the common prefix) and changes it only at C; the important comparison is B → C. |
| D | `late-divergence` | 2 (OpenAI/Anthropic); 3 (DeepSeek) | Keep the whole large prefix; change only a small tail. DeepSeek runs A, B, C with the tail changed at C; the important comparison is B → C. |

DeepSeek's B–D priming sequence (per provider/scenario plan, not hidden
arithmetic):

```
stable-prefix:   A = prefix + tail A;  B = prefix + tail B;  C = prefix + tail C
late-divergence: A = prefix + tail A;  B = prefix + tail B;  C = prefix + changed tail C
early-divergence:A = header + prefix + tail A;  B = header + prefix + tail B;
                 C = CHANGED header + prefix + tail C
```

### Cache settling (DeepSeek)

Official DeepSeek Context Caching documentation states that cache
construction is **asynchronous and best-effort** and can take seconds, and
that common-prefix persistence may be established after multiple requests.
Phase 0B therefore applies a conservative **10-second settle period after
request B and before request C** for every DeepSeek B–D scenario:

```
pre_request_delay_ms:  A = 0;  B = 0;  C = 10000
```

This is an **experimental control**, not a provider SLA or a required value,
and it is not a scientifically validated optimum. A zero cache hit after
settling remains evidence to investigate — it does **not** automatically
prove that structural reuse is incorrect. B is deliberately given no delay:
A/B must first establish the common prefix, and the experiment must not
encode an assumption that B must report zero reuse.

Dry runs report the full per-request delay plan but **never sleep** and make
zero network requests.

Cache-expiry/TTL experiments are **not** part of Phase 0B baseline.

## Timing

Recorded per request: start time, time to response headers, time to first
body byte (approximate with the blocking client), and total time. No
artificial concurrency.

## Artifacts

Live runs always write to `experiments/runs/<experiment-id>/` (gitignored).
The public CLI has **no** user-selectable runs directory: the artifact root
is fixed, so a live run can never be pointed at an arbitrary filesystem
destination. An existing experiment destination that is a symlink is
rejected. (Tests inject temporary roots internally.)

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
conservative conclusion. Per-pair comparisons record both measurement bases
and the two reuse **proportions**, plus their absolute difference
(`reuse_ratio_difference`). The old field that subtracted provider tokens
from Prefixity estimated tokens (mixing incompatible units) has been
removed.

Classification is **proportion-based** with Phase 0B experimental
thresholds (`REUSE_RATIO_MATCH_TOLERANCE = 0.10` absolute percentage-point
distance; effectively-zero ≤ 0.05; clearly-substantial ≥ 0.20):

- **MATCH** — both proportions effectively zero, or the absolute
  proportion distance is within the 0.10 threshold.
- **PARTIAL_MATCH** — material but nonzero proportion disagreement (> 0.10)
  with meaningful reuse on both sides.
- **NO_MATCH** — one side effectively zero while the other is clearly
  substantial.
- **INCONCLUSIVE** — provider cache-read or total-input unavailable, or the
  Prefixity estimated input denominator is zero.
- **SCHEMA_MISMATCH** — the response does not fit our normalizer.

These thresholds are research defaults, not scientifically validated; see
`PHASE_0B_FINDINGS.md` for the live evidence that motivated the
proportion-based approach.

One successful run proves very little: providers vary by model, region,
cache state, and time. Repeated, documented runs across providers are
required before any claim.

## Manual procedure for the first live test (schema-smoke)

1. The first live provider in the current validation sequence is **DeepSeek**
   with model **`deepseek-v4-flash`**. Do not use the retired
   `deepseek-chat` / `deepseek-reasoner` aliases.
2. Enter the credential **interactively into the process environment** — do
   **not** type the key into a command line that may be retained in shell
   history. The exact Windows procedure will be supplied immediately before
   the live test.
3. Dry-run first (zero network, prints exactly what would be sent):
   ```
   cargo run -p prefixity-live -- dry-run --provider deepseek --model deepseek-v4-flash --scenario schema-smoke
   ```
4. Run the single request with explicit opt-in:
   ```
   cargo run -p prefixity-live -- run --provider deepseek --model deepseek-v4-flash --scenario schema-smoke --execute-live
   ```
5. Inspect `experiments/runs/<experiment-id>/result.json`. The conclusion
   must be `MATCH` (or `SCHEMA_MISMATCH`, which means **stop** for that
   provider and report the discrepancy — do not silently adapt unknown
   fields during a paid run). For DeepSeek, `MATCH` requires the
   `total_input_tokens` / `fresh_input_tokens` / `cache_read_tokens`
   categories to have been derived from hit + miss; completion tokens alone
   are not a match.
6. Repeat steps 2–5 for `anthropic` and `openai` before running scenarios
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
- Do not use `deepseek-chat` or `deepseek-reasoner` aliases for live runs;
  use `deepseek-v4-flash` with thinking explicitly disabled.
- Do not type a real API key into any command line that could be retained in
  shell history; enter it interactively into the process environment.
- Do not enable OpenAI explicit caching in the baseline; we first observe
  native/default behaviour.
- Do not add Anthropic 1-hour TTL or cache-diagnostics requirements yet.
- Do not record real prices or claim savings; pricing remains data and
  synthetic-only for now.
