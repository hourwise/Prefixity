# Phase 1C reasoning-budget calibration design

Status: `PREPARATION COMPLETE - NO LIVE CALIBRATION AUTHORIZATION`

This is an independent, non-scored runtime-feasibility calibration created
after the accepted h001 V2 result. It does not reopen h001, create V3, assess
Prefixity capability, or make a correctness claim about Qwen. Its only
question is whether a bounded reasoning-on configuration can produce a
complete, parseable terminal response under the existing total
`max_tokens=2048` ceiling.

## h001 V2 closure

The accepted h001 V2 BASELINE result is recorded as:

`PHASE 1C h001 V2 - INCOMPLETE / NON-COMPARABLE`

BASELINE is `INCONCLUSIVE - GENERATION CEILING EXHAUSTED`. The request was
HTTP 200 with 2048 completion tokens and `finish_reason=length`; reasoning
content was present and terminal final content was absent. The deterministic
evaluator did not run. NO_OP and INTERVENTION were not executed. V1 and V2
evidence remain immutable and no future calibration result may be combined
with either scored lineage.

## Local llama.cpp capability

The installed executable was inspected offline at
`C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe`.
`--version` reported:

`b10217-ddd4ec142`

The installed `serve --help` reports:

- `--reasoning [on|off|auto]`; `on` explicitly enables chat reasoning;
- `--reasoning-budget N`; `-1` means unrestricted, `0` means immediate end,
  and positive `N` means a token budget for thinking;
- `--reasoning-budget-message MESSAGE`; a message injected before the
  end-of-thinking tag when the reasoning budget is exhausted.

The flags are server launch options, so the installed build exposes them as
server-side configuration. The calibration uses only `--reasoning-budget N`.
It leaves `--reasoning-budget-message` unset because that message would be an
additional runtime intervention. `--reasoning on` remains explicit. No server
was started and no localhost endpoint was contacted while establishing this
support record.

## Frozen runtime and candidate order

Every candidate retains the accepted scored-runtime settings:

- model `ggml-org/Qwen3.5-0.8B-GGUF:Q4_0`, quantization `Q4_0`;
- context `8192`, one parallel slot, metrics enabled;
- reasoning `on`, temperature `0`, top-p `1`, seed `1`;
- total `max_tokens=2048`, stream `false`;
- connect timeout `1000 ms`, request timeout `1200000 ms`, supervisor timeout
  `1320000 ms`;
- automatic retries, fallback requests, and adaptive replicates all `0`.

Exactly these server-side reasoning budgets are pre-registered, in descending
order:

`1024 -> 512 -> 256`

Each candidate uses a fresh llama.cpp process. The three independent synthetic
cases run in fixed order on that fresh process, with no warmup. The cases do
not share h001 material and the feasibility predicate concerns terminal
generation and structural completion rather than cache reuse, so a fresh
server per candidate is sufficient and is the registered freshness policy.

## Independent calibration cases

The machine-readable manifest freezes exactly three cases:

| Case | Structural workload | Request SHA-256 | Bytes |
| --- | --- | --- | ---: |
| `rbcal-001` | duplicate records with an explicit operation reference | `f32863dfb1da27c00a61d54986d4984569c87e9636cf5c6263c69906cb336461` | 1224 |
| `rbcal-002` | two dependency records with an explicit job dependency | `e9cb29143ed1be27ce5c5b27bda4daa546ff63825189b170b37083624534c1b3` | 1206 |
| `rbcal-003` | ordered facts with an explicit task source reference | `2c9839a9482080b3d03fa89d908c63d442d35ec64e142d5c573d276e801aec7e` | 1166 |

Each request contains only `model`, `messages`, `temperature`, `top_p`,
`max_tokens`, `stream`, and `seed`. The reasoning budget is deliberately not
an API request field. Each expected response is a trivial exact JSON object
with three string fields. The cases are structural feasibility probes, not
semantic capability benchmarks.

The frozen manifest is
`PHASE_1C_REASONING_BUDGET_CALIBRATION_MANIFEST_V1.json` with canonical
SHA-256:

`4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b`

Its sidecar is
`PHASE_1C_REASONING_BUDGET_CALIBRATION_MANIFEST_V1.sha256`.

## Pass, stopping, and accounting rules

