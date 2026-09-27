# Phase 1C Claim-2 advancing-output protocol correction

Supervisor offline status: `WORKLOAD_PROTOCOL_DOMAIN_CORRECTION_ACCEPTED`.
The exact publication candidate still requires successful CI before promotion.
This offline workload-protocol correction follows the promoted Part-A domain
review baseline `1fa391bc333fa6c698cf67e4b97fe48c77124626`. It is not a
Prefixity planner or mechanism change. No model, tokenizer, or server was
contacted, and no inference occurred.

## Preregistered advancing-response rule

Only request slots 1 and 2 advance the trajectory. For each slot the fixture's
frozen expected action identity determines the one accepted raw response:

```text
UTF-8 bytes of {"action_id":<serde_json string for the expected action ID>}
```

The JSON object is compact, has exactly one field named `action_id`, and uses
the generated string value and field order. It has no byte-order mark, leading
or trailing whitespace, newline, extra field, alternate escaping, positional
array form, or Markdown fence. The implementation compares the observed raw
UTF-8 bytes with this exact representation before parsing or accepting an
advancing transition. It never canonicalizes or repairs a response.

The raw response is recorded before the comparison. Exact canonical bytes
continue unchanged into that arm's next request and produce the already-pinned
deterministic receipt. Any other bytes produce an observed structural `FAIL`,
remain unchanged in the slot record, produce no receipt for the failed step,
and leave every later request `NOT_EXECUTED_AFTER_FAILURE`. Existing semantic,
state, and receipt checks still run after exact-byte equality.

The rule applies equally to `BASELINE`, `NO_OP`, and `INTERVENTION`. Each arm
constructs and must emit its own response; arm outputs are never copied or
substituted. The final request-3 output schema and evaluator remain unchanged.
The accepted `max_tokens: 1024` envelope remains unchanged, and there is no
byte cap. The planning-byte estimate is not an output limit.

## Finite-domain ledger

The reproducible [version-1 finite-domain ledger](../../fixtures/claim2/advancing-output-domain-v1.json)
contains all 12 case/slot records, canonical raw response bytes, SHA-256,
receipt identity/path/hash, resulting state, arm scope, and raw-language
cardinality. The test suite pins the expected action mapping and receipt
mapping, checks every response hash, and verifies the ledger byte-for-byte.

| Case | Slot | Canonical raw response | Canonical response SHA-256 |
| --- | ---: | --- | --- |
| CP01 | 1 | `{"action_id":"inspect_retry_implementation"}` | `a162e3c32432fac88ed667c03a05875411e8412d0666522eb0dcf0b8d87409ee` |
| CP01 | 2 | `{"action_id":"run_retry_regression"}` | `4a3f0a7b0b635b644cfe1db2592cd69aa7b1a41f7b601aa09c8a0b11f08549f6` |
| CP02 | 1 | `{"action_id":"build_symbol_inventory"}` | `0885393d691f49d7ffdac93a029cd069b53fece50f200362dba696f29ab230fe` |
| CP02 | 2 | `{"action_id":"inspect_resolution_chain"}` | `b342853edb38641b8c5436d6d3647d2a2a8e766304bba1fd549ea9710dce5f2c` |
| CP03 | 1 | `{"action_id":"collect_dependency_inventory"}` | `d8e934c761601d55b6ef45d85968c8d7315bd003cda1b25190f38351d06dfc5b` |
| CP03 | 2 | `{"action_id":"verify_snapshot_and_constraints"}` | `d5bafc1765e8ed215fa9bda242f2cebcf00519ac788695e8df0672da88ed2c48` |
| CP04 | 1 | `{"action_id":"run_candidate_guardrail_probe"}` | `1c59f30f97067c353a215a7bb9c6b39ea0f297055f03382b121bee76000f62c5` |
| CP04 | 2 | `{"action_id":"verify_restored_state"}` | `0606c4dfc1a237c33ea72f9a5b6876cda7dd67c70a84fb8888fc2a3574fb5da7` |
| CP05 | 1 | `{"action_id":"inspect_initial_catalog"}` | `2d825a5710f0926419b09a77d1c89deb032b87911a9dbae81369fd2ca01ad6c0` |
| CP05 | 2 | `{"action_id":"rebuild_route_index_and_export"}` | `b4e1375372aca21a660281104c7602ecbc03ee05f35fcf9dd80cffc2fa1b42d1` |
| CP06 | 1 | `{"action_id":"inspect_policy_index"}` | `a316002641d343e0c5c123b1ad621425f8ebb06a3f9ea23731a1ec15445083dc` |
| CP06 | 2 | `{"action_id":"verify_audit_receipt"}` | `b26a09e02d258006cff404d74b115cd2a91ed869aa6a1c43a3e4eaba2dac4762` |

