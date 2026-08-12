# P0-L7 — Prefix Diff and Request Envelope Diff

P0-L7 adds provider/runtime-neutral diagnostics for comparing two neutral
`ConformanceRequest` values. The diagnostics describe structural differences;
they do not optimize requests or predict cache behavior.

## Prefix Diff

`prefix_diff(left, right)` compares only the model-visible `RequestContext`:

- system/instruction text;
- ordered context artifacts, identities, and contents;
- current-user content;
- ordered tools, descriptions, and ordered schema fields.

The versioned `PrefixDiff` result contains both context fingerprints, an
`identical` flag, a bounded list of `DiffChange` records, the first change,
and a `CommonPrefixMeasurement`. Structural units count the common leading
request components. Artifact and tool common-unit counts are reported
separately. Directly comparable text fields report a byte common-prefix
offset. No tokenizer or token estimate is introduced, so token-level prefix
units remain `not_observed`.

Each change has a stable path, neutral category, bounded value summaries,
presence/content/order flags, and an optional sequence index. Value summaries
contain type, hash, byte size, and only a short explicitly bounded text
preview. Full context values are never copied into diagnostics.

Whitespace-only changes are classified with `whitespace_only: true` without
normalizing either input. Ordered artifacts, tools, and schema fields are
compared as sequences. When members are the same but their order changes, the
result reports an ordering category rather than falsely reporting removals and
additions. Added, removed, changed, and optional schema-field cases remain
distinct.

## Request Envelope Diff

`envelope_diff(left, right)` compares only the currently represented
`RequestEnvelope` fields: model, reasoning, and response format. It is
separate from Prefix Diff, so a model-only or reasoning-only change produces
an identical Prefix Diff and a changed Envelope Diff.

`request_diff(left, right)` combines both results and supplies a bounded
interpretation with independent `context: identical/changed` and
`envelope: identical/changed` states.

Example context-plus-envelope diagnostic:

```text
REQUEST DIFF

Context
  changed: yes
  first divergence: context.tools[2].parameters.fields[1].value
  category: ordered schema/value change

Envelope
  model: unchanged
  reasoning: unchanged
  response format: unchanged

Cache impact
  unknown
  no runtime evidence attached
```

Example envelope-only diagnostic:

```text
Context
  identical

Envelope
  reasoning: changed

Cache impact
  unknown
```

## Evidence boundary

P0-L7 does not answer whether a difference caused a cache hit, cache miss,
invalidation, latency change, or performance change. Every new diagnostic
reports `cache_impact: unknown`. A future evidence-attached runtime result may
use the reserved `evidence_supported` state for a demonstrated mutation, but
no such inference is implemented here.

P0-L6 remains `environment-blocked`: this repository has no existing usable
`llama-server` binary and no existing suitable GGUF model in the inspected
environment. No live observation is fabricated or admitted by P0-L7. P0-L8
and later work remain outside this slice.
