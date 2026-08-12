# P0-L8 — Evidence-backed cache regression diagnostics

P0-L8 adds a bounded comparison layer for two versioned
`prefixity_core::observation::CacheObservation` values. It keeps references,
identities, fingerprints, and directional deltas rather than copying raw
observations or raw adapter telemetry.

## Three-layer model

The diagnostic has three deliberately separate layers:

1. **Structural request evidence.** P0-L7 `PrefixDiff`, `EnvelopeDiff`, and
   `RequestDiff` describe what changed in model-visible context and request
   envelope. A request mutation is not itself a cache hit, miss, invalidation,
   or performance result.
2. **Observed runtime evidence.** P0-L8 compares the independently reported
   fields of two `CacheObservation` records. Runtime/backend/provider/model/
   protocol/profile identity, experiment/case identity, request/context
   fingerprints, envelope references, and evidence source are retained as a
   compact `ObservationReference`.
3. **Association-only diagnostic.** `CacheDiagnostic` combines the first two
   layers and emits a bounded `CacheRegressionAssessment` plus a deterministic
   `EvidenceStatement`. Its causality status is always `not_established`.

The layers must not be collapsed into a universal score. Structural change can
be associated with an observed metric change while other explanations remain
open.

## Comparability and evidence states

Comparability is one of `directly_comparable`, `partially_comparable`,
`incomparable`, or `insufficient_evidence`. A known model/runtime/provider/
protocol/profile mismatch is incomparable. Missing fingerprints or identity
evidence is not treated as a matching value. A partial identity can support
limited directional reporting, but it is not upgraded to direct evidence.

Every token, timing, and resource metric preserves `known`, `unknown`, and
`not_observed`. In particular, known zero is a measured zero. Token fields are
compared independently: transmitted input, provider-cached, fresh-prefill,
reconstructed-context, and output tokens are never combined into an assumed
accounting equation. Cached tokens are not interpreted as “tokens removed”.
Known token scope mismatches make that metric unavailable rather than silently
normalising provider/model/tokenizer units.

Derived ratios are explicitly marked derived and name their denominator. The
Phase-0 ratios use transmitted input tokens as the denominator and are absent
when either value is missing, scoped incompatibly, or has a zero denominator.
Relative timing/resource changes use the left-hand known value only as an
explicit normalisation denominator; they are not significance tests or scores.

Directional vocabulary is limited to `increased`, `decreased`, `unchanged`,
and `unavailable`. The assessment vocabulary is bounded to insufficient
evidence, no observed cache reuse change, observed reuse increase, observed
reuse decrease, mixed observations, and incomparable. It intentionally avoids
“better” and “worse”.

## Evidence source classes

Diagnostics can identify evidence as synthetic protocol/test,
documented capability, experimentally observed runtime, or unknown/unverified.
Runtime capability contracts remain documentation about a runtime; they are
not substituted for an observation and do not change observation deltas.
P0-L8 has no live runtime observation and does not make provider or model
performance claims.

The test fixtures cover exact-repeat reuse increase, late partial reuse,
apparent reuse decrease, fresh-prefill increase, timing-only evidence, missing
telemetry, explicit zero, incompatible model/runtime identity, mixed signals,
and envelope-only structural change. They are deterministic synthetic tests,
not provider validation.

## Scope boundary

P0-L6 remains environment-blocked because a usable local llama.cpp server and
model were not available. P0-L8 does not install a runtime or model, contact a
provider, add ContextBench, perform runtime cache probing, infer causality,
aggregate repetitions statistically, or publish a performance claim.
