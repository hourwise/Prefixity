# P0-L6A — Live Experiment Harness Preparation

P0-L6A prepares the local llama.cpp experiment boundary. It does not run
llama-server, load a model, contact localhost, inspect an external benchmark,
or produce live observations. The implementation is in
`crates/prefixity-controlled-benchmark/src/live_harness.rs` and is separate
from ordinary offline commands and tests.

## Evidence boundary

```text
P0-L13 source + certified candidate
              |
              v
       P0-L6A definition and preflight  -- zero network calls --> Prepared
              |
       explicit execute_live=true only
              v
  loopback HTTP -> P0-L5 raw response boundary
              |
              v
  P0-L5 normalization -> CacheObservation -> ConformanceResult
              |
              v
       P0-L8 comparison -> P0-L12 evidence/admission
```

The external model benchmark remains a separate reference stream. It can
describe an external model/runtime benchmark, but cannot by itself validate a
Prefixity candidate. Candidate evaluation remains the controlled
control/treatment comparison defined here.

## Safety and opt-in

The transport accepts only unauthenticated HTTP URLs whose literal host is
`127.0.0.1`, `::1`, or `localhost`. HTTPS, credentials, redirects, LAN/WAN
hosts, query strings, and fragments are rejected. Connect and request
timeouts are explicit, response bytes are bounded, response JSON is parsed
through P0-L5, and there is no retry, startup, endpoint discovery, model
loading, or runtime tuning logic.

`LlamaCppLiveConfig.execute_live` is false when omitted and must be explicitly
true before the future execution function can use a transport. The preflight
function validates the endpoint, runtime profile/configuration, context and
generation bounds, P0-L13 source/candidate/certificate identities, sequence,
evidence location, and caller provenance without constructing a request to the
endpoint. It returns a machine-readable `Prepared` record with
`network_calls: 0`.

The environment manifest is an explicit, versioned record. Unknown fields are
preserved as `Observed::Unknown`; preparing a manifest does not scan the
machine, query WMI, discover processes, or record usernames and personal
absolute paths. The runtime configuration records the selected profile and
caller-supplied build/model/runtime details without embedding a current Qwen
or other model-specific configuration.

## Fixed initial sequence

The first sequence is immutable and structurally comparable:

| Step | Role | Relation |
| --- | --- | --- |
| A1 | control | initial control |
| A2 | control | exact repeat of A1 |
| C1 | candidate treatment | certified P0-L13 treatment |
| C2 | candidate treatment | exact repeat of C1 |
| B1 | interference | deterministic bounded early-different request |
| A3 | control | return to A1 |
| C3 | candidate treatment | return to C1 |

B1 is derived deterministically from the source request with a bounded marker
at the beginning of the system instruction. It is an interference case, not a
tuning or optimization step. No alternative ordering is selected at runtime.

## Evidence states and failure handling

The harness separates `prepared`, `executed`, `normalized`, `admitted`,
`partial`, and `failed` states. Raw evidence contains request/response
fingerprints, status, bounded body size/hash, bounded native telemetry, and
timing. It never stores full generated model text, credentials, or absolute
machine paths. Normalized observations are produced only through the existing
P0-L5 adapter and are not copied into raw evidence.

The first endpoint, malformed-response, timeout, server, context-limit,
normalization, or evidence failure aborts the sequence. An incomplete or
failed record cannot contain a final normalized result and cannot be admitted
as a complete experiment. Evidence persistence is a later caller concern and
is represented as a bounded failure class rather than an implicit retry.

Semantic experiment identity is derived from the versioned definition,
source/candidate/certificate fingerprints, runtime/model configuration, and
fixed sequence. Volatile timestamps are not part of that identity; temporal
provenance is caller-supplied separately.

## Current status

P0-L6A preparation is implemented and validated offline. P0-L6 live execution
remains environment-blocked/live-run-pending because a suitable local
llama-server and model have not been established. No live observation,
benchmark score, cache claim, performance claim, or P0-L14 work is implied by
this preparation slice.
