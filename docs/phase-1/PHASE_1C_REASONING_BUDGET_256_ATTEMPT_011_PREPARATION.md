# Phase 1C reasoning-budget calibration — Attempt 011 preparation

Status: `ATTEMPT_011_PREPARED`, `ATTEMPT_011_NOT_EXECUTED`

This record prepares Attempt 011 at candidate budget 256. It does not
authorize or perform a live run. No model server was started, port 8080 was
not contacted, and no readiness, HTTP, or inference request was made.
**Budget 256 remains experimentally untested.**

## A. Baseline

```text
accepted baseline (main)   292dd1f0951f20fdcf6f49facbb446076d80a135
code/build source commit   4dd619e82a4d494f0ee8ca954440d46bb30f300e
identity                   docs/phase-1/PHASE_1C_REASONING_BUDGET_256_ATTEMPT_011_IDENTITY_V1.json
identity canonical SHA-256 60b442a1b09a4ff817b1ca40c103853f6972baeb3d13c91f551cb58ff5bd03ed
```

The baseline contains the accepted authoritative-transition generalization
(`PHASE_1C_AUTHORITATIVE_TRANSITION_GENERALIZATION_ACCEPTED`).

## B. Eligibility

```text
LAST_ADMISSIBLE_ATTEMPT = 010   (budget 512, FAIL, integrity ACCEPTED)
NEXT_FRESH_ATTEMPT_ID = 011
NEXT_CANDIDATE_BUDGET = 256
ATTEMPT_011_ELIGIBLE
ATTEMPT_011_VIRGIN
```

Before preparation no Attempt-011 identity, evidence root, request ledger,
result, retry record, execution lock, or stale handoff existed; no llama
process was running and port 8080 had no owner.

## C. Predecessor provenance

Candidate 256 resolves only through `resolve_authoritative_candidate_transition`
over `fixtures/phase1c/calibration-candidate-transitions.json` (normalized
SHA-256 `ceef4e5e…`, bound in the identity and re-checked by the contract):

```text
SOURCE_ATTEMPT_010
SOURCE_BUDGET_512
SOURCE_STATE_FAIL
SOURCE_INTEGRITY_ACCEPTED
SOURCE_CALIBRATION_ADMISSIBLE
NEXT_BUDGET_256
CANDIDATE_BUDGET_256_ORDER_VALID
ATTEMPT_007_EXCLUDED_FROM_SELECTION
ATTEMPT_009_EXCLUDED_FROM_SELECTION
ATTEMPT_008_NOT_SOURCE_FOR_256
```

The transition is bound to the Attempt-010 identity
(`9292e9ec…16f7`), evidence manifest
(`5673e55b381d1f5171c38bc9f1721b3105adf829ece4ec029cbcca4db150c4b9`), and
execution record (`c85dd5f3…0edc`). No ignored raw evidence is read.

The consumed Attempt-010 identity is not required to match current source;
Attempt 011 binds only to current source and its own frozen build outputs.

## D. Frozen contract

| Parameter | Value |
| --- | --- |
| experiment | `phase1c-reasoning-budget-calibration` |
| attempt / budget | 011 / 256 |
| model | `ggml-org/Qwen3.5-0.8B-GGUF:Q4_0` (Q4_0) |
| cases | `rbcal-001`, `rbcal-002`, `rbcal-003` (request hashes unchanged) |
| manifest | `4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b` |
| request ceiling | 3 |
| reasoning | on, server-side `--reasoning-budget 256` |
| context / slots | 8192 / 1 |
| temperature / top_p / seed | 0 / 1 / 1 |
| output token limit / stream | 2048 / false |
| retries / fallback / replicates / warmup | 0 / 0 / 0 / none |
| fresh runtime / workflow exclusivity | required / required |

Prompts, cases, scoring, output ceiling, and selection rules are unchanged.

## E. Frozen executable identities

Built with `cargo build -p prefixity-controlled-benchmark --bins --locked
--offline` (cargo/rustc 1.97.1, `x86_64-pc-windows-msvc`, dev profile,
`Cargo.lock` `79f4fb5e…1cf8`) from `4dd619e`, after fmt, clippy, the full
workspace tests (controlled-benchmark library 150 passed, 0 failed), and the
build had passed. Frozen with the non-overwriting `attempt-011-freeze` into
`target/phase1c-attempt-011-frozen/`.

| Object | SHA-256 | Size | Windows file ID |
| --- | --- | --- | --- |
| supervisor | `9dbf6dff7d914bce6f620fb2455bc64d304015ed6c2321d5ca7a7f935f72b8b1` | 1259008 | `volume=ba2f80f4;index=0006000000387bbc` |
| child | `10533899e3d9bc1e35b6ae83e518f499e66ee7d819a2b64f8626fae76e8e15d5` | 9281536 | `volume=ba2f80f4;index=0005000000387bbd` |

Frozen content equals the build output; each frozen copy has its own file ID.

## F. Pre-live prerequisite traversal

`run-attempt-011` begins with `attempt_011_live_prerequisites()`; the offline
`attempt-011-live-prerequisites` command calls the same function. It was run
through the real frozen supervisor and child:

