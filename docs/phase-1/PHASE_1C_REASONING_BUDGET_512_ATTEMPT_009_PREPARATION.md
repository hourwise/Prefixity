# Phase 1C reasoning-budget calibration — Attempt 009 preparation

Status: `PHASE_1C_ATTEMPT_009_PREPARATION_ACCEPTED`

This is an offline preparation record. It does not authorize, report, or
contain an Attempt-009 model execution.

## A. Starting state

The accepted starting baseline was:

```text
HEAD        cc473780596f4085ad21a1a468e569a7688337b4
main        cc473780596f4085ad21a1a468e569a7688337b4
origin/main cc473780596f4085ad21a1a468e569a7688337b4
worktree    clean
```

The Attempt-008 tracked execution record existed, its ignored local raw
evidence root was present, and no Attempt-009 identity or evidence root,
execution ledger, lock, stale handoff, Qwen/llama process, or port-8080 owner
was present before this preparation.

## B. Admissible transition and eligibility

Attempt 008 was independently revalidated from its canonical identity,
evidence manifest, recorded supervisor handoff, candidate result, and all 23
manifested raw files:

```text
SOURCE_ATTEMPT_008
SOURCE_BUDGET_1024
SOURCE_STATE_FAIL
SOURCE_INTEGRITY_ACCEPTED
SOURCE_CALIBRATION_ADMISSIBLE
CASE_SET_COMPLETE=true
REQUESTS=3
AUTOMATIC_RETRIES=0
FALLBACK_REQUESTS=0
ADAPTIVE_REPLICATES=0
NEXT_BUDGET_512
```

Attempt-008 identity SHA-256 is
`917fde56d11e79a3b700de82f13e5f072bda483fa6b7abe6e2da9ff37ee2dfb5`.
Its evidence-manifest SHA-256 is
`f20c4ce0149070e3ca1bc167f4400d71b88fe0bd7adac41851169ba8540e4779`.

Attempt 007 remains permanently consumed, integrity-rejected, and
calibration-inadmissible. Its historical raw `next_budget=512` was explicitly
excluded from selection. The 512 candidate is sourced only from the
integrity-accepted, calibration-admissible Attempt-008 FAIL transition.

```text
ATTEMPT_009_ELIGIBLE
CANDIDATE_BUDGET_512_AUTHORIZED
ATTEMPT_007_EXCLUDED_FROM_SELECTION
```

The deterministic provenance fixture is
`fixtures/phase1c/attempt-009-budget-provenance.json`; its validator returned
`BUDGET_512_PROVENANCE_ACCEPTED` and `SOURCE_ATTEMPT_008`.

## C. Frozen candidate contract

The candidate changes only the server-side reasoning budget from the admitted
Attempt-008 value. The calibration manifest, prompt cases, request bodies,
case order, and request hashes are unchanged:

```text
experiment_id       phase1c-reasoning-budget-calibration
attempt             009
candidate_budget    512
model_reference     ggml-org/Qwen3.5-0.8B-GGUF:Q4_0
quantization        Q4_0
case_order          rbcal-001, rbcal-002, rbcal-003
request_ceiling     3
reasoning_mode      on
reasoning_budget    512 (server-side --reasoning-budget)
context_size        8192
parallel_slots      1
temperature         0
top_p               1
seed                1
output_token_limit  2048
stream              false
automatic_retries   0
fallback_requests   0
adaptive_replicates 0
warmup              none
fresh_runtime       required
workflow_exclusive  required
```

The request hashes remain:

```text
rbcal-001  f32863dfb1da27c00a61d54986d4984569c87e9636cf5c6263c69906cb336461
rbcal-002  e9cb29143ed1be27ce5c5b27bda4daa546ff63825189b170b37083624534c1b3
rbcal-003  2c9839a9482080b3d03fa89d908c63d442d35ec64e142d5c573d276e801aec7e
```

## D. Build and frozen execution objects

All mutable build-dependent validation completed before freezing. The source
build commit was `5918141941132fc444ce34a039574974bd363753` using
`cargo 1.97.1 (c980f4866 2026-06-30)`, `rustc 1.97.1 (8bab26f4f 2026-07-14)`,
target `x86_64-pc-windows-msvc`, and lockfile SHA-256
`79f4fb5ea5e2b698c9785ac609294ed44e793dd8674f49cd53001792a2491cf8`.

The dedicated stage is `target/phase1c-attempt-009-frozen/`. It was created
by the non-overwriting `freeze_copy` helper after the build and is the only
authorized future execution location:

