# Phase 1C Reasoning-Budget Calibration — Attempt 007 Preparation

Status: `PHASE_1C_ATTEMPT_007_PREPARATION_ACCEPTED`

This record prepares, but does not execute, Attempt 007 in the independent
`phase1c-reasoning-budget-calibration` experiment namespace. The final
execution classification is:

```text
PHASE_1C_ATTEMPT_007_PREPARATION_ACCEPTED
ATTEMPT_007_PREPARED
ATTEMPT_007_NOT_EXECUTED
```

## Scope and namespace boundary

This is the reasoning-budget calibration Attempt 007. It is distinct from the
historical P0-L6 fresh-arm lineage whose older records describe a permanently
closed Attempt 007. That historical result is not reused, relabelled, or
aggregated into this calibration preparation. The calibration evidence root is
fixed as:

```text
experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-007
```

The accepted workflow-identity certification remains identity `900`. It is a
non-experimental certification identity and is not this Attempt 007 identity.

No Attempt 007 evidence root, supervisor result, child result, or execution
record existed before this preparation, and none was created by the offline
checks below.

## Baseline and preserved evidence

The preparation was gated against the accepted repository baseline:

```text
main/origin/main: f311b237f6f3bbde3e4f002177d5a80ed9572afe
remote:          https://github.com/hourwise/Prefixity.git
```

The baseline and `origin/main` matched before preparation. The worktree was
clean before the bounded preparation changes. No existing Attempt 006 artifact
was edited. The preserved Attempt 006 supervisor artifact remains:

```text
experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-006/supervisor.json
sha256: c8feaf01a4083d609dd5fdf5ccaf96b40db56c1cbb3ef0b7bb88f52b67724b07
```

The accepted workflow certification remains:

```text
state:              WORKFLOW_IDENTITY_CERTIFIED
classification:     PATH_REPRESENTATION_MISMATCH
identity:           900
canonical manifest: 824b65a0f93e18a12e917fd49662790bdb2dce945c6e4f8f0c8b72a9c41524a4
```

The canonical certification-manifest hash above is the accepted value recorded
by the workflow-certification evidence. Attempt 007 binds to that certification
as a prerequisite; it does not replace or mutate it.

## Frozen Attempt 007 identity

The tracked identity is:

```text
docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_007_IDENTITY_V1.json
canonical sha256: fd536cb507aab2b76511e1731d3860bbbc0074b940e4e21067ec7c6b1f4bbbe8
sidecar: docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_007_IDENTITY_V1.sha256
```

It binds the accepted baseline, calibration manifest, fixed request hashes,
Attempt 006 preserved artifact, accepted workflow certification, current source
fingerprints, current built executable identities, exact future commands, and
the separate Attempt 007 evidence root. The identity is preparation-only and
does not claim that an execution occurred.

### Experimental configuration

| Field | Frozen value |
| --- | --- |
| Experiment | `phase1c-reasoning-budget-calibration` |
| Attempt | `007` |
| Candidate budget | `1024` |
| Model reference | `ggml-org/Qwen3.5-0.8B-GGUF:Q4_0` |
| Quantization | `Q4_0` |
| Context / slots | `8192` / `1` |
| Metrics | enabled |
| Reasoning | explicitly `on` |
| Temperature / top-p / seed | `0` / `1` / `1` |
| Total output limit | `2048` tokens |
| Stream | `false` |
| Candidate order | `1024 -> 512 -> 256` in the design; this attempt is only `1024` |
| Attempt cases | `rbcal-001`, `rbcal-002`, `rbcal-003`, in that order |
| Requests | maximum `3`, one per case; no warm-up |
| Retries / fallback / adaptive retry | `0` / `0` / `0` |
| Freshness | fresh server per candidate; fresh runtime required |
| Endpoint | `127.0.0.1:8080` for a separately authorized future run |

The frozen request hashes are:

```text
rbcal-001  f32863dfb1da27c00a61d54986d4984569c87e9636cf5c6263c69906cb336461
rbcal-002  e9cb29143ed1be27ce5c5b27bda4daa546ff63825189b170b37083624534c1b3
rbcal-003  2c9839a9482080b3d03fa89d908c63d442d35ec64e142d5c573d276e801aec7e
```

The canonical calibration-manifest SHA-256 is
`4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b`.
The request bodies do not contain a reasoning-budget field; the budget is a
server launch setting, recorded separately in the identity.

