# Phase 1B.4 Evidence-Adapter Recharacterization

Status: `PASS WITH RECORDED LIMITATIONS`

Phase 1B.4 implemented the smallest evidence-adapter revision justified by
the pinned Phase 1B.3 raw-schema verification. The adapter preserves verified
source facts and bounded structural identity while leaving intervention-safety
evidence absent where the raw corpus does not establish it.

## Frozen source and integrity

The source remains `NJU-LINK/CodeTraceBench`, revision
`aa213b84ffb6690fc37ca15766d6ca174ec36d4d`, split `verified`, with the
existing accepted selection of 24 trajectories. Re-import produced 719
request traces and 1,498 source events. Two independent deterministic imports
produced the same 724-file derivative set and identical SHA-256 hashes. The
repository contains no raw trajectory archive or trajectory-content file; the
stored derivative remains hash-only.

## Adapter and model changes

Trace format v2 remains unchanged. An additive evidence schema version `1`
now describes optional typed evidence fields:

- source-explicit numeric message timestamps on blocks and source events;
- bounded provider response metadata (`id`, `model`, `created`, `object`,
  selected choice/finish metadata, and selected response-field presence
  states);
- provider/raw usage provenance and field-level evidence provenance;
- bounded source locators containing trajectory identity, source-file SHA-256,
  source-event identity/index, and upstream field path.

Provenance origins are `source_explicit`, `derived_structural`, and
`unknown`. Role-to-source/zone projection, generated message IDs, message
paths, and unique locator-to-message-span joins are explicitly marked as
derived. Provider response IDs are response identity only, not dependencies or
tool relationships.

The importer no longer classifies a user message as a tool result based on
content markers. Classification is role-only. No optional, required, stale,
dependency, tool-call/result, invalidation, supersession, or removability
evidence is synthesized. Timestamp age is not interpreted as staleness.

## Coverage

- Provider response metadata: 719/719 traces have explicit response IDs,
  models, `created`, and finish reasons.
- Provider usage: 719/719 traces retain the raw response usage object. The
  recorded schema distribution is Anthropic custom schema 268, DeepSeek custom
  schema 118, and OpenAI Chat Completions schema 333.
- Exact existing usage normalization applies only to the OpenAI schema fields
  already supported by Prefixity: `prompt_tokens`,
  `prompt_tokens_details.cached_tokens`, `completion_tokens`, and
  `total_tokens`. Anthropic and DeepSeek provider-specific fields remain raw
  because this adapter does not assert cross-provider semantics. No pricing or
  cache-savings claim is made.
- Timestamps: 1,498/1,498 source events retain an explicit numeric timestamp;
  the adapter does not derive stale, invalidation, supersession, or
  removability state from timestamp age.
- Evaluation locators: 63 explicit locator references occur across 60 labeled
  steps. Exactly 32 steps join to a unique source event by explicit path and
  bounded line span; 28 remain unresolved. No positional fallback is used.
  Labels are sidecar-only and loaded after both planner passes.

## Frozen planner result

The existing frozen Phase 1B planner checkpoint
`3436e16afcdf359a33a691c15202900d796b25bc` was run offline over all 719
traces, without provider/model calls, provider profiles, or evaluation labels.
The decision distribution is:

| Recommendation | Count |
| --- | ---: |
| `DO_NOTHING` | 719 |
| `KEEP` | 0 |
| `DEFER` | 0 |
| `PRUNE` | 0 |
| `RELOCATE_CANDIDATE` | 0 |
| `COMPRESS_CANDIDATE` | 0 |

Both planner passes produced 719 plans, zero validation failures, and the
same aggregate hash
`5157f5a4a8b59d58d8898bf3df3fc4ad9bea60f08ccf9d920b87f41734e806fb`.
The unchanged `DO_NOTHING` result is valid because the newly preserved
provider/timestamp evidence does not establish removal safety, dependency
closure, staleness, or tool-call/result relationships.

The hard safety audit remained clean: zero source-trace mutations, zero
destructive recommendations against current requests/required/protocol blocks,
zero missing-or-cyclic dependency evidence violations, zero unsafe relocation
recommendations, zero contradictory destructive recommendations, and zero
non-hypothetical recommendations.

## Assessment and limitations

The revision materially improves auditability of provider response identity,
provider-specific usage, source-explicit timing, field-level provenance, and
bounded evaluation-source mapping. It does not provide causal evaluation
coverage or positive intervention evidence. The remaining 28 labeled steps are
correctly unresolved, and no quality, savings, latency, replay, or task-success
claim is authorized. The accepted corpus has now exhausted the useful evidence
available for this narrow Phase 1B adapter revision; another planner
characterization on the same slice is not justified without new evidence.

Recommended next task: separately review and, if authorized, source a corpus
or evaluation artifact with explicit action/result/dependency/removability
identity and task-quality joins. Do not begin Phase 1C from this result.
