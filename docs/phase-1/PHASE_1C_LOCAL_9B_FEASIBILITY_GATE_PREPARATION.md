# Phase 1C local-9B capable-model feasibility gate — preparation

Status:

```text
LOCAL_9B_FEASIBILITY_GATE_PREPARED
LOCAL_9B_FEASIBILITY_GATE_NOT_EXECUTED
REGISTERED_MAX_INFERENCE_REQUESTS = 4   (3 structural + 1 frozen h001 BASELINE request)
TERMINAL STATES: LOCAL_9B_FEASIBILITY_PASSED | LOCAL_9B_FEASIBILITY_FAILED |
                 LOCAL_9B_FEASIBILITY_INCONCLUSIVE | GATE_PRE_INFERENCE_FAILURE
NO_LIVE_INFERENCE_AUTHORIZED
```

This record prepares the single local-9B competence gate defined by
`PHASE_1C_CAPABLE_MODEL_DESIGN_DECISION.md`. It tests the Claim-2 measuring
instrument, not Prefixity. It does not authorize or perform a live run. The
model was downloaded and fingerprinted but never loaded: no model server was
started, port 8080 was not contacted, and no readiness, token-count, or
inference request was made. `LOCAL_9B_FEASIBILITY_PASSED` is not claimed.

## A. Baseline and provenance

```text
accepted design (main)          34f88e582d011f7434cab1707cec1f568856b999
source provenance               2ad31436abab4b149de0b0bf425b979616b1cb8a
superseded source provenance    0d19d411f60d6a8f066f9fa00fefc971133c8372 and
                                0703fa18d25055a2af4f1116106c76db86dd2f25 (preparation history only)
gate identity                   docs/phase-1/PHASE_1C_LOCAL_9B_FEASIBILITY_GATE_IDENTITY_V1.json
identity canonical SHA-256      d84ba488321334c9bb885fc34ad5fdeaeff59f8dc4ac7802a8cef373eb2ef3d8
runtime contract                docs/phase-1/PHASE_1C_CAPABLE_MODEL_RUNTIME_CONTRACT_LOCAL_9B_V1.json
contract canonical SHA-256      6b97193d0b3d8422aefe1b82d4fe28f3b02e71e85a5866839d6db82e90d9de88
calibration manifest SHA-256    4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b
```

**Superseded candidates.** No inference or gate execution occurred for
either (`inference_requests = 0`, `gate_executions = 0`), so nothing was
consumed, and neither frozen object nor identity is reused.

- `0d19d41` registered a six-request capacity. Implementation showed that the
  frozen h001 BASELINE path issues exactly one request, so the gate was
  tightened before any freeze was accepted (section D). Its frozen stage is
  archived as `target/phase1c-local-9b-feasibility-frozen-superseded-0d19d41/`;
  its identity was never tracked; its frozen dry run was stopped during
  hashing and produced no result.
- `0703fa1` (identity `62412a9b…df9a`, promoted as `3d15ef8`) classified an
  infrastructure failure after dispatch as `LOCAL_9B_FEASIBILITY_FAILED`.
  That over-states the capability result, so the terminal classification was
  split (section I) before live authorization. Its frozen stage and traversal
  output are archived with the suffix `-superseded-0703fa1`. The correction
  also fixed a defect: after an ambiguous h001 transport the gate would have
  tried to score a normalized turn that the h001 executor does not write.

## B. Model artifact (acquired once, on D:)

```text
repository     lmstudio-community/Qwen3.5-9B-GGUF (LM Studio team, llama.cpp b8185)
revision       1379f25c6b505a3fc737bd7818cb09389cf807c1
file           Qwen3.5-9B-Q4_K_M.gguf  (advertised 5627044256 bytes)
source URL     https://huggingface.co/lmstudio-community/Qwen3.5-9B-GGUF/resolve/1379f25c6b505a3fc737bd7818cb09389cf807c1/Qwen3.5-9B-Q4_K_M.gguf
base model     Qwen/Qwen3.5-9B @ c202236235762e1c871ad0ccb60c8ee5ba337b9a
license        Apache-2.0 (upstream LICENSE file verified; repositories public, ungated)
local path     D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf
SHA-256        cd76ec205963b3b33350093e6904d9de16c4e666fd104e1f632d25c7f15f2a13 (equals upstream LFS SHA-256)
size           5627044256
file ID        volume=ba2f80f4;index=00050000004555a6  (D: volume serial BA2F-80F4)
GGUF metadata  architecture qwen35, file_type 15 (Q4_K_M), embedded chat template SHA-256
               a4aee8afcf2e0711942cf848899be66016f8d14a889ff9ede07bca099c28f715
```

