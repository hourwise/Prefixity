# Phase 1C h001 BASELINE timeout forensic review

Status: offline review complete. No localhost contact, listener check, model
startup, inference, retry, or evidence mutation occurred during this review.

## 1. Frozen evidence and accounting

The original `h001 / replicate 1 / BASELINE` execution remains frozen as
`AMBIGUOUS / INCONCLUSIVE`. Its execution branch is
`agent/phase-1c-scored-pilot-h001`; the reviewed execution-record commit is
`46fc5cb136c710c0ea627d80cf7fda953639deb9`, and `origin/main` remains
`46289a6c5f16bca0415ac7cd1419dfcbcf9e8f2e`.

The preserved evidence directory is
`experiments/runs/phase1c-scored-capability-v1/h001/replicate-1/baseline/`.
Ordinary SHA-256 revalidation produced:

| File | SHA-256 |
| --- | --- |
| `preflight.json` | `1072ff8ecdd503e665216ca3544ba4381b60f431eb4df47d05ebf5a219444f9` |
| `runtime-confirmation.json` | `bec37fc5982d2d775700a344dbac7a5a41bf4025d0f3ceede78196bde23441e6` |
| `readiness.json` | `1b96fcf761d07e5577d66d389d343d46b3ac87d7d047195c2110f181e02e6c7c` |
| `request-turn-1.json` | `26bc77415683d81c9f3af4e556151d8abab775b48dd5f4632ed6caba1ad25a2a` |
| `trajectory-state.json` | `dd5f1ce80a9f95f3216ab5b220658e94bc4ed0c25134e834f40a51fc6c59183c` |
| `arm-result.json` | `012b3dea5859e5279b3ddabf6a6ab7571104f9f4194975d2d9285e7b0ed453e5` |

The expected absent files remain absent: `response-turn-1.bin`,
`normalized-turn-1.json`, and `evaluator-result.json`. The dispatch is
consumed for accounting even though no response body exists:

- historical Phase 1C schema-smoke requests: `2`;
- h001 BASELINE requests: `1`;
- total Phase 1C live requests: `3`;
- h001 NO_OP and INTERVENTION requests: `0`;
- automatic retries: `0`.

## 2. Request and runtime contract

`request-turn-1.json` is `1310` bytes and has exactly these top-level fields:
`max_tokens`, `messages`, `model`, `seed`, `stream`, `temperature`, and
`top_p`. It contains two messages, in order:

1. one `system` message instructing a careful context-reasoning assistant to
   use only supplied records, return one JSON object, call no tools, and add
   no commentary;
2. one `user` message containing the h001 duplicate-message records `e001`
   and `e002`, action `a003`, and its explicit `reference_event_ids=[e001]`,
   requiring exactly `action_id`, `required_event_id`, and
   `required_content_hash`.

The exact serialized request is preserved in the evidence file. Its frozen
generation fields are `max_tokens=2048`, `temperature=0`, `top_p=1`,
`seed=1`, and `stream=false`; the model is
`ggml-org/Qwen3.5-0.8B-GGUF:Q4_0`. There are no tool definitions. Reasoning
is not a request-body field in this OpenAI-compatible projection; it was
confirmed at the runtime boundary as explicit `--reasoning on`. No
model-independent h001 input-token estimate is persisted by the task
materialization or runner, and no tokenizer was contacted or substituted
during review.

The request fingerprint and wire fingerprint both equal
`26bc77415683d81c9f3af4e556151d8abab775b48dd5f4632ed6caba1ad25a2a`, matching
the BASELINE arm projection and the frozen scored runtime contract. The
runtime timeout hierarchy is:

- connect timeout: `1000 ms`;
- complete request timeout: `600000 ms`;
- declared supervisor timeout: `660000 ms`.

## 3. Exact runner timeout path

The experiment-only runner in
`crates/prefixity-controlled-benchmark/src/phase1c_h001.rs` performs the
following bounded sequence:

1. load and validate all frozen artifacts offline;
2. perform one `TcpStream::connect_timeout` listener check;
3. persist preflight, runtime confirmation, readiness, and the exact request;
4. construct a reqwest blocking client with the `1000 ms` connect timeout,
   `600000 ms` overall request timeout, and redirects disabled;
5. issue one POST to `/v1/chat/completions` and wait for `send()` to return.

The observed error was:

`request dispatch/completion is ambiguous: error sending request for url (http://127.0.0.1:8080/v1/chat/completions)`

The measured transport duration was exactly `600000 ms`. Because the error
was returned by `send()`, no `Response` object, HTTP status, headers, or body
was available to persist. The runner then persisted `trajectory-state.json`
and `arm-result.json` with `state=AMBIGUOUS`, `turns=0`,
`transport_attempts=1`, `inference_requests=1`, and `next_arm=null`.

If headers had arrived before a later body-read timeout, the implementation
would read at most `4 MiB`, write any bytes read to `response-turn-1.bin`, and
then classify the body-read error as ambiguous. It does not persist headers
as a separate artifact before full body completion; the ambiguous result
path would not retain the status or safe headers. That alternate partial-body
path was not observed here.

