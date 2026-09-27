# Phase 1C local-9B capable-model feasibility gate — execution record

Status: `LOCAL_9B_FEASIBILITY_RESULT_ACCEPTED` — terminal gate result

```text
LOCAL_9B_FEASIBILITY_RESULT_ACCEPTED
LOCAL_9B_FEASIBILITY_EXECUTED_ONCE
LOCAL_9B_FEASIBILITY_INTEGRITY_ACCEPTED
LOCAL_9B_FEASIBILITY_PASSED
PREFIXITY_PILOT_CONTEXT_ADEQUACY_REVIEW_ONLY
```

This record reports the single live execution of the prepared local-9B
competence gate (`PHASE_1C_LOCAL_9B_FEASIBILITY_GATE_PREPARATION.md`). The
gate tests the Claim-2 measuring instrument, not Prefixity. The frozen
supervisor launched the frozen child exactly once; the child made three
structural requests and, after three passes, one h001 BASELINE request, with
no retry. All four passed. The result permits only the offline
`PREFIXITY_PILOT_CONTEXT_ADEQUACY_REVIEW`; it does not authorize the scored
pilot, and it is not a Prefixity capability, efficacy, or efficiency result.

## A. Starting baseline and provenance

```text
repository at execution     8348550f7d7e0345079f986e11362df6d1443238  (main = origin/main)
source provenance           2ad31436abab4b149de0b0bf425b979616b1cb8a
gate identity (V1, gate 1)  d84ba488321334c9bb885fc34ad5fdeaeff59f8dc4ac7802a8cef373eb2ef3d8
runtime contract            6b97193d0b3d8422aefe1b82d4fe28f3b02e71e85a5866839d6db82e90d9de88
```

Before any contact the tracked worktree was clean, the identity and contract
sidecars matched, and all nine bound-source hashes matched the identity.

## B. Accepted instrument

```text
model          Qwen3.5-9B Q4_K_M  (lmstudio-community/Qwen3.5-9B-GGUF @ 1379f25, Apache-2.0)
reasoning      off  (--reasoning off; chat_template_kwargs absent)
context        8192
max_tokens     1024
temperature    0
top_p          1
seed           1
stream         false
```

## C. Frozen identities (unchanged from preparation)

```text
supervisor  dd20deb27a6c486e2d75ea2914efe631d81f514b5cc365f10fd327956bf84a0a  1412096     volume=ba2f80f4;index=00080000004557b1
child       f59b7f2b0eda35f7557504b9a2cec3ebcaf956565875c7d7368120caaccb0e90  8920576     volume=ba2f80f4;index=000a0000004557b4
llama.exe   cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617  15277056    volume=c4c93b54;index=00060000001ea970
GGUF        cd76ec205963b3b33350093e6904d9de16c4e666fd104e1f632d25c7f15f2a13  5627044256  volume=ba2f80f4;index=00050000004555a6
```

The frozen executables and llama.exe were re-inspected before execution; the
llama.exe and GGUF identities were re-verified by the frozen child's offline
dry run (488976 ms, zero contacts) and again by the gate prerequisites.

## D. Runtime server verification

The operator started the server; the workflow did not start, restart, or
reconfigure it.

```text
server PID          8520
parent PID          23420 (powershell.exe)
created             2026-09-27 10:32:58 (local)
llama processes     1
port-8080 listeners 1  (127.0.0.1:8080, owned by 8520)
command line        llama.exe serve -m D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf
                    -c 8192 -np 1 --metrics --reasoning off --offline --host 127.0.0.1 --port 8080
```

The command line equals the registered launch arguments; the gate's own
parsed-argv check (`NtQueryInformationProcess`) confirmed it, including one
`--reasoning off`. The gate child's environment had no `LLAMA_ARG_*`
variable; the server process's own environment is not observable with the
accepted read-only mechanism and is recorded as unverified. Before execution
the server was idle (flat CPU time and working set after 575 s uptime) and no
Prefixity process was running.

## E. Supervisor execution

