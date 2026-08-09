# Trace format (Phase 0A, version 2)

This is the normative description of the on-disk trace format. The Rust
structs in `prefixity-core::model` mirror this document.

Version 2 (Phase 0A.1) changes vs version 1:

* `usage` is now `RawUsage`: a `provider_schema` plus a verbatim `raw` map.
  Provider field semantics are **not interchangeable** (Anthropic
  `input_tokens` = uncached remainder; synthetic `input_tokens` = total).
* `ContextBlock` gained structural identity (`semantic_zone`,
  `structural_path`, `role`) used by the structural fingerprint.
* `byte_count` is validated against `content` when content is present.

Phase 1B.4 adds an optional evidence-adapter extension. It does not change
`format_version`: trace-v2 readers remain compatible with historical fixtures.
Evidence-aware adapters may emit `evidence_schema_version: 1`, typed field
provenance, source-explicit message timestamps, and non-content provider
response metadata. The extension is additive and may be omitted by historical
or unrelated trace writers.

## Concepts that must never be conflated

| Concept | Meaning | Source |
| --- | --- | --- |
| Prefixity score | Experimental heuristic (0..1) of stable-prefix suitability | `prefixity_score` |
| Observed prefix reuse | Exact structural prefix match between two recorded traces | `compare` |
| Provider-reported cache reuse | Tokens the provider reports, normalized per schema | `usage` |

A single isolated trace can never prove reuse; all single-trace figures are
called **stable-prefix candidates** and are heuristic only.

## RequestTrace

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `format_version` | integer | yes | Must be `2`. |
| `request_id` | string | yes | Non-empty. |
| `session_id` | string | no | Groups requests. |
| `timestamp` | string | no | Opaque; never parsed, never hashed. |
| `provider` | string | yes | e.g. `synthetic`, `openai`, `anthropic`. |
| `model` | string | yes | e.g. `synthetic-model`. |
| `blocks` | array of ContextBlock | yes | At least one block; contiguous positions. |
| `usage` | RawUsage | no | Provider-specific raw usage, preserved verbatim. |
| `latency` | LatencyInfo | no | Optional latency. |
| `metadata` | object | no | Free-form JSON; preserved verbatim. |
| `evidence_schema_version` | integer | no | Optional additive evidence-adapter version. Phase 1B.4 emits `1`; absent means no evidence-adapter extension is claimed. |
| `provider_response` | ProviderResponseMetadata | no | Explicit provider response identity and bounded response metadata; response IDs are not dependency or safety evidence. |
| `provenance` | object | no | Field-level evidence provenance. Values use `source_explicit`, `derived_structural`, or `unknown` and may include bounded source locators. |

## ContextBlock

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Unique within the trace; bounded length. |
| `source` | string | yes | e.g. `system_policy`, `tool_definition`, `tool_result`, `user_request`, `timestamp`, `git_status`, `file_content`, `repository_map`, `conversation`. Unknown values are accepted and scored conservatively. |
| `position` | integer | yes | Must equal the array index (0-based, contiguous). |
| `content_hash` | string | yes | 64-char lowercase hex SHA-256 of the **content**. If `content` is present, validation recomputes and requires a match. |
| `token_count` | integer | no | Recorder-known token count. If absent, a documented heuristic (chars/4) is used when `content` is present; otherwise tokens are unknown (0 + warning). |
| `byte_count` | integer | yes | Byte count of the content. Must equal the UTF-8 length of `content` when content is present. |
| `timestamp` | number | no | Source-explicit upstream message timestamp. Its presence or age does not imply `stale`, invalidation, supersession, or removability. |
| `content` | string | no | **Optional.** Phase 0 fixtures omit it wherever possible (privacy stance). |
| `semantic_zone` | string | no | e.g. `tools`, `system`, `messages`, `other`. Absent blocks are treated as `other`. Used by zone-aware policies. |
| `structural_path` | string | no | Wire path, e.g. `tools[3]`, `messages[7].content[1]`. |
| `role` | string | no | e.g. `system`, `user`, `assistant`, `tool`, `tool_result`. |
| `sensitivity` | string | no | Informational (e.g. `public`, `confidential`). |
| `dependencies` | array of string | no | Block IDs this block depends on (informational). |
| `lifetime` | integer | no | Observed age in turns. |
| `optional` | boolean | no | Explicitly removable/optional. |
| `required` | boolean | no | **Never removed by any policy, regardless of other flags or size.** |
| `stale` | boolean | no | Explicitly stale (e.g. superseded tool output). |
| `metadata` | object | no | Free-form JSON. |
| `provenance` | object | no | Field-level provenance for additive evidence captured on this block. |

Unknown JSON fields anywhere in a trace are ignored (forward compatibility).

### Evidence-adapter metadata

The Phase 1B.4 extension uses these bounded, non-content structures when an
upstream recorder provides them:

* `provider_response` retains explicit response `id`, `model`, `created`,
  `object`, selected choice/finish metadata, and presence states for selected
  response fields. A provider response ID identifies the response only; it is
  not a tool-call ID, result ID, dependency edge, or safety proof.
* `provenance` maps captured fields to an `origin` of `source_explicit`,
  `derived_structural`, or `unknown`. A source locator may contain the
  trajectory ID, source-file SHA-256, source-event index/ID, and an upstream
  field path. Evaluation-only locators remain sidecar metadata and are not
  planner inputs.
* `ContextBlock.timestamp` is the numeric timestamp explicitly present on the
  corresponding upstream message. Timestamp age is not a staleness signal.
  The adapter does not synthesize `optional`, `required`, `stale`, dependency,
  invalidation, supersession, tool-link, or removability evidence.

