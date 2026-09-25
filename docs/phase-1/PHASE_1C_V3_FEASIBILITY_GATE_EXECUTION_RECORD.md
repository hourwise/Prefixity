# Phase 1C scored runtime V3 — feasibility-gate execution record

Status: `PHASE_1C_V3_FEASIBILITY_RESULT_ACCEPTED` — terminal gate result

```text
V3_FEASIBILITY_EXECUTED_ONCE
V3_FEASIBILITY_INTEGRITY_ACCEPTED
V3_FEASIBILITY_FAILED
CURRENT_QWEN_SCORED_PATH_CLOSED
DIFFERENT_CAPABLE_MODEL_DESIGN_REVIEW_ONLY
```

This record reports the single live execution of the prepared V3 feasibility
gate (`PHASE_1C_V3_FEASIBILITY_GATE_PREPARATION.md`). The frozen supervisor
launched the frozen child exactly once. The child counted all three frozen
requests with the authoritative token counter, all three fit the context
bound, and it made three inference requests without retry. All three cases
exhausted the complete 4096-token output ceiling (`finish_reason = length`)
without an acceptable terminal answer. Under the pre-registered stopping rule
this closes the current Qwen3.5-0.8B scored path.

## A. Starting baseline and provenance

```text
repository at execution     21b80a8666a1b02b15cd4310b4abf2f5022fccfe  (main = origin/main)
source provenance           b255606528388dda6bc098f7cb07a5e22d033b2b
gate identity (V1, gate 1)  65958c75ca0b4cd8c7223b1e5382222fa182e37df73bc3a349fc219c514d242a
runtime contract            f73250f33235f6dfd5396c6ed68a3e5d10f96340a1cf6e3c1c59c72707d931ef
calibration manifest        4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b
```

Before any contact the worktree was clean, the identity sidecar matched, and
all seven bound-source hashes matched the identity.

## B. Runtime configuration

```text
model                   Qwen3.5-0.8B Q4_0 (frozen GGUF, direct -m, --offline)
reasoning               on
reasoning_budget_flag   ABSENT
max_tokens              4096
context                 8192
parallel slots          1
temperature / top_p / seed   0 / 1 / 1
```

## C. Frozen identities (unchanged from preparation)

```text
supervisor  2cdac8ec4e40e5926bf24d27b61e9152c4c80b19d9fe55a42852cd98666edc1c  1410048  volume=ba2f80f4;index=000b000000369612
child       1b0342b265ee0ff10276766bc5c823fc0af327b38df90c6341fbb083ba7697f0  8274944  volume=ba2f80f4;index=000900000036a253
llama.exe   cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617  15277056  volume=c4c93b54;index=00060000001ea970
GGUF        57d1997790d1744fba5b40a7317df71ea5e2acee28c47e78f0cce39c0703f8cf  563036064  volume=ba2f80f4;index=00030000002035c4
```

The llama.exe and GGUF identities were re-verified by the frozen child's
offline dry run before execution and again by the gate prerequisites.

## D. Runtime server verification

The operator started the server; it was not started, restarted, or
reconfigured by the workflow.

```text
server PID          6508
parent PID          17816 (powershell.exe)
created             2026-09-25 22:52:45 (local)
llama processes     1
port-8080 listeners 1  (127.0.0.1:8080, owned by 6508)
command line        llama.exe serve -m <frozen GGUF> -c 8192 -np 1 --metrics
                    --reasoning on --offline --host 127.0.0.1 --port 8080
```

The command line equals the registered launch arguments token for token; no
`--reasoning-budget` argument was present. The gate child's environment had no
`LLAMA_ARG_*` variable. The server process's own environment is not
observable with the accepted read-only mechanism, so its `LLAMA_ARG_*`
absence is recorded as unverified rather than asserted.

## E. Supervisor execution