| object | frozen path | SHA-256 | size | Windows file ID |
| --- | --- | --- | ---: | --- |
| supervisor | `target/phase1c-attempt-009-frozen/prefixity-phase1c-live-supervisor.exe` | `aaba6a8202ccc88c4ea6c277b5c20058588cabdd98c5e5d101756e40e33aa2fb` | 1,259,008 | `volume=ba2f80f4;index=000500000038291e` |
| child | `target/phase1c-attempt-009-frozen/prefixity-phase1c-reasoning-budget-calibration.exe` | `c5c4913d1b4719b333d972ecfff11530fadbf24aaac99fde71f5a5bb8f4e1beb` | 8,813,056 | `volume=ba2f80f4;index=000a000000382aae` |

The source build paths were `target/debug/prefixity-phase1c-live-supervisor.exe`
and `target/debug/prefixity-phase1c-reasoning-budget-calibration.exe`. Those
mutable paths are build inputs only; neither is an authorized live execution
object. The frozen destination was initially absent and replacement is
refused if it already exists.

Correct frozen supervisor and child objects passed exact SHA-256, size,
Windows file-ID, and final-path binding. The existing executable-identity
regression tests rejected wrong supervisor, wrong child, and mutable
`target/debug` substitutions as `FROZEN_EXECUTABLE_IDENTITY_MISMATCH` before
child spawn. The accepted v2 certification also passed its independent
substitution and parent/child checks.

## E. Workflow certification dependency

Attempt 009 is bound to the accepted v2 workflow certification:

```text
identity: target/phase1c-workflow-identity-certification-v2-final/identity.json
canonical SHA-256: acf2d18d93b2baf08b4fa4bc5bb0846e8f3f8bedab83927cf9547240abb2d194
state: WORKFLOW_IDENTITY_CERTIFIED
namespace: 900-v2
frozen supervisor binding: true
frozen child binding: true
parent/child relationship: validated by native process table
pre-spawn mismatch rejection: true
independent evidence validation: true
path-only fallback: false
```

## F. Attempt identity and future commands

The Attempt-009 identity is
`docs/phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_009_IDENTITY_V1.json`.
Its sidecar is
`docs/phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_009_IDENTITY_V1.sha256`.
The canonical identity SHA-256 is
`0929aae1d371415d3efd92e4e7310490ad06fb69fd0398aa76aa9d857818f812`.

The following commands are recorded exactly once and are both
`NOT_EXECUTED`:

```text
C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe serve -hf ggml-org/Qwen3.5-0.8B-GGUF:Q4_0 -c 8192 -np 1 --metrics --reasoning on --reasoning-budget 512 --host 127.0.0.1 --port 8080
```

```text
.\target\phase1c-attempt-009-frozen\prefixity-phase1c-live-supervisor.exe --attempt-identity docs/phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_009_IDENTITY_V1.json --evidence experiments/runs/phase1c-reasoning-budget-calibration/budget-512-attempt-009/supervisor.json -- .\target\phase1c-attempt-009-frozen\prefixity-phase1c-reasoning-budget-calibration.exe run-attempt-009
```

The future supervisor registration resolves to attempt `009`, candidate
budget `512`, request ceiling `3`, zero automatic retries, zero fallback
requests, and zero adaptive replicates.

## G. Offline validation

The following preparation-only checks passed:

```text
attempt-009-fingerprint             PASS
attempt-009-preflight               PASS
attempt-009-dry-run                 PASS (3 fixed combinations; zero contact)
attempt-009-validate-preparation   PASS
Attempt-008 raw manifest audit      PASS (23 files; all sizes/hashes valid)
Attempt-008 handoff integrity       PASS
Attempt-009 frozen binding          PASS
v2 workflow certification           PASS (accepted identity; zero network)
substitution rejection              PASS before child spawn
cargo fmt --all -- --check          PASS
cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings PASS
cargo test --workspace --locked --offline PASS
cargo build -p prefixity-controlled-benchmark --bins --locked --offline PASS
git diff --check                    PASS
```

No preparation command created an Attempt-009 evidence file or execution
record. The evidence root remains absent.

## H. Runtime accounting

```text
model_server_startups=0
port_8080_contacts=0
tcp_readiness_contacts=0
http_model_requests=0
inference_requests=0
attempt_008_executions_added=0
attempt_009_executions=0
```

Final local preparation state:

```text
ATTEMPT_009_PREPARED
ATTEMPT_009_NOT_EXECUTED
FROZEN_EXECUTABLE_IDENTITY_BOUND
ATTEMPT_009_VIRGIN
EXCLUSIVE_PRESTART
```

No Attempt-009 live execution, readiness probe, HTTP request, inference, or
model-server startup was performed by this task.