```text
started    2026-09-27T09:42:49.610Z
ended      2026-09-27T09:56:37.206Z   (13 min 48 s; supervisor exit 0, stderr empty)
child PID  21960
state COMPLETED; applied deadline 15601000 ms; one launch; no retry;
child exit 0; not terminated; supervisor network 0, inference 0
```

The handoff bound identity `d84ba488…f3d8` and ceiling 1024; supervisor and
child runtime identities equal the frozen binding. About eight minutes of the
run was prerequisite identity hashing before readiness.

## F. Execution order and pre-inference checks

1. Frozen prerequisites: every check flag true.
2. Post-start ownership: `READY` / `EXCLUSIVE_POSTSTART` on server PID 8520;
   single expected llama process; port owner matches; no competing
   processes.
3. Server command line: matches the registered effective launch contract.
4. Readiness: one TCP listener connect, bound 1000 ms, passed in 2 ms.
5. Authoritative token counts (`/v1/chat/completions/input_tokens`, exact
   request body, bound 60000 ms each; not inference) and context guard
   (`input_tokens + 1024 <= 8192`), each immediately before its dispatch:

| Stage | Case | HTTP | Input tokens | + 1024 | Fits |
| --- | --- | ---: | ---: | ---: | --- |
| A | rbcal-001 | 200 | 334 | 1358 | yes |
| A | rbcal-002 | 200 | 337 | 1361 | yes |
| A | rbcal-003 | 200 | 341 | 1365 | yes |
| B | h001 | 200 | 391 | 1415 | yes |

## G. Request ledger and results

The wire request files hash to the registered request hashes
(`5552f95e…`, `4377f602…`, `3bc1e46a…`, `5b54046e…`).

| Stage | Case | HTTP | finish_reason | Input tokens | Completion tokens | Evaluator | Classification |
| --- | --- | ---: | --- | ---: | ---: | --- | --- |
| A | rbcal-001 | 200 | `stop` | 334 | 100 | PASS | PASS |
| A | rbcal-002 | 200 | `stop` | 337 | 34 | PASS | PASS |
| A | rbcal-003 | 200 | `stop` | 341 | 36 | PASS | PASS |
| B | h001 BASELINE | 200 | `stop` | 391 | 90 | PASS | PASS |

- Structural probes: terminal non-`length` content, exact structural object,
  no reasoning content, unambiguous transport.
- h001 BASELINE (Stage 0 evaluator `stage0-deterministic-evaluator-v1`):
  task success true, required-context recall 1.0, dependency protocol valid,
  zero critical regressions, no reasoning content.

Stage B was authorized because all three structural probes passed. NO_OP and
INTERVENTION were not executed. No model text is reproduced here.

## H. Runtime accounting

```text
operator_server_startups = 1
readiness_contacts       = 1
token_count_contacts     = 4   (3 structural + 1 task-level)
inference_requests       = 4   (3 structural + 1 task-level)
retry_requests           = 0
fallback_requests        = 0
warmup_requests          = 0
adaptive_replicates      = 0
gate_executions          = 1
supervisor_launches      = 1
```

## I. Measured CPU throughput (descriptive only)

Server-reported timings on this workstation. These are observations, not
contract values, and they do not change the gate's rules.

| Request | Prompt tokens processed | Prompt speed | Decode speed | Elapsed |
| --- | ---: | ---: | ---: | ---: |
| rbcal-001 | 334 | 9.6 tok/s | 1.47 tok/s | 102.6 s |
| rbcal-002 | 295 (+42 cached) | 9.4 tok/s | 1.42 tok/s | 55.3 s |
| rbcal-003 | 299 (+42 cached) | 9.2 tok/s | 2.04 tok/s | 50.2 s |
| h001 | 391 | 10.3 tok/s | 1.22 tok/s | 111.7 s |

These fall within the preparation estimates (about 9-14 tok/s prompt, 1.2-1.6
tok/s decode). Each input count is two tokens above the Qwen3.5-0.8B count
for the same request material (332/335/339/389), consistent with the empty
think block that reasoning-off adds to the generation prompt.

## J. Integrity gate

Checked before any outcome was interpreted:

```text
LOCAL_9B_FEASIBILITY_INTEGRITY_ACCEPTED
```