Downloaded once with a revision-pinned URL to a `.partial` file (no automatic
retry), verified against the upstream LFS SHA-256, then renamed before any
identity was registered. The repository `inspect()` gave the same identity
from a D: working directory and a C: working directory. The multimodal
projector was not acquired; `--mmproj` is forbidden.

## C. Runtime and reasoning-off semantics

```text
llama.exe  C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe  (not moved)
           b10217-ddd4ec142, SHA-256 cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617,
           15277056 bytes, volume=c4c93b54;index=00060000001ea970  (matches the accepted identity)
```

Reasoning-off semantics were established from the exact-build source
(commit `ddd4ec1428a6201e18975ea52b07c71e0f9aef26`), the binary's help, and
the GGUF header, without loading the model:

- `--reasoning off` sets `enable_reasoning = 0` and the server default
  template kwarg `enable_thinking = false`; it is server-wide.
- A request's `chat_template_kwargs.enable_thinking` or `reasoning_effort`
  can override that default per request (`oaicompat_chat_params_parse`).
- `/v1/chat/completions` and `/v1/chat/completions/input_tokens` call the same
  parse with the same server chat parameters, so token counts use the same
  reasoning-off template semantics as inference.
- The embedded template ends the generation prompt with an empty think block
  when `enable_thinking` is false; reasoning-off changes request formatting,
  not only generation behaviour.
- Environment variables are applied before command-line arguments, so an
  explicit `--reasoning off` overrides `LLAMA_ARG_REASONING` and an
  `enable_thinking` from `LLAMA_ARG_CHAT_TEMPLATE_KWARGS`; no `LLAMA_ARG_*`
  variable may be present.

Enforcement in the gate:

- launch contract requires `--reasoning off`;
- every registered request has `chat_template_kwargs` ABSENT (any value,
  including `enable_thinking: false`, is rejected), and `reasoning_effort`,
  `reasoning_format`, `reasoning_budget_tokens`, `thinking_budget_tokens`, and
  `reasoning_budget_message` absent; the registered request hashes bind that
  absence;
- request message content must be plain text strings (`mmproj = ABSENT`);
- before any contact the gate reads the verified server process's command
  line (`NtQueryInformationProcess`, windows-sys `Wdk_System_Threading`
  feature) and parses its effective options.

No non-inference model load was required.

## D. Gate shape, limits, and deadline

Stage A sends the three frozen `rbcal` probes to the unchanged calibration
evaluator (HTTP 200, terminal non-`length` content, exact three-key object,
code fences rejected). Only if all three pass, Stage B sends one h001
BASELINE request to the unchanged Stage 0 evaluator
(`stage0-deterministic-evaluator-v1`, no-tool contract
`phase1c-h001-no-tools-v1`). The frozen h001 no-tool contract terminates on
its first response; `MAX_TURNS = 3` in the historical h001 design is a
declared bound, not an active loop. NO_OP and INTERVENTION are not executed.

```text
REGISTERED_MAX_INFERENCE_REQUESTS = 4 = 3 structural + 1 frozen h001 BASELINE request
structural_inference_limit = 3     task_inference_limit = 1
max_token_count_contacts   = 4     readiness_contacts   = 1
retry = fallback = warmup = adaptive_replicates = 0
if Stage A fails: task_token_count_contacts = 0, task_inference_requests = 0
```

Stage capacity is checked before every token count and dispatch. Envelope:
reasoning off, context 8192, `max_tokens` 1024, temperature 0, top_p 1,
seed 1, stream false. Every request is counted with
`/v1/chat/completions/input_tokens` and guarded with
`input_tokens + 1024 <= 8192`; a bound failure is not dispatched and fails
the gate.

Derived supervisor deadline (structural and task inference share one bound):

```text
readiness            1 x    1000 =     1000
structural counts    3 x   60000 =   180000
task count           1 x   60000 =    60000
structural inference 3 x 3540000 = 10620000
task inference       1 x 3540000 =  3540000
non-request margin               =  1200000
supervisor deadline              = 15601000 ms
```

- Inference bound 3540000 ms: a conservative preparation estimate, not a 9B
  measurement (a full 1024-token response at about 0.3 tokens/s; the 0.8B
  runs varied about 4x between sessions), kept below the server's default
  3600 s read/write timeout.