A case passes only when it receives HTTP 200, the bounded response is valid
JSON, `finish_reason` is not `length`, terminal assistant content is
non-empty, the content is exactly the registered structural JSON object, and
transport is unambiguous. A candidate passes only when all three cases pass.

The runner completes the full three-case set for a candidate. A passing 1024
candidate stops calibration; otherwise the registered next candidate is 512.
A passing 512 candidate stops calibration; otherwise the registered next
candidate is 256. A failing or inconclusive 256 candidate produces the exact
terminal classification:

`REASONING-ON / 2048-TOKEN SCORED CONFIGURATION NOT FEASIBLE`

No candidate is tested twice, and no unregistered value may be supplied. The
maximum is 9 inference requests: three cases for each of the three
candidates. Runtime ambiguity is preserved as `INCONCLUSIVE`; it is never
retried or inferred to be zero.

## Experiment-only runner

The isolated runner is implemented in
`crates/prefixity-controlled-benchmark/src/phase1c_reasoning_budget_calibration.rs`
and exposed as the
`prefixity-phase1c-reasoning-budget-calibration` binary.

Offline commands are:

```text
prefixity-phase1c-reasoning-budget-calibration fingerprint
prefixity-phase1c-reasoning-budget-calibration preflight
prefixity-phase1c-reasoning-budget-calibration dry-run
prefixity-phase1c-reasoning-budget-calibration summarize --budget 1024
```

The future live candidate command is intentionally strict:

```text
prefixity-phase1c-reasoning-budget-calibration run --budget 1024 --confirm-fresh-runtime
```

The child must be run through the checked-in live supervisor under a later
authorization. The runner validates candidate order, uses the fixed server
launch template, performs one listener check per candidate, sends at most
three sequential requests, and never retries. It persists raw bounded
responses, normalized final-content records, separate reasoning diagnostics,
per-case results, and a candidate aggregate. Reasoning content is never used
as final content, scored, or passed to another case.

## 1024 launch attempt-001 closeout and attempt-002 preparation

The first live 1024 launch is immutable and classified as:

`1024 / LAUNCH ATTEMPT 001 - INVALID / AMBIGUOUS`

The reason is:

`LLAMA_RUNTIME_FAILURE_WITH_UNCERTAIN_REQUEST_COMPLETION`

`rbcal-001` produced a persisted request record, but no response was
persisted and the llama.cpp runtime subsequently failed. `rbcal-002` and
`rbcal-003` were not run. The result is not a structural infeasibility
finding, and the root cause is recorded as `ROOT CAUSE NOT ESTABLISHED`.

Attempt 002 has a separate tracked identity at
`PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_002_IDENTITY_V1.json`, with canonical
SHA-256
`7d9dd05ed5c855f02dc5b37a70e7cac257af03ce5dc87686acf1e64590f37610`.
It binds the unchanged calibration manifest
`4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b`, budget
`1024`, the unchanged three case hashes, the attempt-001 evidence root and
classification, the llama.cpp build, the three-request maximum, zero retries,
fresh-server requirement, and runtime exclusivity requirement. Its evidence
root is the distinct
`experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-002/`;
attempt 001 is not reused or overwritten.

The checked-in runner exposes offline attempt-002 commands:

```text
prefixity-phase1c-reasoning-budget-calibration attempt-002-fingerprint
prefixity-phase1c-reasoning-budget-calibration attempt-002-preflight
prefixity-phase1c-reasoning-budget-calibration attempt-002-dry-run
prefixity-phase1c-reasoning-budget-calibration attempt-002-exclusivity-preflight --confirm-no-other-workflow
```

The future live command is strict and requires the operator-supplied server
PID plus both confirmations:

```text
prefixity-phase1c-reasoning-budget-calibration run-attempt-002 --budget 1024 --server-pid PID --confirm-fresh-runtime --confirm-exclusive-runtime
```

Before server startup, the exclusivity preflight uses bounded operating-system
inspection (`tasklist /FO CSV /NH` and `netstat -ano -p tcp`) to require no
`llama.exe`, no listener on port `8080`, and no other Prefixity/Qwen/Luna/Codex
workflow process. It also requires an explicit operator attestation that no
other Luna, Codex, helper, browser, terminal, or automation workflow is
configured to interact with the runtime. After server startup and before any
calibration request, the runner records the expected PID, registered
executable path, port owner, and fresh-start identity; exactly one llama PID
must own the listener. Any mismatch blocks inference.

