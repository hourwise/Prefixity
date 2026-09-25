# Phase 1C reasoning-budget calibration — Attempt 010 preparation

Status: `ATTEMPT_010_PREPARED`, `ATTEMPT_010_NOT_EXECUTED`

This record prepares Attempt 010 at candidate budget 512. It does not
authorize or perform a live run. No model server was started, port 8080 was
not contacted, and no readiness, HTTP, or inference request was made.
**Budget 512 remains experimentally untested.**

## A. Baseline

```text
accepted baseline (main)   a0019bcd96ca8759801a879619f119032067a3f9
code/build source commit   cd7145319555d6f00c979e6e333fc66c921fa335
identity                   docs/phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_010_IDENTITY_V1.json
identity canonical SHA-256 9292e9ecdd2e89f695dfb34bc782ade41b70412c427807c6c3a5653b50ec16f7
```

The baseline contains the accepted candidate-order remediation
(`PHASE_1C_CANDIDATE_ORDER_GATE_REMEDIATION_ACCEPTED`).

## B. Eligibility

```text
LAST_ADMISSIBLE_ATTEMPT = 008   (budget 1024, FAIL, integrity ACCEPTED)
ATTEMPT_009                     executed once, integrity rejected,
                                calibration inadmissible, permanently
                                consumed, inference_requests = 0
NEXT_FRESH_ATTEMPT_ID = 010
NEXT_CANDIDATE_BUDGET = 512
ATTEMPT_010_ELIGIBLE
ATTEMPT_010_VIRGIN
```

Before preparation no Attempt-010 identity, evidence root, request ledger,
result, retry record, execution lock, or stale handoff existed.

## C. Predecessor provenance

The candidate order uses the accepted canonical loader
`load_authoritative_attempt_008_transition` over the tracked transition
`fixtures/phase1c/attempt-009-budget-provenance.json`:

```text
SOURCE_ATTEMPT_008
SOURCE_BUDGET_1024
SOURCE_STATE_FAIL
SOURCE_INTEGRITY_ACCEPTED
SOURCE_CALIBRATION_ADMISSIBLE
NEXT_BUDGET_512
CANDIDATE_BUDGET_512_ORDER_VALID
ATTEMPT_007_EXCLUDED_FROM_SELECTION
ATTEMPT_009_EXCLUDED_FROM_SELECTION
```

The Attempt-008 evidence-manifest SHA-256 is the complete 64-character
`f20c4ce0149070e3ca1bc167f4400d71b88fe0bd7adac41851169ba8540e4779`, bound
in the identity lineage and compared with the loader output. Attempt 009 is
bound by its tracked identity (`0929aae1…f812`) and execution record
(normalized SHA-256 `2eb51b44…cfaaa`); the selection path never consults it.
No ignored raw prior-run evidence is required.

## D. Frozen contract

| Parameter | Value |
| --- | --- |
| experiment | `phase1c-reasoning-budget-calibration` |
| attempt / budget | 010 / 512 |
| model | `ggml-org/Qwen3.5-0.8B-GGUF:Q4_0` (Q4_0) |
| cases | `rbcal-001`, `rbcal-002`, `rbcal-003` (request hashes unchanged) |
| manifest | `4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b` |
| request ceiling | 3 |
| reasoning | on, server-side `--reasoning-budget 512` |
| context / slots | 8192 / 1 |
| temperature / top_p / seed | 0 / 1 / 1 |
| output token limit / stream | 2048 / false |
| retries / fallback / replicates / warmup | 0 / 0 / 0 / none |
| fresh runtime / workflow exclusivity | required / required |

Prompts and cases were not altered.

## E. Frozen executable identities

Built with `cargo build -p prefixity-controlled-benchmark --bins --locked
--offline` (cargo/rustc 1.97.1, `x86_64-pc-windows-msvc`, dev profile,
`Cargo.lock` `79f4fb5e…1cf8`) from `cd71453`, after fmt, clippy, the full
workspace test suite, and the build had passed. Frozen with the
non-overwriting `attempt-010-freeze` into `target/phase1c-attempt-010-frozen/`.

| Object | SHA-256 | Size | Windows file ID |
| --- | --- | --- | --- |
| supervisor | `3e43fddd7fdfe5e1b2bb510843108ed9fbbdd598ce5f4c265e5d7fb4943c288e` | 1259008 | `volume=ba2f80f4;index=00080000003b6e6d` |
| child | `e65a626eaada5a243ff05b226ca7adbd4976f8a3e37aea7bfeb4dcec79b361c9` | 9010176 | `volume=ba2f80f4;index=0008000000454e40` |

Frozen content equals the build output; each frozen copy has its own file ID.
The mutable `target/debug` objects are build inputs only.

## F. Pre-live prerequisite traversal

`run-attempt-010` begins with `attempt_010_live_prerequisites()`. The offline
command `attempt-010-live-prerequisites` calls the same function, and was run
through the recorded traversal command using the real frozen supervisor and
frozen child:

```text
.\target\phase1c-attempt-010-frozen\prefixity-phase1c-live-supervisor.exe --attempt-identity docs/phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_010_IDENTITY_V1.json --evidence target/phase1c-attempt-010-prerequisite-traversal/supervisor.json -- .\target\phase1c-attempt-010-frozen\prefixity-phase1c-reasoning-budget-calibration.exe attempt-010-live-prerequisites
```