The client-level timeout causes the blocking request operation to return and
the client/response state to be dropped. The runner does not start, stop, or
control the llama.cpp process, so any server-side effect of connection
cancellation is not established. There is no idempotency or server-side
completion marker that could resolve a race between request-timeout expiry
and a late server completion.

## 4. Supervisor relationship

The h001 binary contains no supervisor timer and does not reference
`supervisor_timeout_ms`; that value exists in the checked-in contract and
offline validators only. The executed `cargo run` invocation also had no
separate declared `660000 ms` outer supervisor. Therefore the intended
60-second persistence margin was not independently enforced or verified.

This is distinct from the Attempt 007 supervisor/compilation failure:
compilation completed before the live runner started, and the h001 process
did persist the ambiguous record after the request deadline. The observed
failure is not evidence of compilation consuming the supervisor window.
The missing/enforced-supervisor relationship is an implementation/design
gap, but it is not established as the cause of the request timeout.

## 5. Server-side evidence

The runner persisted no llama.cpp stdout/stderr or server log. The local
experiment tree contains no persisted `.log`, `.out`, `.err`, or transcript
for this request. The operator supplied startup lines confirming model load
and listener availability, but no completion-side server log was preserved.

`SERVER-SIDE COMPLETION STATE NOT ESTABLISHED`

The evidence therefore distinguishes a client request timeout after dispatch
from the unknown server-side cause. Transport failure, slow valid generation,
reasoning-budget execution, llama.cpp failure, process failure, and model
generation stall cannot be selected as the underlying server cause from the
preserved evidence.

## 6. Prior local-Qwen comparison

The relevant persisted observations are contextual only:

- Stage 1 Smoke 01 used reasoning enabled/default and `max_tokens=256`.
  It generated `256` completion tokens with `finish_reason=length`, empty
  assistant content, `predicted_ms=51783.184`, `prompt_ms=16719.927`, and
  `transport_elapsed_ms=68769`.
- Stage 1 Smoke 02 used reasoning off and completed successfully with `29`
  completion tokens, `predicted_ms=3017.873`, `prompt_ms=1218.0`, and
  `transport_elapsed_ms=4274`.
- P0-L6 Attempt 005 observed approximately `7020` generated tokens in
  `503.4` seconds and approximately `12.6` seconds of prompt processing,
  but its generation bound was absent and that run was invalidated. It is
  not a direct h001 estimate.
- P0-L6 Attempt 008 A0/A1 used `max_tokens=1` and observed prompt times of
  approximately `60635 ms` and `26129 ms`; those prompts are not h001
  prompts.

Smoke 01 establishes that reasoning can consume the available output budget
without producing terminal `message.content`. A 2048-token reasoning-enabled
execution exceeding `600` seconds is plausible, but not established. The
prior observations are sparse, use materially different prompts or budgets,
and contain no h001 server-side completion trace. No precise completion-time
extrapolation is justified.

The adequacy classification for the declared `2048` output allowance and
`600000 ms` request budget is therefore `NOT_ESTABLISHED`: this run did not
obtain a certifiable response within the allowance, but the evidence cannot
separate an insufficient allowance from a runtime/process failure or a late
valid generation.

## 7. Root cause and status

Strongest supported classification:

`REQUEST_TIMEOUT_BUDGET_EXHAUSTED`

Confidence is high for the observed client-side termination at the exact
registered request deadline, but low for the underlying server-side cause.
No capability failure is inferred. The original BASELINE remains
`AMBIGUOUS / INCONCLUSIVE`; no capability response was obtained. NO_OP and
INTERVENTION remain `BLOCKED`, and no valid h001 comparison exists.

## 8. Remediation options, not yet implemented

Ranked by semantic preservation, experimental cleanliness, likelihood of
addressing the observed boundary, and reproducibility:

1. **Option A — timeout-only V2.** Keep model, Q4_0, context, reasoning on,
   `max_tokens=2048`, sampling, prompt, task artifacts, evaluator, and arm
   semantics unchanged. Increase the request allowance only after a
   documented design decision, and implement/verify the outer supervisor
   with a real persistence margin. This is the smallest and most defensible
   candidate because the observed boundary was the request timeout, but it
   is not approved by this review.
2. **Option B — reduce `max_tokens`.** More likely to shorten execution, but
   changes the scored generation contract and may suppress capability.
3. **Option C — reasoning off.** Changes the scored capability configuration;
   Smoke 02 does not justify preferring it for scored work.
4. **Option D — different model, runtime, or tuning.** Highest-impact change
   and not justified by the available evidence.

The original evidence must not be overwritten. A future corrected run needs a
new scored runtime-contract version and a distinct attempt/evidence identity,
for example a new versioned experiment root under
`phase1c-scored-capability-v2/h001/replicate-1/baseline/`. That identity is
not created by this review. Any future control run requires separate live
authorization under the new contract; this review authorizes none.

## 9. Security and validation boundary

Review used checked-in Rust/source inspection, ordinary bounded file listing,
and individual `Get-FileHash` operations. No large inline PowerShell
forensic command was used, no endpoint protection was weakened, and no
preserved evidence was modified. Focused h001 tests, workspace tests,
formatting, strict Clippy, and `git diff --check` passed. Runtime evidence
remains ignored and was not force-added.