```text
.\target\phase1c-attempt-011-frozen\prefixity-phase1c-live-supervisor.exe --attempt-identity docs/phase-1/PHASE_1C_REASONING_BUDGET_256_ATTEMPT_011_IDENTITY_V1.json --evidence target/phase1c-attempt-011-prerequisite-traversal/supervisor.json -- .\target\phase1c-attempt-011-frozen\prefixity-phase1c-reasoning-budget-calibration.exe attempt-011-live-prerequisites
```

Result (supervisor PID 19172, child PID 15244, one launch, no retry, exit 0):

```text
ATTEMPT_ID_VALID                 ATTEMPT_IDENTITY_VALID
FROZEN_SUPERVISOR_VALID          FROZEN_CHILD_VALID
WORKFLOW_CERTIFICATION_VALID     RUNTIME_DEPENDENCIES_COMPLETE
PREDECESSOR_TRANSITION_PRESENT   PREDECESSOR_TRANSITION_VALID
SOURCE_ATTEMPT_010               NEXT_BUDGET_256
CANDIDATE_ORDER_VALID            CASE_SET_VALID
REQUEST_CEILING_VALID            RETRY_POLICY_VALID
FALLBACK_POLICY_VALID            ATTEMPT_011_VIRGIN
READY_FOR_MODEL_READINESS_BOUNDARY
```

The traversal includes the shared `calibration_prestart_checks(256)` that the
live path runs immediately before its single listener check, and stopped
before post-start ownership inspection, TCP readiness, HTTP, and inference.
The supervisor record is under `target/`; the Attempt-011 evidence root
remains absent.

## G. Clean-checkout proof

`validate_attempt_011_repository_contract()` reads only tracked evidence and
is called unchanged by the live prerequisites. The integration test
`crates/prefixity-controlled-benchmark/tests/phase1c_attempt_011_contract.rs`
runs it in CI fresh clones with no ignored `experiments/runs/` evidence or
generic `candidate-result.json`, and rejects contract, predecessor, and
executable-identity mutations.

## H. Preparation/runtime parity

Preparation, the traversal, and the live child share the identity,
source-binding, sidecar, frozen-executable, handoff, repository-contract,
resolver, and prestart checks. The traversal asserted that the runtime
candidate order equals the contract candidate order, both resolving
`SOURCE_ATTEMPT_010` / `NEXT_BUDGET_256`. `PREPARATION_RUNTIME_PARITY_VALID`.

## I. Offline gates

| Gate | Result |
| --- | --- |
| frozen child `attempt-011-fingerprint` | PREPARED; canonical SHA equals the independently generated sidecar |
| `attempt-011-repository-contract` | ATTEMPT_011_REPOSITORY_CONTRACT_VALID |
| `attempt-011-preflight` | ATTEMPT_011_PREPARED; native `EXCLUSIVE_PRESTART` |
| `attempt-011-dry-run` | DRY_RUN, three combinations at budget 256, zero network |
| `attempt-011-validate-preparation` | ATTEMPT_011_PREPARATION_ACCEPTED |
| frozen supervisor → frozen child traversal | READY_FOR_MODEL_READINESS_BOUNDARY |
| frozen supervisor + mutable child | FROZEN_EXECUTABLE_IDENTITY_MISMATCH before spawn |
| mutable supervisor + frozen child | FROZEN_EXECUTABLE_IDENTITY_MISMATCH before spawn |
| same-content child copy (different file ID) | FROZEN_EXECUTABLE_IDENTITY_MISMATCH before spawn |
| same-content supervisor copy | FROZEN_EXECUTABLE_IDENTITY_MISMATCH before spawn |
| consumed Attempt-010 frozen child | FROZEN_EXECUTABLE_IDENTITY_MISMATCH before spawn |

No rejected substitution wrote supervisor evidence. Missing and partial
executable identities are rejected by the identity validator (integration
test).

## J. Future commands (not executed)

Server command — `NOT_EXECUTED`:

```text
C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe serve -hf ggml-org/Qwen3.5-0.8B-GGUF:Q4_0 -c 8192 -np 1 --metrics --reasoning on --reasoning-budget 256 --host 127.0.0.1 --port 8080
```

Frozen supervisor command — `NOT_EXECUTED`:

```text
.\target\phase1c-attempt-011-frozen\prefixity-phase1c-live-supervisor.exe --attempt-identity docs/phase-1/PHASE_1C_REASONING_BUDGET_256_ATTEMPT_011_IDENTITY_V1.json --evidence experiments/runs/phase1c-reasoning-budget-calibration/budget-256-attempt-011/supervisor.json -- .\target\phase1c-attempt-011-frozen\prefixity-phase1c-reasoning-budget-calibration.exe run-attempt-011
```

## K. Runtime accounting

```text
model_server_startups=0
port_8080_contacts=0
tcp_readiness_contacts=0
http_model_requests=0
inference_requests=0
attempt_010_executions_added=0
attempt_011_executions=0
```

## L. Deferred defects

The historical `validate_attempt_008_identity` still contains the misplaced
Attempt-009 supervisor-command check from commit `5918141`. No Attempt-011
path calls it — Attempt-008 provenance enters only through the transition
registry's hash binding — so it remains deferred.