From fresh-server confirmation until the candidate finishes or stops, the
runtime is exclusive: no other agent, terminal workflow, automation, browser,
Luna process, Codex helper, or manual action may probe, invoke, restart, stop,
or otherwise interact with the Qwen/llama runtime or port `8080`.

Attempt-002 preparation does not change the manifest, cases, request hashes,
runtime settings, generation settings, timeout policy, candidate ordering, or
retry policy. No server was started, no listener check was run, and no
localhost or inference contact occurred during this preparation.

## Live boundary

This preparation performs no server startup, listener check, HTTP request,
inference, warmup, or calibration. The calibration requires a later explicit
authorization naming the manifest hash, candidate, fresh-server confirmation,
server launch, supervisor boundary, and evidence retention. This document
does not authorize V3 or any scored h001 replay.

## Attempt 002 closeout and Attempt 003 Windows-native exclusivity remediation

Attempt 002 is accepted as `INVALID BEFORE QWEN STARTUP` with immediate cause
`PROCESS_INSPECTION_PERMISSION_FAILURE`. The registered
`tasklist /FO CSV /NH` mechanism returned `ERROR: Access denied`. No
administrator elevation, Defender change, alternate shell syntax, Qwen
startup, port readiness check, localhost contact, inference request, or
calibration evidence-root creation occurred. The execution record is commit
`a27b4fb411a284a6503d9a1c5525e30b1ae8862c`; Attempt 001 remains immutable.

The remediation replaces shell-parsed process and port inspection with the
isolated Windows-native module
`crates/prefixity-controlled-benchmark/src/phase1c_windows_runtime_exclusivity.rs`.
The module is read-only and requires no administrator privileges. It uses
`CreateToolhelp32Snapshot`, `Process32FirstW`, and `Process32NextW` for process
enumeration; `GetExtendedTcpTable` with the owner-PID listener table for IPv4
port ownership; and `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` plus
`QueryFullProcessImageNameW` for the expected llama executable path. It never
terminates processes, elevates privileges, or contacts localhost.

The TCP adapter filters the native listener table to port `8080` before
classification. Pure classification tests cover exclusive prestart, multiple
llama processes, unrelated port ownership, PID/port mismatch, process/port
inspection failure, and executable-path failure. Inspection failures remain
fail-closed and report deterministic outcomes such as
`PROCESS_INSPECTION_FAILED`, `PORT_INSPECTION_FAILED`,
`EXECUTABLE_PATH_FAILED`, `PORT_ALREADY_OWNED`, and `PID_PORT_MISMATCH`.
Luna/Codex/browser/terminal exclusivity remains an explicit operator
attestation rather than an attempt to prove all higher-level workflow state by
process enumeration.

Attempt 003 is preparation-only. Its tracked identity is
`PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_003_IDENTITY_V1.json` with canonical
SHA-256
`c40e4528d6dc895a6e688a77dd1159b4c4e740b6a1b78ab2fc5259a0bc02655f`.
It binds the unchanged manifest SHA, all three request hashes, the Attempt 001
invalid/ambiguous lineage, the Attempt 002 pre-server invalid lineage and
execution-record commit, and the native implementation source fingerprint
`ceb145320517a84a8cad1df7695c5f1138e75a93bce2d53ae1ec4a18a37fe409`.

The distinct Attempt 003 evidence root is
`experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-003/`.
It remains absent. Offline commands are:

```text
prefixity-phase1c-reasoning-budget-calibration attempt-003-fingerprint
prefixity-phase1c-reasoning-budget-calibration attempt-003-preflight
prefixity-phase1c-reasoning-budget-calibration attempt-003-dry-run
```

The bounded local native probe completed under the normal user identity with
no `llama.exe` process and no port-8080 listener. It reported
`EXCLUSIVE_PRESTART`; OS-table inspections were separate from network contact,
with network calls `0`, listener checks `0`, and inference requests `0`.
The dry run projected exactly the three frozen 1024 cases with unchanged
request hashes and no model-visible reasoning-budget field.

No Attempt 003 live branch or runtime execution is created here. A separate
authorization is required before any future Qwen startup, readiness check, or
calibration request.
