# Phase 1C Stage 1 failed-smoke offline forensic review

Status: `FAILED` execution accepted as valid bounded evidence. This review is
offline-only. It does not authorize another schema smoke, a retry, a runtime
change, or scored capability execution.

## Scope and frozen boundary

The immutable evidence is under
`experiments/runs/phase1c-stage1-schema-smoke-01/`. The original response was
not rewritten, normalized, repaired, or replaced. This review performed only
filesystem reads, JSON parsing, hashing, repository inspection, and offline
runtime-help inspection. It made zero network calls and zero inference
requests.

The execution branch was at `81e49430ad59e8e58be9a53384ea4a2d14691c3f`,
`origin/main` remained `36f960579e55c7ad48e54e4cb2670cc55cd1ef3e`, and no Git
lock was present. Exactly one Phase 1C Stage 1 evidence directory exists, and
the scored capability cohort remains at zero requests.

## Evidence integrity

The preserved files and revalidated SHA-256 values are:

| file | bytes | SHA-256 |
| --- | ---: | --- |
| `preflight.json` | 973 | `8f32b9eb344fd16c8ab966704428dd141f2d7f1e5c9217435ef6afcfdebbc541` |
| `readiness.json` | 144 | `ae8318ed9afbf746aabff297ef9cbd71fc3beb991fc3e097772c2443a53958a8` |
| `request.json` | 491 | `ec97953bbb9e9115d9ae63885092fbc0d606daf192dcaafc47c069d45900592d` |
| `response-body.bin` | 1909 | `223006f5bed5fec81dae02097e3e723d8c8d3a5997c8ebb116841e95914eb048` |
| `stage1-result.json` | 2304 | `c0a110ca9ccfdedfa9be9e5fa261fb062fd2f38e1a820c1a56507edbd05bbf25` |

The runtime-contract identity is unchanged at
`6d32375c24d51b98d82a8873d6e0f03f1d8c9b59799b843cd3a3d69c05b097b3`; it
matches the preflight contract fingerprint and the final result identity. The
fixture identity remains
`98b6141de86b56c0e8f2f75633e67c88c4c26f344a020a347c004b704cd68584`.

The run accounting is one listener check, one transport attempt, one
inference request, and zero automatic retries, fallbacks, or replicates. The
listener check recorded `network_calls=1` and `inference_requests=0`; the
single HTTP request returned successfully and produced the sole Phase 1C live
inference request.

## Raw response structure

`response-body.bin` is valid UTF-8 JSON. Its top-level fields are:

`choices`, `created`, `model`, `system_fingerprint`, `object`, `usage`, `id`,
and `timings`.

The first choice contains:

`finish_reason`, `index`, and `message`.

The first message contains exactly:

`role`, `content`, and `reasoning_content`.

No `message.reasoning`, `message.tool_calls`, `choice.text`, top-level
reasoning field, token-probability/logprob field, or other generated-output
field was present. The parser/diagnostic metadata present at the top level was
the normal `usage` and llama.cpp `timings` object, plus response identity
fields.

Generated textual fields inside the first message:

| field | UTF-8 bytes | Unicode characters | empty | exact field SHA-256 | bounded diagnostic |
| --- | ---: | ---: | --- | --- | --- |
| `content` | 0 | 0 | yes | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | empty string |
| `reasoning_content` | 1226 | 612 | no | `19ffca109df7e5c9784abccffb6e9ad5e79335aa0e35e59ee00610c9691ce08a` | non-empty textual prose; 313 non-ASCII characters and 6 newline characters; not a JSON object; raw content intentionally not copied here |

The raw response therefore does contain generated model output, but it is in
`message.reasoning_content`, not in the contract-selected
`message.content`. The raw response is the source of this conclusion; the
reasoning text is not treated as the requested schema payload.

## Token-accounting reconciliation

The response reports:

- `prompt_tokens=79`;
- `completion_tokens=256`;
- `total_tokens=335`;
- `prompt_tokens_details.cached_tokens=0`;
- `timings.prompt_n=79`;
- `timings.predicted_n=256`;
- `finish_reason=length`.

The arithmetic is internally consistent: `79 + 256 = 335`. Native predicted
count and compatibility completion count also agree at 256. The 612 Unicode
characters in `reasoning_content` are not a token count, but they establish
that the complete generated text exposed by this response was separate
reasoning/prose while ordinary assistant content remained empty.

The response directly establishes that the completion limit was reached while
separate reasoning content was being emitted, before an ordinary assistant
JSON payload appeared. It does not expose a distinct internal reasoning-budget
setting beyond the registered `max_tokens=256`; the precise internal budget
mechanism is therefore not separately established.

### Classification

Primary classification: `THINKING_BUDGET_EXHAUSTION` — established at the
response level, with high confidence. The immediate observed cause is
completion-limit exhaustion during separate reasoning output.

Secondary contract consequence: `RUNTIME_RESPONSE_SHAPE_INCOMPATIBILITY` —
established only in the narrow sense that the runtime returned a
`reasoning_content` channel not represented by the content-only Stage 1 smoke
contract. This is not evidence that the llama.cpp HTTP protocol itself is
invalid.

The alternatives are not supported as primary causes:

- `ADAPTER_EXTRACTION_MISMATCH`: not established; the adapter extracted the
  exact field required by the accepted contract, and that field was present as
  an empty string.
- `MODEL_FORMAT_PARSER_INCOMPATIBILITY`: not established; the runtime parsed
  the request, returned valid JSON, surfaced generated reasoning text, and
  reported coherent token/timing metadata. A model/template contribution is
  possible, but parser failure is not shown.
- `OTHER`: not required by the preserved evidence.

