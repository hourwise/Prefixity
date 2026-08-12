# P0-L5 — llama.cpp conformance adapter and observer

P0-L5 adds the first runtime-specific adapter boundary to the existing
provider-neutral P0-L4 conformance harness. It deterministically projects a
neutral `ConformanceRequest` into the documented llama-server
OpenAI-compatible chat-completions shape and normalizes synthetic
llama-server-shaped response telemetry into the existing P0-L2
`CacheObservation` contract.

P0-L5 has not measured llama.cpp. It contains no HTTP client, socket, process
startup, model loading, GGUF handling, or inference path.

## Architecture

```text
ConformanceExperiment
        ↓
LlamaCppConformanceRunner
        ↓
LlamaCppTransport
        ↓
synthetic fake transport in P0-L5 tests
        ↓
LlamaCppResponse / observer
        ↓
CacheObservation
```

`LlamaCppTransport` is the narrow extension point for a later loopback
transport. `FakeLlamaCppTransport` is the only implementation in this slice.
The runner keeps experiment ID, case ID, mutation class, case relationship,
request/context fingerprints, runtime identity, and observation provenance in
the P0-L4 result wrapper.

## Request projection

`RequestContext` is projected into ordered system, artifact, and current-user
messages. Artifact order, content, whitespace, and user content are copied
without normalization. Ordered tool definitions become ordered llama-server
function tools; tool parameter fields retain their intentional order in the
typed projection. The neutral model and response-format envelope fields are
projected to `model` and `response_format`.

The neutral reasoning/thinking setting has no faithful mapping in this
adapter. A case containing one is rejected with an experiment/case/mutation
context rather than silently omitting the requested setting. This means the
P0-L4 fixture's explicit reasoning mutation is preserved as an unsupported
adapter case until a later design supplies a documented mapping.

## Response normalization

The adapter reads the documented native `timings` fields:

| llama-server field | `CacheObservation` field |
| --- | --- |
| `timings.cache_n` | `accounting.provider_cached_tokens` |
| `timings.prompt_n` | `accounting.fresh_prefill_tokens` |
| `timings.prompt_ms` | `timing.prefill_duration_ms` |
| `timings.predicted_n` | `accounting.output_tokens` |
| `timings.predicted_ms` | `timing.generation_duration_ms` |
| `usage.prompt_tokens` | `accounting.transmitted_input_tokens` |
| `usage.completion_tokens` | `accounting.output_tokens` when consistent |
| `usage.prompt_tokens_details.cached_tokens` | cached tokens when consistent |

The adapter does not infer reconstructed context, wall-clock time, cache
residency, cache hit/miss, persistence, slot state, or performance. Cached
tokens remain distinct from transmitted and fresh-prefill tokens.

Native and compatibility fields are both retained in bounded raw telemetry.
If overlapping native and usage values conflict, normalization returns a
bounded error instead of choosing a value silently. Malformed numeric values
also fail safely. An absent field is `not_observed`; an explicitly reported
zero is `known(0)`.

## Evidence boundary

The protocol fixtures under `fixtures/llama-cpp/` are synthetic validation
inputs. They are not runtime observations. The
`llama-cpp-documented-v1.json` capability fixture records only documented
protocol support as `documented`; it does not assert experimentally observed
behavior. No cache rate, retention duration, slot eviction behavior,
host-RAM benefit, latency reduction, or performance improvement is claimed.

## Next boundary

P0-L6 is the planned first real local llama.cpp conformance/session-cache
experiment using the P0-L4 harness and this P0-L5 adapter. P0-L6 is not
implemented here.