### Runtime material

The validated server executable is:

```text
path:      C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe
version:   b10217-ddd4ec142
size:      15277056 bytes
sha256:    cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617
file id:   0x000000000000000000060000001ea970
```

The model is intentionally frozen as the `-hf` reference above. No local GGUF
path or local GGUF hash was present in the inspected material, so the identity
records `model_file_path: null`, `model_file_sha256: null`, and
`model_material_status: not_frozen_local_path`; it does not invent a local
model artifact.

The current offline-built executable identities bound into the preparation are:

```text
supervisor: target/debug/prefixity-phase1c-live-supervisor.exe
sha256:     8427e2ca64fce890e24d8d0474ee9e964e367b7d805caf0dbdbfb3abecc592db
file id:    0x0000000000000000000100000039821c

child:     target/debug/prefixity-phase1c-reasoning-budget-calibration.exe
sha256:    844e02c7aab8d6223c93c4f9e1c94b74877c1ec766066037bf999d9f05745a39
file id:   0x00000000000000000006000000393382
```

## Exact future launch boundary

The following commands are recorded verbatim for a separately authorized live
run. Both are explicitly `NOT_EXECUTED` in this preparation.

Future server command:

```text
C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe serve -hf ggml-org/Qwen3.5-0.8B-GGUF:Q4_0 -c 8192 -np 1 --metrics --reasoning on --reasoning-budget 1024 --host 127.0.0.1 --port 8080
```

Future supervisor command:

```text
.\target\debug\prefixity-phase1c-live-supervisor.exe --attempt-identity docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_007_IDENTITY_V1.json --evidence experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-007/supervisor.json -- .\target\debug\prefixity-phase1c-reasoning-budget-calibration.exe run-attempt-007
```

The supervisor contract remains fail-closed on parent identity, exact child
identity, handoff identity, process ownership, and endpoint ownership. The
preparation adds only the Attempt 007 registration and evidence-root binding;
it does not change the accepted workflow-identity rule or the model-facing
configuration.

## Offline evidence and contact accounting

The following checks were completed without starting the server or a live
Attempt 007 supervisor:

| Check | Result |
| --- | --- |
| `attempt-007-fingerprint` | `PREPARED`; identity, manifest, certification, source, and executable bindings match |
| `attempt-007-preflight` | `READY`, `EXCLUSIVE_PRESTART`; no llama processes, no port-8080 listeners, no competing workflow processes |
| `attempt-007-dry-run` | `DRY_RUN`; all three frozen request bytes/hashes and case order reproduced |
| Attempt 007 evidence root | absent; no result artifact created |
| Attempt 006 preserved artifact | present and hash preserved; no mutation |
| local GGUF material | no local path/hash frozen; `-hf` reference retained |

The preflight used read-only native process-table and socket-table inspection.
It did not open a TCP connection and did not perform a readiness probe.

Contact accounting for this preparation is exactly:

```text
model/server startups:       0
port-8080 listener contacts: 0
TCP readiness contacts:      0
HTTP requests:               0
inference requests:          0
Attempt 007 executions:      0
automatic retries:           0
```

The preparation also did not start a model, invoke calibration inference,
consume Attempt 007, or produce an experimental result.

## Validation performed

The bounded implementation and artifacts were validated with:

```text
cargo fmt --all
cargo build -p prefixity-controlled-benchmark --bins --locked --offline
cargo test -p prefixity-controlled-benchmark --lib --locked --offline phase1c
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings
cargo test --workspace --locked --offline
git diff --check
```

The focused suite completed with `82 passed; 0 failed; 33 filtered out`.
The offline locked build was successful. Strict Clippy, the full workspace
test suite, formatting check, and diff check also passed. The current build
produced the executable identities recorded above.
The current built supervisor and child were also run in the separate
certification-only identity `900` mode; the supervisor result was `COMPLETED`,
the child result was `WORKFLOW_IDENTITY_CERTIFIED`, and both reported zero
network, model, readiness, HTTP, inference, and retry activity. Those ignored
current-binary certification outputs are in `target/` and are not Attempt 007
evidence.

## Closeout

Attempt 007 is prepared and remains unexecuted. Any future run requires a
separate live authorization and must use the exact identity, commands, fresh
runtime, endpoint ownership, case order, request ceiling, and evidence root
recorded here. This preparation does not authorize that run.
