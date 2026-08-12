# P0-L4 — Cache conformance harness foundation

The cache-conformance harness provides a provider/runtime-neutral experiment
structure for controlled cache-behaviour experiments. It can represent a
baseline, generate reproducible request mutations, execute them through an
in-process transport boundary, and record neutral `CacheObservation` values.

It does not perform inference and does not encode provider assumptions as
truths. The P0-L4 fixture is a small synthetic coding-agent-style workload,
not a benchmark corpus or a performance claim.

## Architecture

The foundation extends the existing offline
`prefixity-controlled-benchmark` crate:

- `ConformanceExperiment` v1 contains the baseline request, ordered
  `ConformanceCase` values, a runtime/profile reference, and bounded metadata.
- `ConformanceCase` v1 contains a stable case ID, `MutationClass`, a complete
  request variant, a `CaseRelationship`, and expected-observation metadata.
- `ConformanceRequest` v1 separates `RequestContext` from `RequestEnvelope`.
  Context has stable instructions, artifact content, current user content,
  and ordered tool definitions. The envelope has model, reasoning, and
  response-format settings.
- `ConformanceResult` v1 records ordered case results, request/context
  fingerprints, runtime identity, embedded neutral observations, completion
  status, and provenance.
- `ConformanceRunner` is the narrow future adapter boundary. P0-L4 supplies
  only `MockConformanceRunner`, an in-process deterministic implementation.

Ordered JSON fields are represented as vectors with duplicate-name validation.
The existing recursive canonical JSON helper sorts JSON object keys but
preserves intentional vector order, so request fingerprints are deterministic
without depending on map iteration or filesystem order.

## Mutation vocabulary

The fixture expresses baseline and exact repeat, beginning-of-stable-content,
end-of-current-content, whitespace-only, JSON field-order, tool-definition
order, optional tool-schema field, one-tool-definition change, model
identifier, reasoning setting, and response-format mutations.

These are experiment classes only. No class means cache hit, cache miss,
invalidation, or provider-specific behavior. Expected cache reuse and cache
write remain `to_be_observed`.

## Observation boundary

The runner uses the P0-L2 `CacheObservation` contract. It records experiment
and case association in the harness result wrapper, while request identity,
artifact identities/hashes, runtime identity, and raw adapter telemetry remain
in their appropriate observation fields.

The mock transport sets token accounting, cache behavior, timing, resources,
and task outcome to `not_observed`. Its raw telemetry identifies the transport
as synthetic and explicitly records `cache_metrics: not_observed`. It never
fabricates cache-hit values, realistic token counts, latency, quality, or
provider-validation evidence.

## Declared versus observed behavior

P0-L2 distinguishes documented, experimentally observed, and unverified
capability evidence. P0-L4 adds no runtime evidence and changes no capability
fixture. A future adapter may record observations only when a real controlled
transport supplies them; the harness will not turn an expectation or fixture
label into a capability claim.

## Later sequence

```text
P0-L4  neutral conformance harness
  ↓
P0-L5  llama.cpp observer/adapter
  ↓
P0-L6  first real local cache/session experiment
  ↓
later  Prefix Diff / Envelope Diff, cache simulator, optimisation
```

Only P0-L4 is implemented here. There is no network access, provider API,
credential handling, live inference, ContextBench dependency, final corpus,
benchmark score, provider ranking, cache simulator, or optimization path.