- Margin 1200000 ms: measured debug-build GGUF identity hashing took 550943
  ms idle and 674945 ms under concurrent builds; the frozen traversal
  (section F) took 493774 ms end to end.

## E. Frozen executables and bound sources

Built at `2ad3143` from a clean tree with
`cargo build -p prefixity-controlled-benchmark --bins --locked --offline`
(cargo/rustc 1.97.1, x86_64-pc-windows-msvc, dev profile) after fmt, clippy
(`-D warnings`), workspace tests (628 passed), and an MSRV 1.86 check, then
frozen with `local-9b-freeze` (non-overwriting). Each frozen copy equals its
build output and has its own file ID; both differ in content and file ID
from the superseded candidates. One pre-commit test run hit the pre-existing
500 ms handoff timing flake
(`expected_workflow_handoff_serializes_supervisor_generated_identity`, from
`56cdcc8`); two further full runs passed 628/628 and the test is unchanged.

```text
supervisor  target/phase1c-local-9b-feasibility-frozen/prefixity-phase1c-live-supervisor.exe
            dd20deb27a6c486e2d75ea2914efe631d81f514b5cc365f10fd327956bf84a0a  1412096  volume=ba2f80f4;index=00080000004557b1
child       target/phase1c-local-9b-feasibility-frozen/prefixity-phase1c-capable-model-gate.exe
            f59b7f2b0eda35f7557504b9a2cec3ebcaf956565875c7d7368120caaccb0e90  8920576  volume=ba2f80f4;index=000a0000004557b4
```

Bound sources (UTF-8, CRLF/CR normalized to LF), each equal to `2ad3143`:

```text
0aa2a8af1208e402e422bfb68022b46b77b23853515cfa712f44871958d076fa  src/phase1c_capable_model_gate.rs
c3dd2ebc4118596c5d44c251cf74e0a0ae34f0af3fdea86b30763eeaf7580e28  src/bin/phase1c_capable_model_gate.rs
45a5b00794b3ccca88c7bee48b548c8aad57524bfd84e190065ed6e6cde6781e  src/phase1c_v3_feasibility.rs
44ceed653871aefa08feb44a85f41d6d560e652acaa9aebfb7885eec7d825375  src/phase1c_reasoning_budget_calibration.rs
3e9504aa39d54bc50c78de0b61e72bdf472cb6e2fdfdea9ae6a348cf883e1325  src/phase1c_h001.rs
906b76afb8e78817108644df406773d58710d501697373a25a194ce56feebfcb  src/phase1c_live_supervisor.rs
5b3cc976f41801504a15e0c7efaf196fe7b129b1199ba2164c6b9eb19f86a282  src/bin/phase1c_live_supervisor.rs
9b9188b1e90b686d705ec5cf7cdd0ba40692c5e5f0db413bb6bc1b78c4968b29  src/phase1c_windows_runtime_exclusivity.rs
cffbff046bb7d42208d6fb85c7331a0244270e6d3b95534e860e50d6f82334dc  src/phase1c_executable_identity.rs
```

(paths relative to `crates/prefixity-controlled-benchmark/`)

Registered requests (only the model label and `max_tokens` differ from the
frozen request material):

```text
rbcal-001  5552f95ea160e023f06efe29641e2dd378e9d136511e93d3f7a0d91b11a35c50
rbcal-002  4377f602a9e99ba25c318f8c856e54e363d39b92846006b83c1f7b300847e9ab
rbcal-003  3bc1e46aef64a6975c11988f8d0d3fe53354a7fe2194fb8857d94c0fed14ce9e
h001       5b54046ecf52aefee74271731f3728c620e56a5cba3490c501cb29d60bf60a05  (BASELINE)
h001 evaluator 5fc19f47bd36e8625535e63939caebbb0e9ca1a458bb41b357de9d3f6d1f7fe3
```

The identity was generated by an untracked preparation program that calls
only the committed library's public `derive_local_9b_gate_spec` and
`inspect_executable_identity`; the committed frozen loader re-validates it.

**Source generalizations (minimum necessary).** A data/spec-driven
`CapableModelGateSpec` gate reusing the V3 types, deadline derivation, token
counter, context guard, and ownership helpers; the supervisor registers gate
identities through a small gate-kind table (V3 unchanged); calibration
request records take their experiment and model labels from the manifest and
request actually used (identical for historical calls, whose manifest pins
them); h001 single-turn dispatch is extracted into a helper that the
historical executor still calls in the same order; both executors record
`transport_timeout` on their ambiguous-transport branches so a timeout after
dispatch is distinguishable from other transport failures. The consumed V3 gate
identity now fails closed on its source binding (tested). No historical
identity or evidence changed.

