# Phase 1C — Reasoning-budget 1024 Attempt 004 execution record

Attempt 004 was authorized for the fresh-runtime candidate with reasoning
enabled and reasoning budget 1024. The execution was required to use the
native supervisor handoff and to stop before any readiness or inference
activity if runtime identity was not proven.

## Result

The pre-live repository gate passed at commit
56cdcc879b0488ac9459ca234bb5c8ccd1dd70cc. The native prestart gate then
reported EXCLUSIVE_PRESTART with zero llama processes, zero port-8080
listeners, zero unexpected processes, successful process/TCP inspection, no
elevation, and zero network, listener, or inference contacts.

The authorized llama.cpp command was started with the frozen Attempt 004
settings: Qwen3.5-0.8B-GGUF Q4_0, context 8192, one slot, metrics enabled,
reasoning on, reasoning budget 1024, host 127.0.0.1, and port 8080. The
server loaded the model and announced the local listener. Startup also
reported an HTTPLIB repository-commit lookup failure while resolving the
hf model reference; startup continued to local model load and listening.
This was not a localhost readiness or inference request.

The mandatory native poststart identity gate failed closed before readiness:
EXPECTED_WORKFLOW_IDENTITY_INVALID, because the expected workflow launch
identity had not been handed off by the supervisor. The server had been
started directly from the authorized shell command, so no supervisor launch
identity or calibration child identity existed. No arbitrary PID or path was
fabricated, and no second workflow was started to repair the state.

The observed llama PID and listener were stopped. Native shutdown verification
then returned EXCLUSIVE_PRESTART with zero llama processes, zero port-8080
listeners, successful process/TCP inspection, no elevation, and zero network,
listener, or inference contacts.

Accordingly, Attempt 004 is INVALID / AMBIGUOUS. No case was dispatched; no
readiness check was made; inference requests are 0; readiness attempts are 0;
automatic retries are 0; and no request or response artifacts exist. The
remaining Attempt 004 cases were not run. The 512 and 256 candidates, NO_OP,
INTERVENTION, V3, and Attempt 005 were not executed or authorized.

## Preserved evidence

The ignored evidence root is
experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-004/.
The files were created after the failed-closed run and were not used to
modify prior evidence:

- preflight.json — SHA-256
  ddd3bc415f9565fd461449988618630cd95cc8bacd5c89306163aafaea164d6f
- runtime-startup.json — SHA-256
  5710b1bb021b727cee6907314d5757635255bd06f0ad2862163ee25faa914c7b
- runtime-ownership.json — SHA-256
  7206588dbadc82d31fa99739914a4be3c226318dbad34290e8cce858d619be54
- candidate-result.json — SHA-256
  eebdff4272ee9b0215397a8de1f4cd5979388533eb971e3fb69b3d66064a9ea5
- execution-record.json — SHA-256
  e002c05a3b6da64d4a5fa03ba8aa080c39182eee673cfeafac8d629e7c12c8e3

The Attempt 004 identity, native implementation, and manifest fingerprints
remain respectively:

- identity:
  7e59288ccc2847298482dfe6aa4dfe0e2d3e4197fdfbe72031f9551e51c675c9
- native implementation:
  9dba33fdc0c4c9279e5df4eb12b3be0a0f5f99492f74efa548310ebf4163c37b
- manifest:
  4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b

## Validation and disposition

The focused calibration, supervisor, and native runtime-exclusivity tests
passed after the closeout bookkeeping change. Rust formatting, strict
workspace clippy, and the repository diff check also passed before the
execution-record commit was pushed. This record is scoped to the Attempt 004
branch; it is not merged to main.

The next permitted action is runtime investigation of the missing supervisor
handoff. No further localhost/model contact is authorized by this record.

REASONING BUDGET 1024 ATTEMPT 004 INVALID — STOP FOR RUNTIME INVESTIGATION
