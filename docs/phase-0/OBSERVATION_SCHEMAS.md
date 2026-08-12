# Neutral observation schemas

P0-L2 defines three versioned serde contracts in
crates/prefixity-core/src/observation.rs:

| Contract | Version | Rust type |
| --- | ---: | --- |
| Context artifact | 1 | ContextArtifact |
| Cache observation | 1 | CacheObservation |
| Runtime capabilities | 1 | RuntimeCacheCapabilities |

These are neutral data contracts. They do not implement request rewriting,
runtime integration, cache routing, restore logic, or benchmark scoring.

## Explicit absence

Observed<T> has three states:

- known: the value was established;
- unknown: the value cannot currently be established;
- not_observed: the recorder did not collect it.

This distinction prevents a missing metric from becoming a fabricated zero or
false. Serde ignores unknown top-level fields for forward compatibility, while
bounded raw_telemetry and raw_capabilities maps retain adapter-specific data
that is not yet normalized.

## ContextArtifact

ContextArtifact identifies a logical artifact independently of its provider
rendering. origin_id is the stable logical source identity. The optional
content_source_id identifies the concrete representation that supplied the
model-visible contents when it differs from the logical origin. Hash,
revision, provenance, trust, size, cacheability, materialisation, and generic
residency are separate fields.

artifact_type supports text, source files, tool schemas, tool results, images,
video, reasoning state, explicit unknown, and extensible other values.
stability and lifecycle are separate dimensions:
immutable/stable/append-only/volatile does not imply persistent/transient.

Token sizes carry provider, model, and tokenizer scope. A universal token
count is not assumed.

## CacheObservation

CacheObservation records one inference request/run with:

- observation time and provider/model/protocol/runtime/session identity;
- artifact references, serialized request identity, and reusable-prefix
  identity;
- distinct transmitted input, provider-cached, fresh-prefill,
  reconstructed-context, and output token fields;
- optional prefill, TTFT, generation, wall-time, RAM, VRAM, and KV metrics;
- neutral cold/warm, resident/restored/rebuilt, read/write, hit/miss, and
  restore/rebuild observations;
- neutral task/result status and quality/evaluation references; and
- bounded raw adapter telemetry.

Cached tokens are not defined as “tokens removed,” and the schema does not
require accounting fields to sum. Backends that expose only part of the
measurement leave the other fields unknown or not observed.

Conversation identity, cache residency, persistence, and provider request
flags remain separate observations. A store=false-style flag is not treated
as equivalent to conversation chaining, disk persistence, or KV caching.

## RuntimeCacheCapabilities

RuntimeCacheCapabilities describes a provider/model/protocol/runtime
combination. It includes prefix/cache semantics, residency/storage, session
behaviour, metrics, and KV-cache precision support.

Every capability uses:

- supported, unsupported, or unknown support state; and
- documented, experimentally_observed, or unverified evidence state.

Lack of documentation is not encoded as unsupported. Experimental support is
distinct from documented support, and KV precision option support is separate
from evidence that a quality regression is absent.

## Fixtures and evidence boundary

Representative artifact and observation fixtures live under
fixtures/observations/. Capability examples live under
fixtures/capabilities/ for llama.cpp, Ollama, DeepSeek, Meta, Mistral,
Alibaba Model Studio, and Z.AI / GLM.

These fixtures are schema examples, not provider validation. No capability
without approved repository evidence is asserted; the capability examples
remain unknown/unverified. No live provider call, API key, paid inference,
ContextBench integration, or benchmark claim is part of this slice.
