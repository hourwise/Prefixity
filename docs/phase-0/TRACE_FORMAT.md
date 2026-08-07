# Trace format (Phase 0, version 1)

This is the normative description of the on-disk trace format. The Rust
structs in `prefixity-core::model` mirror this document.

## RequestTrace

Top-level object:

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `format_version` | integer | yes | Must be `1`. |
| `request_id` | string | yes | Non-empty. |
| `session_id` | string | no | Groups requests. |
| `timestamp` | string | no | Opaque; never parsed, never hashed. |
| `provider` | string | yes | e.g. `synthetic`, `openai`, `anthropic`. |
| `model` | string | yes | e.g. `synthetic-model`. |
| `blocks` | array of ContextBlock | yes | At least one block; contiguous positions. |
| `usage` | ProviderUsage | no | Provider-reported usage. |
| `latency` | LatencyInfo | no | Optional latency. |
| `metadata` | object | no | Free-form JSON; preserved verbatim. |

## ContextBlock

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Unique within the trace; bounded length. |
| `source` | string | yes | e.g. `system_policy`, `tool_definition`, `tool_result`, `user_request`, `timestamp`, `git_status`, `file_content`, `repository_map`, `conversation`. Unknown values are accepted and scored conservatively. |
| `position` | integer | yes | Must equal the array index (0-based, contiguous). |
| `content_hash` | string | yes | 64-char lowercase hex SHA-256. If `content` is present, validation recomputes and requires a match. |
| `token_count` | integer | no | Recorder-known token count. If absent, a documented heuristic (chars/4) is used when `content` is present; otherwise tokens are unknown (0 + warning). |
| `byte_count` | integer | yes | Byte count of the content. |
| `content` | string | no | **Optional.** Phase 0 fixtures omit it wherever possible (privacy stance). |
| `sensitivity` | string | no | Informational (e.g. `public`, `confidential`). |
| `dependencies` | array of string | no | Block IDs this block depends on (informational). |
| `lifetime` | integer | no | Observed age in turns. |
| `optional` | boolean | no | Explicitly removable/optional. |
| `required` | boolean | no | **Never removed by any policy, regardless of other flags or size.** |
| `stale` | boolean | no | Explicitly stale (e.g. superseded tool output). |
| `metadata` | object | no | Free-form JSON. |

Unknown JSON fields anywhere in a trace are ignored (forward compatibility).

## ProviderUsage

| Field | Type | Notes |
| --- | --- | --- |
| `input_tokens` | integer | Total input tokens, if reported. |
| `cache_read_tokens` | integer | Tokens served from cache, if reported. |
| `cache_write_tokens` | integer | Tokens written to cache, if reported. |
| `output_tokens` | integer | Output tokens, if reported. |
| `provider_raw` | object | Provider-specific raw usage, preserved verbatim. |

Per source-of-truth principle 7, reported usage **outranks** Prefixity's
theoretical estimates.

## CostProfile (provider profiles)

| Field | Type | Notes |
| --- | --- | --- |
| `name` | string | Profile name. |
| `version` | integer | Profile format version (currently 1). |
| `synthetic` | boolean | Must be `true` for every profile in this repository. |
| `currency` | string | e.g. `USD`. |
| `input_price_per_1m` | number | Price per 1M input tokens. |
| `cache_read_price_per_1m` | number | Price per 1M cache-read tokens. |
| `cache_write_price_per_1m` | number | Price per 1M cache-write tokens (0 if not applicable). |
| `output_price_per_1m` | number | Price per 1M output tokens (0 if out of scope). |
| `notes` | string | Must state the profile is SYNTHETIC unless audited. |

## Validation invariants (enforced)

- `format_version == 1`.
- `request_id`, `provider`, `model` non-empty.
- At least one block; at most `MAX_BLOCKS` (100,000).
- Positions contiguous `0..n`.
- Block IDs unique and bounded.
- `content_hash` well-formed; verified against `content` when present.
- Usage consistency checked; inconsistencies are warnings, not errors.
- Size limits from `prefixity-core::limits` applied.

## Example

```json
{
  "format_version": 1,
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
    "input_tokens": 1350,
    "cache_read_tokens": 1200,
    "cache_write_tokens": 0,
    "output_tokens": 100
  }
}
```
