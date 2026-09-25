# Phase 1C scored runtime V3 — feasibility-gate preparation

Status:

```text
PHASE_1C_V3_FEASIBILITY_PREPARATION_ACCEPTED
V3_FEASIBILITY_GATE_PREPARED
V3_FEASIBILITY_GATE_NOT_EXECUTED
CURRENT_QWEN_SCORED_PATH_OPEN_PENDING_GATE
NO_SCORED_INFERENCE_AUTHORIZED
```

This record prepares the single V3 feasibility gate defined in
`PHASE_1C_SCORED_RUNTIME_V3_DESIGN_DECISION.md` (sections 1-12 and
Amendments 1-3). It does not authorize or perform a live run. No model server
was started, port 8080 was not contacted, and no readiness, token-count, or
inference request was made. The gate outcome is unknown; nothing here is a
capability result.

## A. Baseline

```text
accepted design (main)          100872e  phase1c: record scored-runtime V3 design decision
source-provenance commit        b255606528388dda6bc098f7cb07a5e22d033b2b
identity                        docs/phase-1/PHASE_1C_V3_FEASIBILITY_GATE_IDENTITY_V1.json
identity canonical SHA-256      65958c75ca0b4cd8c7223b1e5382222fa182e37df73bc3a349fc219c514d242a
runtime contract                docs/phase-1/PHASE_1C_SCORED_RUNTIME_CONTRACT_V3.json
contract canonical SHA-256      f73250f33235f6dfd5396c6ed68a3e5d10f96340a1cf6e3c1c59c72707d931ef
calibration manifest SHA-256    4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b
gate identity number            1 (one replacement identity 2 permitted only after a
                                genuine zero-inference pre-inference failure)
```

The source-provenance commit contains the whole accepted V3 source candidate.
No source or contract file changed between that commit and the freeze; the
build ran on a clean tree at that commit.

## B. Runtime semantics bound by the identity

```text
reasoning                      on
reasoning_budget_flag          ABSENT
max_tokens                     4096
context                        8192
model loading                  -m <frozen GGUF>; -hf forbidden; --offline
environment                    no LLAMA_ARG_* variable
token counter                  POST http://127.0.0.1:8080/v1/chat/completions/input_tokens
context bound                  input_tokens + 4096 <= 8192, else INCONCLUSIVE_CONTEXT_BOUND without dispatch
```

These are enforced through the contract hash and the requirement that the
identity's `spec` equals the spec derived from the tracked contract and
manifest; the identity also lists them explicitly under `runtime_semantics`.

## C. Deadlines (Amendment 2)

```text
connect_timeout_ms              1000
readiness_timeout_ms            1000     x 1 readiness contact
token_count_request_timeout_ms  60000    x 3 token-count contacts
inference_request_timeout_ms    2400000  x 3 inference requests
non_request_margin_ms           120000
supervisor_deadline_ms          7501000  = 1000 + 180000 + 7200000 + 120000
```

The frozen supervisor re-derives the deadline from the identity and applied
`7501000` ms in the traversal (section F).

## D. Frozen runtime objects

```text
llama.exe  C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe
           b10217-ddd4ec142
           sha256  cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617
           size    15277056
           file_id volume=c4c93b54;index=00060000001ea970
GGUF       D:\Prefixity-Lab\models\models--ggml-org--Qwen3.5-0.8B-GGUF\snapshots\8fea620810c4afa23dd6443f999a48574c1611a3\Qwen3.5-0.8B-Q4_0.gguf
           sha256  57d1997790d1744fba5b40a7317df71ea5e2acee28c47e78f0cce39c0703f8cf
           size    563036064
           file_id volume=ba2f80f4;index=00030000002035c4
```

The llama.exe volume component is the C: serial (Amendment 3). The Attempt
009-011 records keep their historical `ba2f80f4` value unchanged.

Frozen gate requests (the three calibration cases; only `max_tokens` differs):

```text
rbcal-001  d3f7d2bfc81d...  (full hashes in the identity spec)
rbcal-002  c6a515beb0af...
rbcal-003  161b25cf79d8...
```

## E. Frozen executables and bound sources

Built with `cargo build -p prefixity-controlled-benchmark --bins --locked
--offline` (cargo/rustc 1.97.1, x86_64-pc-windows-msvc, dev profile) at the
source-provenance commit, then frozen with `v3-freeze` (non-overwriting).
Each frozen copy equals its build output byte for byte and has its own file ID.