`RawUsage` remains the provider-specific usage boundary. The evidence adapter
preserves the complete raw usage map and its explicit schema name. It uses an
existing normalizer only for schemas and fields whose semantics are already
exactly supported; unsupported provider fields remain raw and are not
converted into universal token or savings claims.

### Structural fingerprint

`content_hash` alone is insufficient for prefix comparison: two blocks may
contain identical text in different semantic positions. The **Prefixity
structural fingerprint** is derived from `semantic_zone`, `role`,
`structural_path` and `content_hash`. It is a Prefixity fingerprint — not a
guarantee of any provider's hidden tokenizer or serializer. Without any
structural identity on a block, the fingerprint falls back to the content
hash. Comparison uses the fingerprint for prefix equality; `content_hash`
remains content-level identity.

## RawUsage

| Field | Type | Notes |
| --- | --- | --- |
| `provider_schema` | string | An **explicit versioned API-surface identifier** naming the exact endpoint/request schema the `raw` fields follow: `synthetic`, `openai-chat-completions-v1`, `anthropic-messages-v1`, `deepseek-chat-completions-v1`, or a custom versioned name. |
| `raw` | object | Provider-specific raw usage fields, preserved verbatim and never reinterpreted in place. |

### Trace-v2 legacy alias compatibility

Historical trace-v2 files written before Phase 0B (Phase 0A.1) used the bare
generic provider names `openai`, `anthropic` and `deepseek` as
`provider_schema`. Readers accept these **read-only** as compatibility
aliases for `openai-chat-completions-v1`, `anthropic-messages-v1` and
`deepseek-chat-completions-v1` respectively, so existing valid trace-v2 data
keeps normalizing after the Phase 0B schema change. Writers MUST emit
explicit versioned API-surface identifiers; the generic aliases are never
advertised and must not be emitted. `TRACE_FORMAT_VERSION` is unchanged by
this compatibility layer.

The provider name alone is **not sufficient** to interpret `raw`: a single
provider can expose different usage semantics across different API surfaces
(e.g. OpenAI's Chat Completions vs. the newer Responses API report tokens
differently). The versioned API-surface identifier is what disambiguates them.
Normalization is a separate, offline, schema-aware step
(`prefixity-core::usage`) that produces a provider-independent
`NormalizedUsage` with explicit categories: `total_input_tokens`,
`fresh_input_tokens`, `cache_read_tokens`, `cache_write_tokens`,
`output_tokens`. Values that cannot be derived are left unset — never
invented. Unknown schemas are never interpreted; in particular, an unknown
`openai-*` schema is **not** silently treated as Chat Completions.

Known schema semantics:

| Schema | Meaning |
| --- | --- |
| `synthetic` | `input_tokens` = total; `fresh = total - read - write` is supported. |
| `anthropic-messages-v1` | Messages API: `input_tokens` = uncached remainder; total = input + cache_read_input_tokens + cache_creation_input_tokens. |
| `deepseek-chat-completions-v1` | Chat Completions-compatible: total = prompt_cache_hit_tokens + prompt_cache_miss_tokens; cache writes not reported. |
| `openai-chat-completions-v1` | Chat Completions API: `prompt_tokens` = total; `cached_tokens` nested under `prompt_tokens_details`; cache writes not reported. |
| `openai-responses-v1` | **Reserved.** The OpenAI Responses API surface is not yet implemented; traces carrying this schema are recognized but never interpreted as Chat Completions. |

Per source-of-truth principle 7, provider-reported usage **outranks**
Prefixity's heuristic candidates when determining what actually happened.

## CostProfile (provider profiles)

| Field | Type | Notes |
| --- | --- | --- |
| `name` | string | Profile name. |
| `version` | integer | Profile format version (currently 1; validated). |
| `synthetic` | boolean | Must be `true` for every profile in this repository. |
| `currency` | string | e.g. `USD`. |
| `input_price_per_1m` | number | Price per 1M fresh input tokens. |
| `cache_read_price_per_1m` | number | Price per 1M cache-read tokens. |
| `cache_write_price_per_1m` | number | Price per 1M cache-write tokens (0 if not applicable). |
| `output_price_per_1m` | number | Price per 1M output tokens (0 if out of scope). |
| `notes` | string | Must state the profile is SYNTHETIC unless audited. |

Billing consumes explicit normalized categories. The
`fresh = total - read - write` relationship is only applied where the schema
explicitly supports it (e.g. `synthetic`) or inside a labelled hypothetical
model — never silently assumed for provider-normalized usage.

## Validation invariants (enforced)

- `format_version == 2`.
- `request_id`, `provider`, `model` non-empty.
- At least one block; at most `MAX_BLOCKS` (100,000).
- Positions contiguous `0..n`.
- Block IDs unique and bounded.
- `content_hash` well-formed; verified against `content` when present.
- `byte_count` equals the UTF-8 length of `content` when present.
- Cost-profile `version` matches the supported profile format version.
- Size limits from `prefixity-core::limits` applied.
- Raw usage is opaque to validation; consistency is checked by normalizers.

## Example

```json
{
  "format_version": 2,
  "request_id": "example-1",
  "provider": "synthetic",
  "model": "synthetic-model",
  "blocks": [
    {
      "id": "system-policy",
      "source": "system_policy",
      "position": 0,
      "content_hash": "1111111111111111111111111111111111111111111111111111111111111111",
      "token_count": 1200,
      "byte_count": 4800
    },
    {
      "id": "user-request",
      "source": "user_request",
      "position": 1,
      "content_hash": "5555555555555555555555555555555555555555555555555555555555555555",
      "token_count": 150,
      "byte_count": 600
    }
  ],
  "usage": {
    "provider_schema": "synthetic",
    "raw": {
      "input_tokens": 1350,
      "cache_read_tokens": 1200,
      "cache_write_tokens": 0,
      "output_tokens": 100
    }
  }
}
```