```text
command    target\phase1c-v3-feasibility-frozen\prefixity-phase1c-live-supervisor.exe
           --attempt-identity docs/phase-1/PHASE_1C_V3_FEASIBILITY_GATE_IDENTITY_V1.json
           --evidence experiments/runs/phase1c-scored-capability-v3/feasibility-gate-supervisor-result.json
           -- target\phase1c-v3-feasibility-frozen\prefixity-phase1c-v3-feasibility.exe run-v3-feasibility-gate
started    2026-09-25T21:55:14.721Z
ended      2026-09-25T22:11:50.784Z   (16 min 36 s; supervisor exit 0)
supervisor PID 14904; child PID 420 (parent 14904)
state COMPLETED; applied deadline 7501000 ms; one launch; no retry;
child exit 0; not terminated; supervisor network 0, inference 0
```

The handoff bound identity `65958c75…242a` and launch identity
`phase1c-attempt-1-budget-4096-65958c75…242a`; supervisor and child runtime
identities equal the frozen binding.

## F. Execution order and pre-inference checks

1. Frozen prerequisites: every check flag true.
2. Post-start ownership: `READY` / `EXCLUSIVE_POSTSTART` on server PID 6508;
   single expected llama process; port owner matches; no competing processes.
3. Readiness: one TCP listener connect, bound 1000 ms, passed in 2 ms.
4. Authoritative token counts (`POST /v1/chat/completions/input_tokens`, exact
   request body, bound 60000 ms each; not inference):

| Case | HTTP | Input tokens | + 4096 | Context bound (<= 8192) |
| --- | ---: | ---: | ---: | --- |
| rbcal-001 | 200 | 332 | 4428 | PASS |
| rbcal-002 | 200 | 335 | 4431 | PASS |
| rbcal-003 | 200 | 339 | 4435 | PASS |

5. Context decision: no request was context-bound; dispatch permitted.

The authoritative pre-inference counts exactly equal the prompt-token counts
the server later reported for each inference request.

## G. Request ledger and per-case results

The wire request files hash to the registered request hashes
(`d3f7d2bf…`, `c6a515be…`, `161b25cf…`). `max_tokens` 4096; no reasoning
budget in the request.

| Case | HTTP | finish_reason | Prompt tokens | Completion tokens | Elapsed | Transport | Result |
| --- | ---: | --- | ---: | ---: | ---: | --- | --- |
| rbcal-001 | 200 | `length` | 332 | 4096 | 316.4 s | unambiguous | FAIL |
| rbcal-002 | 200 | `length` | 335 | 4096 | 267.4 s | unambiguous | FAIL |
| rbcal-003 | 200 | `length` | 339 | 4096 | 356.1 s | unambiguous | FAIL |

Evaluator detail for every case: response JSON parsed, no tool call,
reasoning content present and diagnostic only, `terminal_final_content =
false`, `structural_response_valid = false`, error `total 4096-token ceiling
exhausted before terminal completion`. All three exhausted the complete
4096-token output ceiling without producing an acceptable terminal answer.
No response or reasoning text is reproduced here.

## H. Runtime accounting

```text
operator_server_startups = 1
readiness_contacts       = 1
token_count_contacts     = 3
inference_requests       = 3
retry_requests           = 0
fallback_requests        = 0
adaptive_replicates      = 0
warmup_requests          = 0
gate_executions          = 1
supervisor_launches      = 1
```

## I. Integrity gate

Checked before any outcome was read:

```text
V3_FEASIBILITY_INTEGRITY_ACCEPTED
```

- Supervisor completed once with the frozen supervisor and child and the
  accepted identity; no retry, no early termination, no supervisor network or
  inference.
- Prerequisites, post-start ownership, and readiness passed in the registered
  order; token counting preceded any inference and was not counted as
  inference.
- Three requests in the fixed case order with the registered hashes; HTTP 200
  and unambiguous transport for each; no retry, fallback, replicate, or
  warmup.
