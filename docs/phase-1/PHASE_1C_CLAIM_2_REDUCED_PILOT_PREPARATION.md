# Claim 2 reduced-cohort scored-pilot preparation

Status: `CLAIM_2_REDUCED_COHORT_SCORED_PILOT_PREPARATION_READY_FOR_EXECUTION`.
This is preparation for the separate
`CLAIM_2_REDUCED_COHORT_DOWNSTREAM_EFFICACY_FEASIBILITY_PILOT`.
It performed **zero server contacts, tokenizations, and inference requests**.
No scored model result exists.

## Frozen design and admission evidence

Starting accepted main was `1f6812983054aa460060d0bbd6e6cfd60ce3666f`.
The cohort and order are CP02 (positive), CP03 (positive), CP05
(different-state zero-mutation control), CP06 (consumer-protected
zero-mutation control). Each runs BASELINE, NO_OP, INTERVENTION, in that order.
Each arm has at most three sequential requests: **12 arms, 36 requests
maximum**. The [manifest](PHASE_1C_CLAIM_2_REDUCED_PILOT_MANIFEST.json)
binds all 36 IDs, exact body hashes, input counts, expected intermediate
actions, fixture and hidden evaluator hashes, roles, arm order, evidence
destinations, endpoint, model instrument, and scoring rules. No thirteenth
arm can be selected by the client.

The frozen client replayed the accepted renderer and canonical advancing
outputs for all 12 arms offline. Each UTF-8 request body matched the exact
corresponding V1 ledger body and V1 result SHA-256. There are 36 mappings to
14 unique exact body hashes. Independent summation of the sealed V1 result
gives BASELINE 22,165, NO_OP 22,165, INTERVENTION 20,282, and total 64,612
input tokens. These are **V1 TOKEN MEASUREMENTS INHERITED INTO REDUCED-COHORT
PILOT**, never fresh counts. The V1 token identity seal is
`4b7265e8a509829b3109947302ec82c2efec4f05cedd3aeac926324fa2a11f16`,
the raw SHA-256 is
`caf62ccaba1130a0e75d55316a1828cdcb17a8df4cc73f839f96f31a6110c264`,
and the canonical interpreted-result seal is
`03263b532a13619f9e6e38a447bf984651a712944d7c9aa83df9a86764821040`.
The client verifies these and refuses to send any body that differs from its
sealed bytes. A mismatch requires
`REDUCED_COHORT_PILOT_PREPARATION_REQUIRES_IDENTITY_REVIEW`; no recount or
fixture repair is authorized here.

The historical admission gates remain D3 >= 800, R3 >= 0.20, Rsum >= 0.08,
input_tokens <= 6000, and input_tokens + 1024 <= 8192. CP02/03 previously
passed; CP05/06 are admitted controls. There is no new admission decision.
The accepted workload [design](PHASE_1C_CLAIM_2_CONTEXT_PRESSURE_WORKLOAD_DESIGN.md)
requires a fresh independent server **per arm**, with cache state shared only
within that arm's three requests. This pilot therefore requires 12 operator
starts. No request retry, warmup, adaptive replicate, or early scientific
stopping is allowed.

## Preregistered scoring

The existing `Claim2ArmState` renderer and deterministic evaluator are the
sole scoring authority. At slots 1 and 2, assistant content must equal the
exact compact canonical expected action JSON, with no whitespace or extra
fields. Accepted actual raw content generates the pinned receipt in that
arm and is carried into the next request. A noncanonical or wrong action is
a model failure at that slot; later slots are
`NOT_EXECUTED_AFTER_FAILURE`. The client never substitutes an expected
answer or re-prompts. At slot 3, the case's existing hidden deterministic
evaluator checks schema, answer, required context and relations. Hidden keys
are loaded separately and are never sent to the model.

`ARM_TASK_SUCCESS` requires all three stages to pass without a protocol or
integrity failure. Otherwise it is `ARM_TASK_FAILURE` with the recorded
stage and reason. A BASELINE failure makes that case `BASELINE_INCAPABLE`;
it is not a treatment regression. Given BASELINE success, NO_OP failure is
`NO_OP_PIPELINE_REGRESSION`. Given both success, INTERVENTION success or
failure on CP02/03 yields `DOWNSTREAM_SUCCESS_PRESERVED` or
`DOWNSTREAM_SUCCESS_REGRESSED`. For CP05/06, the three raw slot-output
trajectories must match exactly; otherwise the classification is
`CONTROL_OUTCOME_DIVERGENCE`. If identical trajectories fail, baseline
incapability is also reported.

The study terminal priority is: pipeline/control/integrity issue ->
`REDUCED_COHORT_DOWNSTREAM_EFFICACY_INCONCLUSIVE_INTEGRITY`; observed positive
treatment regression -> `REDUCED_COHORT_DOWNSTREAM_EFFICACY_REGRESSION_OBSERVED`;
positive baseline incapability ->
`REDUCED_COHORT_DOWNSTREAM_EFFICACY_INCONCLUSIVE_BASELINE`; both positives
preserved with controls invariant ->
`REDUCED_COHORT_DOWNSTREAM_EFFICACY_FEASIBILITY_PASSED`. The last result
supports only this selected two-positive feasibility cohort, not the original
four-positive Claim-2 proof. All 12 arms run even after scientific failures;
infrastructure or evidence integrity failure stops execution.