```text
supervisor  target/phase1c-v3-feasibility-frozen/prefixity-phase1c-live-supervisor.exe
            sha256  2cdac8ec4e40e5926bf24d27b61e9152c4c80b19d9fe55a42852cd98666edc1c
            size    1410048
            file_id volume=ba2f80f4;index=000b000000369612
child       target/phase1c-v3-feasibility-frozen/prefixity-phase1c-v3-feasibility.exe
            sha256  1b0342b265ee0ff10276766bc5c823fc0af327b38df90c6341fbb083ba7697f0
            size    8274944
            file_id volume=ba2f80f4;index=000900000036a253
```

Bound sources (canonical: UTF-8, CRLF/CR normalized to LF, BOM rejected);
each equals the committed bytes:

```text
8f6cdafdbaf333163caf6ea73f747c5b7903fbb8460869fba29abe3a172d00cf  src/phase1c_v3_feasibility.rs
b0b548e50cb59a9938642388612d33e37e2a611069d109526535c0fe60f20066  src/bin/phase1c_v3_feasibility.rs
d2d5c1074333af761ed07e056f14ffd913b4478ae96d391f8447fd431665d3d2  src/phase1c_reasoning_budget_calibration.rs
0ad44ea345e0df24fae60787aefe1bc4b4990bcb11f99a40fb8d36fd41078caa  src/phase1c_live_supervisor.rs
5b3cc976f41801504a15e0c7efaf196fe7b129b1199ba2164c6b9eb19f86a282  src/bin/phase1c_live_supervisor.rs
9b9188b1e90b686d705ec5cf7cdd0ba40692c5e5f0db413bb6bc1b78c4968b29  src/phase1c_windows_runtime_exclusivity.rs
cffbff046bb7d42208d6fb85c7331a0244270e6d3b95534e860e50d6f82334dc  src/phase1c_executable_identity.rs
```

(paths relative to `crates/prefixity-controlled-benchmark/`)

The identity was generated by an untracked preparation program that calls
only the committed library's public `derive_gate_spec` and
`inspect_executable_identity`; it was not written by hand. Every field that
matters is re-validated by the committed frozen loader (sidecar, source
hashes, spec equality with the contract-derived spec, frozen executables,
runtime objects), so a generator error fails closed.

## F. Frozen offline validation

All runs used only the frozen objects, from `D:\Users\fleur\Prefixity`.

| Run | Result | Contacts |
| --- | --- | --- |
| frozen child `v3-dry-run` | rc 0, `V3_FEASIBILITY_GATE_DRY_RUN`, identity present, runtime objects match (46.6 s) | network 0, readiness 0, token count 0, inference 0 |
| frozen child `v3-preflight` | rc 0, `V3_FEASIBILITY_GATE_PREPARED` / `V3_FEASIBILITY_GATE_NOT_EXECUTED`, identity SHA matches, Windows exclusivity `READY` / `EXCLUSIVE_PRESTART` (49.9 s) | network 0, inference 0 |
| frozen supervisor -> frozen child `v3-live-prerequisites` | supervisor `COMPLETED`, applied deadline 7501000 ms, one launch, no retries; child `READY_FOR_MODEL_READINESS_BOUNDARY`, every check flag true (52.4 s) | model_server_startups 0, port_8080_contacts 0, readiness 0, token count 0, inference 0 |

The traversal stopped before post-start runtime ownership inspection, TCP
listener readiness, token counting, and inference. The gate evidence root
`experiments/runs/phase1c-scored-capability-v3/` does not exist. The
traversal's 52.4 s end-to-end time is measured evidence that the
pre-readiness non-request work fits the 120000 ms margin; it excludes the
post-start ownership inspection and evidence writes of a live run.

## G. Substitution rejection

Executable substitutions under the frozen supervisor with the registered
identity (`v3-live-prerequisites`), each rejected before child spawn with exit
code 2 and no supervisor result written:

| Substitution | Rejection |
| --- | --- |
| mutable `target/debug` supervisor | `FROZEN_EXECUTABLE_IDENTITY_MISMATCH: supervisor` |
| mutable `target/debug` child | `FROZEN_EXECUTABLE_IDENTITY_MISMATCH: child` |
| same-content copy of the frozen supervisor (identical SHA-256, new file ID) | `FROZEN_EXECUTABLE_IDENTITY_MISMATCH: supervisor` |
| same-content copy of the frozen child (identical SHA-256, new file ID) | `FROZEN_EXECUTABLE_IDENTITY_MISMATCH: child` |
| historical Attempt-011 frozen calibration child | `FROZEN_EXECUTABLE_IDENTITY_MISMATCH: child` |