- Supervisor completed once with the frozen supervisor and child and the
  accepted identity; no retry, no early termination, no supervisor network or
  inference.
- Prerequisites, post-start ownership, command-line verification, readiness,
  token counts, and dispatches occurred in the registered order and within
  the registered limits; token counting was not counted as inference.
- Four requests with the registered hashes; HTTP 200 and unambiguous
  transport for each; no retry, fallback, replicate, or warmup.
- The gate-result seal matches (section K).
- Apart from `evidence-manifest.json` and its sidecar, the evidence root
  contains exactly the 30 raw artifacts listed in the manifest; none was
  edited.

## K. Evidence freeze

Evidence root (ignored, not copied into source control):

```text
experiments/runs/phase1c-capable-model-local-9b/
```

```text
gate-result seal (feasibility-gate/gate-result.sha256, canonical JSON):
06483255c56829efeffbec728caae7ad1b6ee48a09edb9d6abf8bb6e74774a95  gate-result.json

evidence-manifest.json SHA-256:
61ce82411203882a9402240973f25110dff7035bf923a10a3e8f532a0f59f49a

sidecar:
61ce82411203882a9402240973f25110dff7035bf923a10a3e8f532a0f59f49a  evidence-manifest.json
```

File sizes and SHA-256 values of all 30 raw artifacts were recorded before
any outcome was read, re-verified before sealing the manifest, and again
before publication. Key raw artifact hashes:

```text
feasibility-gate-supervisor-result.json   7a267c6ca7488c11e7524707e603ca0ac14a54689f9d1e34678ffaebaef16c99
feasibility-gate/gate-result.json         3c2d1d38f2016d807c69db86b1b56a18cbbccb727d2b6645ea735c27a6a2b6be
feasibility-gate/preflight.json           50a3a246d9092a01c815eb85b748354943e142f6ed15d46e22e0e769f82fcfce
feasibility-gate/readiness.json           c1ab8a18acb0dbb0778f7e55688173692ee74bcd402b648839e5a6c0db5a12a5
feasibility-gate/token-counts.json        b0c7776c761ae6c0a547018787e0f6167be0443c07a3b4b12aa6e4e79b8fbd16
rbcal-001/case-result.json                a4756e58bf2b5380db8f44ed75b84dc74ee4660cd00ff45a39886b6ed45f126e
rbcal-002/case-result.json                beba2232516088501140e5b7f121542d6ef2ab56af1c248a657756cbea8c0595
rbcal-003/case-result.json                278919d3959973542e13862a8adad5ff589033b530fe17d103a9acecf069411f
h001-baseline/arm-result.json             403a86b70fd463b8ee4dd5916040570ed6e88b66baa51287e2dc7677c4ec6bc5
h001-baseline/evaluator-result.json       dd810269614daec80cdfeeebadee3cb6e99a78c894bee2d5d12c880a9dbccd61
```

Raw outputs were not edited, normalized, repaired, or retried.

## L. Terminal classification

Each dispatched request was classified from its persisted record in dispatch
order; all four were `PASS`. The sealed gate result records
`LOCAL_9B_FEASIBILITY_PASSED`, permission
`PREFIXITY_PILOT_CONTEXT_ADEQUACY_REVIEW_ONLY`, and terminal semantics:
capability result established, identity consumed, model inadequacy not
established, no cloud-GPU review authorized, no inconclusive flags.

```text
LOCAL_9B_FEASIBILITY_PASSED
PREFIXITY_PILOT_CONTEXT_ADEQUACY_REVIEW_ONLY
```

Scope: Qwen3.5-9B Q4_K_M with reasoning off is accepted as competent enough
on these four frozen probes to serve as the Claim-2 measuring instrument,
subject to the pilot-context adequacy review. The scored pilot is not
authorized, and the gate identity is consumed and must not be rerun.

## M. Cleanup

After evidence sealing and integrity classification, server PID 8520 was
re-confirmed immediately before (same executable, exact command line, parent
PID, and creation time; sole llama process and sole 8080 listener; no gate
processes) and terminated. Post-run checks:

```text
server exited:          yes
llama processes:        0
port-8080 listener:     none
Prefixity processes:    0
```