Confidence is high for the immediate response-level explanation and medium
for the exact model-template mechanism. The latter remains unresolved because
the raw response does not identify the selected model's exact chat-template
decision or internal reasoning-budget configuration.

## Request review

The preserved request has exactly these seven top-level keys and no others:

`model`, `messages`, `temperature`, `top_p`, `max_tokens`, `stream`, and
`seed`.

Its messages are exactly the registered two-message fixture:

- system: `You are performing a schema and transport validation. Return only
  one JSON object matching the requested fields. Do not include Markdown or
  explanatory prose.`
- user: `Return a JSON object containing exactly these semantic values:
  schema_version=phase1c-stage1-smoke-v1; status=ok;
  marker=PREFIXITY_PHASE1C_STAGE1.`

The transmitted generation fields are `temperature=0`, `top_p=1`,
`max_tokens=256`, `stream=false`, and `seed=1`. The model is exactly
`ggml-org/Qwen3.5-0.8B-GGUF:Q4_0`. Offline comparison against the tracked
contract and fixture found exact key, message, model, and generation matches;
there were no additional transmitted fields. Request compliance: `yes`.

No grammar or JSON-schema constrained decoding was used.

## Local reasoning-mode evidence

The installed llama.cpp executable reports build
`b10217-ddd4ec142`. Its offline `serve --help` output documents:

- `--reasoning [on|off|auto]`, with default `auto` and detection from the
  template;
- `--reasoning-format`, including a `deepseek` mode that places thoughts in
  `message.reasoning_content`;
- `--reasoning-budget`; and
- chat-template and chat-parsing controls.

The canonical Stage 1 launch supplied no explicit reasoning flag, so this
runtime family was left at its default reasoning decision. The raw response's
non-empty `message.reasoning_content` is direct evidence that a separate
reasoning channel was active for this request.

No exact Qwen3.5 GGUF, tokenizer, or chat-template metadata was available in
the repository or in the inspected common local cache locations. Therefore
the exact model-template rule that caused the default decision is not
established offline. No internet, llama.cpp startup, or localhost contact was
used for this review.

## Adapter review

The isolated adapter in
`crates/prefixity-controlled-benchmark/src/phase1c_stage1_local_qwen.rs`:

- persisted the exact bounded HTTP response body before semantic validation;
- counted one transport/inference attempt and never retried;
- extracted only `/choices/0/message/content`, as required by the accepted
  Stage 1 response contract;
- retained other response fields in the raw body rather than using them as a
  silent fallback;
- performed no prose stripping, fence removal, field substitution, or
  reasoning-to-content repair; and
- returned `FAILED` because the exact semantic smoke payload was not present.

Adapter compliance: `yes`. It behaved correctly under the registered
content-only contract. The contract was not compatible with the observed
reasoning-bearing response for this smoke. The transport protocol itself did
complete successfully with HTTP 200.

## Minimum viable remediation options

These are design options only. None was implemented or applied to the frozen
run.

| rank | option | scope and change | later scored cohort | contamination / identity |
| ---: | --- | --- | --- | --- |
| 1 | Explicitly disable reasoning for a schema-only smoke | Start a fresh, explicitly registered runtime with `--reasoning off` or the equivalent supported setting; keep extraction at `message.content`. This changes the smoke runtime contract. | Must not be propagated automatically. The 216-request design needs its own reasoning decision and contract. | Low if isolated to a new smoke identity and fresh runtime. The failed smoke remains independently preserved. |
| 2 | Preserve reasoning and increase the output allowance | Register a larger `max_tokens` bound sufficient for reasoning plus the final JSON, then require `message.content` to contain the exact payload. This changes the smoke request/runtime contract. | No effect unless separately selected and registered for scored execution. | Low to moderate: generation cost and termination behavior change. Requires a fresh smoke identity; original evidence remains preserved. |
| 3 | Register a reasoning-aware response contract | Persist and validate `reasoning_content` as a separate diagnostic field while still requiring the final payload in `message.content`; do not treat reasoning prose as the answer. | Separate scored-design decision; no silent propagation. | Low if the reasoning field is diagnostic-only. Requires a fresh schema-smoke identity because the response contract changes. |
| 4 | Adjust chat-template/parser/reasoning-format configuration | Explicitly register a model/template/parser setting only after exact local metadata is available; this changes the runtime contract and may change output semantics. | Must be independently evaluated for the scored cohort. | Moderate to high until the exact template/parser behavior is documented. Requires a fresh identity; failed evidence stays intact. |

Changing extraction alone to read `reasoning_content` is not a valid fix for
this fixture: the preserved field is explanatory prose, not the required JSON
object. It would be a repair or semantic substitution and is therefore not
recommended.

## Schema-smoke versus scored capability experiment

For a schema/transport smoke, disabling reasoning can be a valid operational
choice when the purpose is only deterministic structured-response plumbing.
That choice changes the smoke runtime contract and should receive a fresh
fixture/identity rather than rewriting this failed run.

The later 216-request capability programme is a separate experimental-design
decision. It must explicitly register whether reasoning is retained, disabled,
budgeted, or represented through a reasoning-aware response schema. No
schema-smoke workaround may be silently propagated to BASELINE, NO_OP, or
INTERVENTION execution.

## Repository result and boundaries

This review changes no production code, runtime contract, adapter behavior,
fixture, or raw evidence. A documentation-only review record and a pointer in
`docs/tasks/ACTIVE.md` are the only intended tracked changes.

Final boundary confirmation:

- new inference requests during this review: `0`;
- total Phase 1C Stage 1 inference requests: `1`;
- Stage 1 retries: `0`;
- scored capability tasks: `0`;
- Prefixity behavioral changes: `0`;
- original failed-smoke evidence: unchanged.

Stop here pending a separate remediation authorization.