- The child's gate-result seal matches (section J).
- Apart from `evidence-manifest.json` and its sidecar, the evidence root
  contains exactly the 24 raw artifacts listed in the manifest; none was
  edited.

## J. Evidence freeze

Evidence root (ignored, not copied into source control):

```text
experiments/runs/phase1c-scored-capability-v3/
```

```text
gate-result seal (feasibility-gate/gate-result.sha256, canonical JSON):
0f232e7393f3fd36740c3bc62a162402094a5785d287acb2a636203b770d8635  gate-result.json

evidence-manifest.json SHA-256:
cf00330744255b9280600cb55310af44e2cb3578a557ecdae0423a769e12cfbc

sidecar:
cf00330744255b9280600cb55310af44e2cb3578a557ecdae0423a769e12cfbc  evidence-manifest.json
```

File sizes and SHA-256 values were recorded before any outcome was read,
re-verified before sealing the manifest, and again before publication. Key raw
artifact hashes:

```text
feasibility-gate-supervisor-result.json  b80512d66974e2ec5accbf3e85f4becf8c1fe0767366ea0a85812b881e8a89fe
feasibility-gate/gate-result.json        978ea031b9be0d3ddc0ac7a76982eb142c5ec8418dc4b6274164f68a9aa73be4
feasibility-gate/preflight.json          69c5a635783b187db95e9196ad91f9ff7f9c5129f23faa59f623766fadecc923
feasibility-gate/readiness.json          c1ab8a18acb0dbb0778f7e55688173692ee74bcd402b648839e5a6c0db5a12a5
feasibility-gate/token-counts.json       59b1db95fc757ce7510b93ecbb87b68d79a78d07169d59473694f70bef6ada19
rbcal-001/case-result.json               ef8461b9b6aefdfbafc913b6339f3f46b9b1326c0e8d6751c858ae223bd9032b
rbcal-002/case-result.json               b8c53705626a549914d6af6a18d5569aefa5de30fa463ad09d4ff6360785d553
rbcal-003/case-result.json               826958c4d622edac49a89e17f9410bcd90d81fc12a9a2d45e0fe0c398ec10215
```

Raw outputs were not edited, normalized, repaired, or retried.

## K. Terminal classification

The sealed gate result records case states `FAIL, FAIL, FAIL`, classification
`CURRENT_QWEN_SCORED_PATH_CLOSED`, and permission
`DIFFERENT_CAPABLE_MODEL_DESIGN_REVIEW_ONLY`.

```text
V3_FEASIBILITY_FAILED
CURRENT_QWEN_SCORED_PATH_CLOSED
DIFFERENT_CAPABLE_MODEL_DESIGN_REVIEW_ONLY
```

Scope of the result. It closes the current Qwen3.5-0.8B scored path. It is
bounded to this exact model and Q4_0 quantization, the frozen llama.cpp
b10217 runtime, reasoning on with no reasoning-budget flag, the 4096-token
ceiling in an 8192 context, the three frozen calibration cases, and the V3
protocol. It does not establish that Qwen3.5-0.8B is generally incapable or
useless, and it is not a Prefixity capability, performance, or causal result.

Explicitly not authorized: a second output ceiling, an 8192-token
experiment, a new reasoning budget, a reasoning-off fallback, a different 0.8B
quantization, a gate rerun, a V3 scored pilot, or any inference. The single
replacement identity of design section 7 applies only to a genuine
zero-inference pre-inference failure and is not available.

Next permitted substantive work: `DIFFERENT_CAPABLE_MODEL_DESIGN_REVIEW`.

## L. Cleanup

After evidence sealing and integrity classification, server PID 6508 was
re-confirmed immediately before (same executable, exact command line, parent
PID, and creation time; sole llama process and sole 8080 listener) and
terminated. Post-run checks:

```text
server exited:          yes
llama processes:        0
port-8080 listener:     none
supervisor exited:      yes
child exited:           yes
Prefixity processes:    0
```
