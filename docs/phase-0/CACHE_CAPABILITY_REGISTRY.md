# P0-L9 — Cache Capability Registry and Matrix

P0-L9 makes the existing P0-L2 `RuntimeCacheCapabilities` records queryable
without creating a second capability schema. The registry answers:

> What does Prefixity currently know about a runtime/provider/model/protocol
> combination?

The capability matrix answers:

> How do the recorded capability and evidence states compare?

The research-gap report answers:

> What do we still not know?

## Registry boundary

`CapabilityRegistry` retains each profile's provider, model, protocol, runtime,
and runtime-version identity. A profile fingerprint is the canonical hash of
the complete semantic `RuntimeCacheCapabilities` value. It excludes ingestion
timestamps and path metadata, so identical semantic profiles have identical
identities while a capability or evidence change produces a different
identity. Profiles are sorted by fingerprint and duplicate semantic profiles
are rejected.

The first registry is loaded through one explicit path list:

- `llama.cpp` neutral profile;
- `llama.cpp` documented protocol profile;
- Ollama;
- DeepSeek;
- Meta;
- Mistral;
- Alibaba Model Studio;
- Z.AI / GLM.

The loader reads only the approved files under `fixtures/capabilities`; it does
not scan directories or contact a provider. Fixture ingestion is retained as
bounded registry provenance and does not promote a fixture to runtime evidence.

## Evidence states

The matrix preserves the P0-L2 support/evidence pair. In particular:

- `supported_documented` is a documented support claim;
- `supported_observed` is an experimentally observed support claim;
- `unsupported_documented` and `unsupported_observed` remain distinct;
- `unknown_documented` and `unknown_unverified` remain distinct from all
  negative claims.

Unknown does not mean unsupported. Absence of documentation is not evidence of
unsupported capability. Documented does not mean experimentally reproduced by
Prefixity. Synthetic fixture provenance is retained separately and never
promotes a documented or unverified field to `experimentally_observed`.

One provider name does not imply one universal cache implementation across
every model, protocol, or runtime version. A query for an explicit identity
value does not treat an unknown value as a wildcard.

## Capability matrix example

The following is a readable projection generated from the approved fixture
records, using the same `CapabilityState` labels as the structured matrix. It
is not a provider ranking or a performance score. All seven non-documented
profiles retain their fixture-declared unknown/unverified values.

| capability | llama.cpp documented | llama.cpp neutral | Ollama | DeepSeek | Meta | Mistral | Alibaba Model Studio | Z.AI / GLM |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| prefix reuse | supported_documented | unknown_unverified | unknown_unverified | unknown_unverified | unknown_unverified | unknown_unverified | unknown_unverified | unknown_unverified |
| host RAM cache | unknown_unverified | unknown_unverified | unknown_unverified | unknown_unverified | unknown_unverified | unknown_unverified | unknown_unverified | unknown_unverified |
| cached-token telemetry | supported_documented | unknown_unverified | unknown_unverified | unknown_unverified | unknown_unverified | unknown_unverified | unknown_unverified | unknown_unverified |
| prompt evaluation duration | supported_documented | unknown_unverified | unknown_unverified | unknown_unverified | unknown_unverified | unknown_unverified | unknown_unverified | unknown_unverified |

The current matrix therefore has zero experimentally observed capability
fields. P0-L5's synthetic llama.cpp protocol tests and documented fixture do
not change that result.

## Queries and gaps

`CapabilityQuery` supports typed provider, model, protocol, runtime,
runtime-version, capability, support, evidence, and profile-origin filters.
Results and selected matrix profiles/capabilities have deterministic ordering.
The API is intentionally not a general query language.

`ResearchGapReport` counts known, unknown, and experimentally observed fields
by capability and by profile. A known unsupported claim counts as known, while
an unknown field counts as a research gap; no gap is relabelled as a defect or
as unsupported capability. A profile with zero experimentally observed fields
is not labelled experimentally validated.

## Scope boundary

P0-L9 is offline registry and research infrastructure. It does not perform
live inference, network/provider calls, runtime installation, cache probing,
cache prediction, simulation, routing, rewriting, KV quantisation, benchmark
scoring, ContextBench integration, or P0-L10 work. P0-L6 remains
environment-blocked because the existing usable `llama-server` and GGUF were
not available.