For each row the raw advancing-response language has cardinality 1. This is a
source-level byte-equality proof: precisely one `String` can equal the
fixture-derived canonical bytes at that case and slot. The permitted path
then produces the one frozen receipt; any other response follows the existing
terminal failure state machine. Thus each case and each arm has one advancing
raw-history sequence before request 3.

## Versioned materialization successor

The old [version-1 materialization report](../../fixtures/claim2/materialization-report.json)
is preserved byte-for-byte. Its SHA-256 is
`3d9571ddd4e17096b55971040531aac6197b6b9b7bc3d01afefd8992c28f6c65`.
The [version-2 successor report](../../fixtures/claim2/materialization-report-v2.json)
pins that predecessor hash, the protocol and normalized-LF source hashes, and
the finite-domain ledger hash.

The successor generator reproduced all 54 historical static projection slots
byte-for-byte, then independently replayed 18 case-arm trajectories and 36
canonical transitions. It rendered 54 requests across slots 1–3 and checked
that each arm carried its own exact response into the next request and emitted
the pinned receipt identity and state. These are deterministic offline
workload-protocol replays, not model observations.

## Boundaries and interpretation

Unchanged boundaries include the six fixture manifests and separate keys,
case tasks and actions, receipt and attachment content, dependencies, the
Phase 1B.9 policy and selection, NO_OP projection, final request-3 schema and
evaluator, the 8192-token context setting, the 1024-token output envelope, and
the 6000-token input preflight. This task adds no dependency and does not
change the planner or production mechanism.

The accepted raw-history domain is now finite for the twelve intermediate
transition points. This does not establish token counts, context fit, token
savings, materiality qualification, model outcome, or runtime benefit. Those
remain unevaluated. The next authorized task is only
`NON_INFERENCE_CONTEXT_TOKENIZATION_PASS`; no tokenization or inference was
started here.

## Reproduction

From the repository root, regenerate the ledger first, then the successor:

```powershell
cargo run -p prefixity-controlled-benchmark --example claim2_protocol_successor --offline --locked -- --write-ledger fixtures/claim2/advancing-output-domain-v1.json
cargo run -p prefixity-controlled-benchmark --example claim2_protocol_successor --offline --locked -- --write-successor fixtures/claim2/materialization-report-v2.json
```

The fixture attributes disable line-ending conversion for all files in this
directory. The source hashes in the successor normalize CRLF to LF before
hashing, so regenerated provenance is stable across Windows and Linux.

## Validation and supervisor acceptance

Part A was promoted at `1fa391bc333fa6c698cf67e4b97fe48c77124626` after all
four jobs passed in [candidate CI](https://github.com/hourwise/Prefixity/actions/runs/36346330551).
Its [main CI](https://github.com/hourwise/Prefixity/actions/runs/36346596731)
also passed. The historical review still records `TOKENIZATION_PASS_INCONCLUSIVE`
with reason `TOKENIZATION_BOUND_NOT_FINITE`; the correction does not rewrite
that result.

Local validation passed: full locked/offline workspace tests; protocol tests
3/3; research-state consistency tests 12/12; formatting; workspace Clippy
with all targets/features and warnings denied; and Rust 1.86 locked/offline
workspace check. Existing suites cover six-case validation, dependency closure,
hidden-key isolation, arm rendering, NO_OP projection, unchanged Phase 1B.9
report parity and consumed-gate rejection. The new protocol suite checks 18
independent case-arm trajectories, 36 canonical advances and 540 rejected
variants. The final-output whitespace regression retains the original schema.

Supervisor review checked the complete source/fixture diff, all twelve
canonical hashes against the frozen keys, receipt hashes and states, source
provenance, local links and whitespace. Only two existing Rust source files,
three new Rust example/test files, the two successor evidence files, this
record, INDEX, ACTIVE and the cohort README are in the publication scope.

Evidence SHA-256 values:

- Finite-domain ledger: `b4689157548864c819945dd8260478dc0ba6a5811dd96626768f3f5b83a09fc8`.
- Materialization successor: `30ecb777b52d201765ca7cba83ed9692519e63bb8e318ff515e93280e27b3dab`.

The finite-domain proof and offline correction are accepted. Publication must
promote only the exact commit whose four CI jobs pass; the supervisor closeout
will report that SHA and CI outcome. Readiness, token-count and inference
contacts remain zero. The next authorized task is
`NON_INFERENCE_CONTEXT_TOKENIZATION_PASS`, which has not begun.