## Instrument and transport

The model is
`D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf`, SHA-256
`cd76ec205963b3b33350093e6904d9de16c4e666fd104e1f632d25c7f15f2a13`.
The llama executable is
`C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe`, build
`b10217-ddd4ec142`, SHA-256
`cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617`.
The server uses context 8192, one slot, reasoning off, offline, 127.0.0.1:8080,
text only, no mmproj or `-hf`. Requests use max_tokens 1024, temperature 0,
top_p 1, seed 1, stream false, and no `chat_template_kwargs`. The client
allows only POST `http://127.0.0.1:8080/v1/chat/completions`; it has no
readiness, tokenization, arbitrary-URL, proxy, redirect, or server-launch
path. It checks model and executable SHA-256 and the exact operator-started
PID, command line, creation time, and sole loopback listener before sending.

## Exactly-once evidence and resume

The client runs one arm per invocation. It creates a new `arm-NN` directory
and writes `START.json`. For each logical request it durably writes its exact
body and `PLANNED`, then `DISPATCHING` **before** the HTTP send. After a
response it durably writes the complete raw HTTP body and `OBSERVED`, then
assistant content, usage, finish reason, receipt hash, deterministic score,
and `SCORED`. Successful or scientifically failed terminal arms get
`DONE.json`. Files use create-new semantics; prior evidence is never
overwritten. If dispatch or response persistence is ambiguous, the arm has
no `DONE.json`, cannot be resumed, and must not be resent. HTTP or API
envelope failures retain raw evidence and stop as integrity inconclusive.
The `next` command checks prior completed evidence and returns only the next
wholly unstarted ordinal. Completed arms are never rerun. `report` requires
all 12 completed arm records.

The [identity](PHASE_1C_CLAIM_2_REDUCED_PILOT_IDENTITY.json) binds the
manifest SHA-256, source-provenance commit
`0dde3366587710e294517a4ad41973b392923732`, V1 seals, and frozen
client at
`D:\Users\fleur\Prefixity\target\claim2-reduced-pilot-freeze\0dde336\prefixity-phase1c-claim2-reduced-pilot.exe`.
The client SHA-256 is
`232c38f426ca10cbadf00da1f31ca3cdb212c660266fa384132689c20613bfb6`,
size 4,389,888 bytes, Windows file ID
`0x0000000000000000000300000045db24`. Canonical identity seal:
`f59b3f71abae4a238cf7939dfa85bf3532d2b1608abbe0753914df111ddf65c2`.
The final frozen executable passed offline preflight: 12 arms, 36 maximum
logical requests, 14 unique bodies, 36 token mappings, 64,612 inherited
input tokens, zero actual inference and server contacts.

At historical 9–10 input tokens/s, prefill arithmetic is about 1.795–1.994
hours. With 50–150 output tokens/request at historical 1.2–2 tokens/s,
combined arithmetic is about 2.045–3.244 hours before 12 starts and checks.
At all 36 requests reaching the 1,024-token ceiling, combined arithmetic is
about 6.915–10.527 hours. These are planning estimates, not pilot timing.

## Future operator procedure — do not execute in preparation

Use an empty, dedicated evidence directory on the next separately authorized
execution task. For **each** ordinal 1 through 12: run `next`, verify it
names that ordinal, ensure the previous server has stopped and port 8080 is
free, then run `Get-ChildItem Env:LLAMA_ARG_*` and require no output. The
human operator starts exactly this server command in a separate terminal:

```powershell
C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe serve -m D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf -c 8192 -np 1 --metrics --reasoning off --offline --host 127.0.0.1 --port 8080
```

Identify its PID and check it is the sole 127.0.0.1:8080 listener. Then run
the exact frozen client, substituting that verified PID and the dedicated
evidence directory. For the first arm:

```powershell
$pilot = 'D:\Users\fleur\Prefixity\target\claim2-reduced-pilot-freeze\0dde336\prefixity-phase1c-claim2-reduced-pilot.exe'
$evidence = 'D:\Prefixity-Lab\evidence\claim2-reduced-scored-pilot-v1'
& $pilot preflight
& $pilot next --evidence $evidence
& $pilot run --ordinal 1 --evidence $evidence --operator-started-pid <verified PID>
```

For each later arm, stop and re-identify only the exact operator-started
server from the preceding arm, verify the port is free, start a fresh server
with the identical command, run `next`, then `run --ordinal N` with its new
verified PID. Never restart a partial arm. If `next` reports incomplete or
ambiguous evidence, stop for integrity review. After 12 arms, run
`& $pilot report --evidence $evidence`; preserve all files unchanged.

The only next authorized task is
`CLAIM_2_REDUCED_COHORT_SCORED_PILOT_EXECUTION`. This preparation does not
start it.