## F. Frozen offline validation

All runs used only the frozen objects, from `D:\Users\fleur\Prefixity`, with
no llama process and no port-8080 listener.

| Run | Result |
| --- | --- |
| frozen child `local-9b-dry-run` | rc 0, `LOCAL_9B_FEASIBILITY_GATE_DRY_RUN`, identity present, runtime objects match; 568765 ms |
| frozen child `local-9b-preflight` | rc 0, `LOCAL_9B_FEASIBILITY_GATE_PREPARED` / `LOCAL_9B_FEASIBILITY_GATE_NOT_EXECUTED`, identity SHA matches, Windows exclusivity `READY` / `EXCLUSIVE_PRESTART`; 495614 ms |
| frozen supervisor -> frozen child `local-9b-live-prerequisites` | supervisor `COMPLETED`, applied deadline 15601000 ms, one launch, no retry; child `READY_FOR_MODEL_READINESS_BOUNDARY`, every check flag true; 493774 ms |

Every run: model_server_startups 0, port_8080_contacts 0, readiness 0,
token counts 0, inference 0. The traversal stopped before post-start
ownership inspection, server command-line verification, readiness, token
counting, and inference. The gate evidence root does not exist.

## G. Substitution rejection

Executable substitutions under the frozen supervisor (`local-9b-live-prerequisites`),
each rejected before child spawn with exit code 2 and no supervisor result:

| Substitution | Rejection |
| --- | --- |
| mutable `target/debug` supervisor | `FROZEN_EXECUTABLE_IDENTITY_MISMATCH: supervisor` |
| mutable `target/debug` child | `FROZEN_EXECUTABLE_IDENTITY_MISMATCH: child` |
| same-content supervisor copy (identical SHA-256, new file ID) | `FROZEN_EXECUTABLE_IDENTITY_MISMATCH: supervisor` |
| same-content child copy (identical SHA-256, new file ID) | `FROZEN_EXECUTABLE_IDENTITY_MISMATCH: child` |
| historical V3 frozen child | `FROZEN_EXECUTABLE_IDENTITY_MISMATCH: child` |
| historical Attempt-011 frozen child | `FROZEN_EXECUTABLE_IDENTITY_MISMATCH: child` |

Identity tampering against the frozen child's `local-9b-preflight`
(temporarily modified, restored byte-identical; the real GGUF and llama.exe
were never modified):

| Tamper (sidecar resealed unless noted) | Rejection |
| --- | --- |
| GGUF SHA-256 changed | spec differs from the contract-derived spec |
| GGUF file ID changed | spec differs from the contract-derived spec |
| GGUF path changed to a 4B file | spec differs from the contract-derived spec |
| llama.exe SHA-256 changed | spec differs from the contract-derived spec |
| llama.exe file ID set to the historical D: volume | spec differs from the contract-derived spec |
| `--reasoning on` in launch arguments | spec differs from the contract-derived spec |
| reasoning mode and flag set to on | spec differs from the contract-derived spec |
| six-request capacity with its own consistent 22801000 ms deadline | spec differs from the contract-derived spec |
| h001 request hash changed | spec differs from the contract-derived spec |
| GGUF SHA-256 changed, sidecar not resealed | identity fingerprint sidecar mismatch |

Unit tests (all passing) cover the remaining live-only checks:

- server command line: exactly one `--reasoning off` required; omitted,
  `on`, `auto`, duplicate, conflicting, `-rea` alias, and `--reasoning=off`
  forms rejected; `--reasoning-budget`, `--chat-template-kwargs`, `-hf`,
  `--mmproj`, `-mm`, another `-m` path, `--model` alias, duplicate `-m`,
  `-c 4096`, `--ctx-size`, `-np 2`, missing `--metrics`, missing
  `--offline`, `--host 0.0.0.0`, `--port 8081`, a missing value, an
  unregistered flag, a non-`serve` subcommand, and another executable
  rejected; argument order alone does not change the effective options;
- requests: `chat_template_kwargs` with `enable_thinking: true`,
  `enable_thinking: false`, `{}`, or any other key rejected, and every other
  reasoning field rejected; array/image content and empty messages rejected;