Result (supervisor PID 10836, child PID 6140, one launch, no retry, exit 0):

```text
ATTEMPT_ID_VALID                 ATTEMPT_IDENTITY_VALID
FROZEN_SUPERVISOR_VALID          FROZEN_CHILD_VALID
WORKFLOW_CERTIFICATION_VALID     RUNTIME_DEPENDENCIES_COMPLETE
PREDECESSOR_TRANSITION_PRESENT   PREDECESSOR_TRANSITION_VALID
SOURCE_ATTEMPT_008               NEXT_BUDGET_512
CANDIDATE_ORDER_VALID            CASE_SET_VALID
REQUEST_CEILING_VALID            RETRY_POLICY_VALID
FALLBACK_POLICY_VALID            ATTEMPT_010_VIRGIN
READY_FOR_MODEL_READINESS_BOUNDARY
```

The traversal includes the extracted `calibration_prestart_checks()` that the
live calibration path runs immediately before its single listener check
(budget registration, fresh-runtime confirmation, manifest and sidecar,
candidate order, evidence root absent). It stopped before post-start runtime
ownership inspection, TCP readiness, HTTP, and inference. The supervisor
record is under `target/`, not the Attempt-010 evidence root, which remains
absent.

Post-start runtime ownership inspection requires the operator-started model
server and is therefore the first step that cannot be exercised offline.

## G. Clean-checkout proof

`validate_attempt_010_repository_contract()` reads only tracked evidence and
is called unchanged by the live prerequisites. The integration test
`crates/prefixity-controlled-benchmark/tests/phase1c_attempt_010_contract.rs`
runs it in CI fresh clones, which contain no ignored `experiments/runs/`
evidence, and rejects contract, predecessor, and executable-identity
mutations. Frozen executables and the v2 certification objects are explicitly
bound by hash in the identity and are validated by the Windows traversal.

## H. Preparation/runtime parity

Preparation, the traversal, and the live child share the same identity
validator, sidecar check, frozen-executable validator, handoff validator,
repository contract, candidate-order loader, and prestart checks. The
traversal additionally asserts that the runtime candidate order equals the
repository-contract candidate order. `PREPARATION_RUNTIME_PARITY_VALID`.

## I. Offline gates

| Gate | Result |
| --- | --- |
| frozen child `attempt-010-fingerprint` | PREPARED; canonical SHA equals independently generated sidecar |
| `attempt-010-repository-contract` | ATTEMPT_010_REPOSITORY_CONTRACT_VALID |
| `attempt-010-preflight` | ATTEMPT_010_PREPARED; native `EXCLUSIVE_PRESTART` |
| `attempt-010-dry-run` | DRY_RUN, three combinations, zero network |
| `attempt-010-validate-preparation` | ATTEMPT_010_PREPARATION_ACCEPTED |
| frozen supervisor → frozen child traversal | READY_FOR_MODEL_READINESS_BOUNDARY |
| frozen supervisor + mutable child | FROZEN_EXECUTABLE_IDENTITY_MISMATCH before spawn |
| mutable supervisor + frozen child | FROZEN_EXECUTABLE_IDENTITY_MISMATCH before spawn |
| same-content child copy (different file ID) | FROZEN_EXECUTABLE_IDENTITY_MISMATCH before spawn |
| same-content supervisor copy | FROZEN_EXECUTABLE_IDENTITY_MISMATCH before spawn |

No rejected substitution wrote supervisor evidence.

## J. Future commands (not executed)

Server command — `NOT_EXECUTED`:

```text
C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe serve -hf ggml-org/Qwen3.5-0.8B-GGUF:Q4_0 -c 8192 -np 1 --metrics --reasoning on --reasoning-budget 512 --host 127.0.0.1 --port 8080
```

Frozen supervisor command — `NOT_EXECUTED`:

```text
.\target\phase1c-attempt-010-frozen\prefixity-phase1c-live-supervisor.exe --attempt-identity docs/phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_010_IDENTITY_V1.json --evidence experiments/runs/phase1c-reasoning-budget-calibration/budget-512-attempt-010/supervisor.json -- .\target\phase1c-attempt-010-frozen\prefixity-phase1c-reasoning-budget-calibration.exe run-attempt-010
```

## K. Runtime accounting

```text
model_server_startups=0
port_8080_contacts=0
tcp_readiness_contacts=0
http_model_requests=0
inference_requests=0
attempt_009_executions_added=0
attempt_010_executions=0
```

## L. Known deferred issues

```text
KNOWN_DEFERRED_ISSUE:
budget-256 candidate-order path still uses the older generic predecessor lookup.
```

This does not block Attempt 010 at 512. Budget 256 is not prepared or
authorized. If a future admissible 512 result selects 256, the 256
candidate-order path must be remediated and validated before any 256 attempt
is prepared.

A second historical defect was observed and not changed: commit `5918141`
placed the Attempt-009 supervisor-command expectation inside
`validate_attempt_008_identity`, so that validator no longer accepts the
Attempt-008 identity. Attempt 008 is consumed and no Attempt-010 path calls
it.