Runtime-object substitutions, frozen child `v3-preflight` against a
temporarily tampered identity (restored byte-identical afterwards; the real
llama.exe and GGUF were never modified):

| Tamper | Rejection |
| --- | --- |
| llama.exe file ID set to the historical `ba2f80f4` value, sidecar resealed | spec differs from the contract-derived spec |
| llama.exe SHA-256 changed, sidecar resealed | spec differs from the contract-derived spec |
| GGUF SHA-256 changed, sidecar resealed | spec differs from the contract-derived spec |
| GGUF file ID changed, sidecar resealed | spec differs from the contract-derived spec |
| GGUF SHA-256 changed, sidecar not resealed | identity fingerprint sidecar mismatch |

Object-level comparison against on-disk objects is covered by unit tests,
all passing: `wrong_executable_or_model_identity_is_rejected`,
`changed_model_bytes_are_rejected` (real file bytes changed),
`v3_executable_identity_is_bound_to_its_containing_volume` (real llama.exe
and GGUF on this host), and `different_file_with_same_basename_is_rejected`.

## H. Build-dependent validation at the source-provenance commit

```text
cargo fmt --all -- --check                                                       pass
cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings  pass
cargo test --workspace --locked --offline                                        see below
cargo build -p prefixity-controlled-benchmark --bins --locked --offline          pass
cargo +1.86.0 check --workspace --locked --offline (MSRV, before commit)         pass
```

The first test run after the commit had one failure:
`phase1c_live_supervisor::tests::expected_workflow_handoff_serializes_supervisor_generated_identity`
reported `SUPERVISOR_TIMEOUT` instead of `COMPLETED`. The test predates V3
(commit `56cdcc8`) and gives a spawned child a 500 ms deadline. Two further
full runs passed 607/607 and five isolated runs passed; it is recorded as a
timing flake of an existing test, not changed, and not a V3 defect.

## I. Future commands (not executed; require separate live authorization)

Operator server command (fresh process, no `LLAMA_ARG_*` in its environment,
started and fully loaded before the gate runs):

```text
C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe serve -m D:\Prefixity-Lab\models\models--ggml-org--Qwen3.5-0.8B-GGUF\snapshots\8fea620810c4afa23dd6443f999a48574c1611a3\Qwen3.5-0.8B-Q4_0.gguf -c 8192 -np 1 --metrics --reasoning on --offline --host 127.0.0.1 --port 8080
```

Frozen supervisor gate command (working directory `D:\Users\fleur\Prefixity`):

```text
D:\Users\fleur\Prefixity\target\phase1c-v3-feasibility-frozen\prefixity-phase1c-live-supervisor.exe --attempt-identity docs/phase-1/PHASE_1C_V3_FEASIBILITY_GATE_IDENTITY_V1.json --evidence experiments/runs/phase1c-scored-capability-v3/feasibility-gate-supervisor-result.json -- D:\Users\fleur\Prefixity\target\phase1c-v3-feasibility-frozen\prefixity-phase1c-v3-feasibility.exe run-v3-feasibility-gate
```

## J. Runtime accounting (preparation)

```text
model_server_startups = 0
port_8080_contacts    = 0
readiness_contacts    = 0
token_count_contacts  = 0
inference_requests    = 0
gate_executions       = 0
```

## K. Notes for live review

- The readiness check is one TCP connect with a 1000 ms bound and no retry;
  the server must already be listening and loaded when the gate starts.
- Post-start ownership requires exactly one `llama.exe` process; it was not
  exercised in preparation because it needs a running server.
- The child checks its own environment for `LLAMA_ARG_*`; the server's
  environment is the operator's responsibility at launch.
- The generated launch identity reads
  `phase1c-attempt-1-budget-4096-<identity sha>`: the shared supervisor format
  renders the gate number as `attempt` and the output ceiling as `budget`.
- The identity `status` is `PREPARATION_ONLY_NO_INFERENCE_AUTHORIZED`, which
  the committed loader requires; live authorization is an operator decision
  recorded outside the identity.