- the registered GGUF volume equals its containing volume; the supervisor
  rejects the former six-request capacity and V3 limits for this gate kind;
- terminal classification (section I): zero-inference failure keeps
  replacement eligibility; structural FAIL, `length`, a timeout after
  dispatch, and an h001 evaluator FAIL give `LOCAL_9B_FEASIBILITY_FAILED`;
  infrastructure failure after structural inference but before a later
  count or dispatch, and a non-timeout transport failure, non-200 status, or
  unusable body of a dispatched request give `LOCAL_9B_FEASIBILITY_INCONCLUSIVE`
  with the identity consumed and no replacement; only four passes give
  `LOCAL_9B_FEASIBILITY_PASSED`. The persisted calibration and h001 record
  shapes are mapped by the same tests.

## H. Future commands (not executed; require separate live authorization)

Operator server command (fresh process, no `LLAMA_ARG_*` in its environment,
fully loaded before the gate runs):

```text
C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe serve -m D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf -c 8192 -np 1 --metrics --reasoning off --offline --host 127.0.0.1 --port 8080
```

Frozen supervisor gate command (working directory `D:\Users\fleur\Prefixity`):

```text
D:\Users\fleur\Prefixity\target\phase1c-local-9b-feasibility-frozen\prefixity-phase1c-live-supervisor.exe --attempt-identity docs/phase-1/PHASE_1C_LOCAL_9B_FEASIBILITY_GATE_IDENTITY_V1.json --evidence experiments/runs/phase1c-capable-model-local-9b/feasibility-gate-supervisor-result.json -- D:\Users\fleur\Prefixity\target\phase1c-local-9b-feasibility-frozen\prefixity-phase1c-capable-model-gate.exe run-local-9b-feasibility-gate
```

## I. Stopping rule and terminal states

Every dispatched request is classified from its persisted record, in
dispatch order, and the first non-pass decides.

- **`LOCAL_9B_FEASIBILITY_PASSED`** only for the complete registered path:
  three structural PASSes and an h001 BASELINE PASS, with no `length`,
  timeout, context bound, or infrastructure failure. It permits only the
  offline `PREFIXITY_PILOT_CONTEXT_ADEQUACY_REVIEW`; the scored pilot stays
  unauthorized.
- **`LOCAL_9B_FEASIBILITY_FAILED`** (model-side) only for evidence
  attributable to the model or scored request path: an evaluator FAIL or
  structural failure, `finish_reason = length`, an inference timeout after
  dispatch, no acceptable terminal answer, or a context bound under the
  registered rule. It closes the local-9B Claim-2 path and permits only
  `CLOUD_GPU_CAPABLE_MODEL_DESIGN_REVIEW`. No reasoning enable, ceiling raise,
  quantization or file change, 4B step-down, or rerun.
- **`GATE_PRE_INFERENCE_FAILURE`** for a readiness, token-count, or executor
  failure while `inference_requests = 0`: exactly one replacement identity may
  be considered under the registered rule; nothing broader.
- **`LOCAL_9B_FEASIBILITY_INCONCLUSIVE`** for an infrastructure or integrity
  failure after any inference was dispatched: a transport failure other than
  a timeout, a non-200 status, an unusable response body, or a token-count or
  executor failure after dispatch (for example, three structural passes and
  then a failing h001 token count). Flags:
  `LOCAL_9B_GATE_IDENTITY_CONSUMED`, `INFRASTRUCTURE_AFTER_DISPATCH`,
  `NO_REPLACEMENT_AUTHORIZED`, `DESIGN_REVIEW_REQUIRED`. It establishes
  neither a pass nor model inadequacy, authorizes no replacement identity and
  no cloud-GPU progression, and requires an explicit design decision.

Each sealed gate result records `identity_consumed`,
`replacement_identity_eligible`, `capability_result_established`,
`model_inadequacy_established`, `cloud_gpu_review_authorized`, and the flags.

## J. Notes for live review

- The readiness check is one TCP connect (1000 ms, no retry); the server must
  be fully loaded first.
- Post-start ownership and server command-line verification need a running
  server and were exercised only by unit tests.
- The frozen child hashes the 5.6 GB GGUF (about 8-9 minutes in the debug
  build) before readiness.
- The launch identity reads `phase1c-attempt-1-budget-1024-<identity sha>`:
  the shared supervisor format renders the gate number as `attempt` and the
  output ceiling as `budget`.
