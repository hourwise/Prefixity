# Phase 1C scored-runtime V3 design decision

Status: `DESIGN DECISION — NO INFERENCE AUTHORIZED`

Decision: `PHASE_1C_V3_DESIGN_RECOMMENDED`, as a **final bounded
feasibility/falsification gate for the existing Qwen3.5-0.8B instrument**. It
is not an expected fix. This record authorizes no server startup, port 8080
contact, readiness check, HTTP request, or inference. It authorizes only the
offline preparation task named in section 12.

## 1. Baseline and inherited state

```text
accepted baseline            ada5c8d77f0d6adfdb2ee7eaa2de808f9cece27f
scored runtime V2            phase1c-scored-runtime-local-qwen-v2 (reasoning on, max_tokens 2048)
h001 V1 BASELINE             AMBIGUOUS / INCONCLUSIVE (request timeout)
h001 V2 BASELINE             INCONCLUSIVE - GENERATION CEILING EXHAUSTED
calibration (1024/512/256)   terminal: REASONING-ON / 2048-TOKEN SCORED CONFIGURATION NOT FEASIBLE
last admissible attempt      011 (budget 256, FAIL), consumed
```

V1, V2, and the calibration remain frozen and historically valid. Nothing in
this record reinterprets them.

## 2. Evidence basis

The calibration evidence was re-read offline, reporting only structure and
lengths; no response or reasoning text is reproduced. Two independent failure
classes are present:

| budget / case | finish | reasoning chars | final chars | leading JSON object | unique-line ratio |
| --- | --- | ---: | ---: | --- | ---: |
| 1024 / rbcal-002 | `length` | 3478 | 3273 | no | 0.27 |
| 1024 / rbcal-003 | `length` | 3941 | 3734 | no | 0.52 |
| 512 / rbcal-001 | `length` | 1420 | 2959 | no | 0.21 |
| 256 / rbcal-002 | `length` | 928 | 5909 | no | 0.39 |
| 512 / rbcal-003 | `stop` | 1936 | 110 | no (code fence) | 1.00 |
| 256 / rbcal-001 | `stop` | 691 | 178 | no | 1.00 |
| 256 / rbcal-003 | `stop` | 933 | 104 | no (code fence) | 1.00 |
| passing cases | `stop` | 1803–2816 | 96–168 | yes, exact | 1.00 |

1. **Runaway final output.** In every `length` failure, bounded reasoning ended
   and the final-content channel then produced long, highly repetitive output
   that never began with the required JSON object.
2. **Structural-format failure.** Three responses terminated normally
   (`stop`) but failed the exact structural predicate, two by wrapping the
   answer in a markdown code fence.

The only unrestricted-reasoning observations all failed to produce terminal
content: Stage 1 Smoke 01 (256 tokens) and h001 V2 (2048 tokens) exhausted
the ceiling, and h001 V1 hit the request deadline. The calibration never
tested unrestricted reasoning. A larger ceiling is therefore **not presumed** to fix
either failure class. It is tested once, as a falsification gate.

## 3. Options compared

| Option | Change | Claim it could support | Disposition |
| --- | --- | --- | --- |
| 1. Reasoning on, larger ceiling (V3) | output ceiling and deadline only | Prefixity on this exact tiny local reasoning runtime | **Selected as the final bounded gate** |
| 2. Reasoning off | reverses the accepted scored reasoning decision | Prefixity on a non-reasoning 0.8B runtime only | Not selected; no fallback |
| 3. Different capable model/runtime | changes the measuring model | Prefixity on model X; stronger evidence for a general mechanism, with fewer floor effects | The next permitted design review if the gate closes the Qwen path |
| 4. Pause or close | none | none | Pause is not needed while one bounded gate remains; closure of the Qwen path is pre-registered below |

Reasoning off is not selected. The accepted scored decision (Option C,
reasoning on) records that reasoning off "could suppress capability". Its only
evidence is one 29-token schema smoke; it was never tested on the calibration
cases. Choosing it would restrict every claim to a non-reasoning runtime, make
floor effects likely, and remove any bearing on reasoning-mode agents.

## 4. V3 runtime definition

V3 changes exactly these fields relative to scored runtime V2:

| Field | V2 | V3 |
| --- | ---: | ---: |
| `max_tokens` (output ceiling) | 2048 | 4096 |
| `complete_request_timeout_ms` | 1200000 | 2400000 |
| `supervisor_timeout_ms` | 1320000 | 2520000 (superseded; see section 14) |

Derivation:

- The context stays fixed at 8192 (prompt plus output), so 8192 is not a
  possible ceiling. 4096 is the largest round step that leaves a 4096-token
  prompt allowance, subject to the proof in section 5.
- At the slowest observed throughput (h001 V2: 2048 tokens in 598.7 s, about
  3.4 tokens/s), a full 4096-token response takes about 1198 s, which reaches
  the V2 deadline. The V3 request deadline keeps V2's implied margin: V2's
  1200 s deadline was about 2x its observed 598.7 s. The supervisor keeps V2's
  +120 s rule.

Everything else is unchanged: model, quantization, context, parallel slots,
metrics, reasoning mode, sampling, seed, prompts, tasks, tool contract,
evaluator, thresholds, arms, arm order, turns, retry policy, freshness policy,
and reasoning isolation.

### Operational definition of unrestricted reasoning

"Unrestricted reasoning" is bound to the runtime identity:

```text
reasoning             = on
reasoning_budget_flag = ABSENT
max_tokens            = 4096
context               = 8192
```

An absent flag relies on the server default. To stop it silently acquiring
different semantics through runtime drift, V3 preparation must freeze:

- the llama.cpp executable identity: path, SHA-256, size, file ID, and
  reported version (currently `b10217-ddd4ec142`, SHA-256
  `cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617`,
  15277056 bytes), re-verified before every live boundary;
- the exact resolved model file. V2 launches with the `-hf` reference and
  never froze the downloaded GGUF by hash. V3 preparation must resolve the
  cached GGUF offline, record its path, size, and SHA-256, and bind it in the
  V3 identity.

Any executable, version, or model-file mismatch before a live boundary aborts
before inference.

## 5. Input feasibility

A nominal input-ceiling field is not sufficient. Context feasibility is
established twice: statically during preparation, and dynamically before
every live request.

### Static proof (preparation)

V3 preparation must prove, using the frozen model's tokenizer and the frozen
request projection, that for every pilot and full-cohort request whose
content is fixed before dispatch:

```text
projected_prompt_tokens + 4096 <= 8192
```

- Token counts must come from a deterministic offline tokenizer bound to the
  frozen GGUF file. Starting a model server to tokenize is not authorized by
  this record.
- The proof covers all twelve full-cohort cases and all six pilot cases, every
  arm (BASELINE, NO_OP, INTERVENTION), every turn whose prompt is fully
  determined by frozen material, and the three feasibility-gate requests.
- If any statically provable request fails the inequality, V3 is infeasible
  within the fixed context and the Qwen scored path closes (section 7).

### Pre-dispatch guard (every live request)

Immediately before dispatching **every** live request, in the feasibility gate
and in any later V3 pilot, the runner tokenizes the exact request with the
same frozen tokenizer and requires:

```text
exact_prompt_tokens + 4096 <= 8192
```

If the bound fails, the runner:

- does not dispatch the request;
- records `INCONCLUSIVE_CONTEXT_BOUND`;
- does not truncate context;
- does not summarize or drop history;
- does not reduce `max_tokens`;
- does not increase context;
- does not retry.

The consequence depends on where the bound fails:

1. **V3 feasibility gate.** Any `INCONCLUSIVE_CONTEXT_BOUND`, like any `length`,
   timeout, other inconclusive, or structural/schema failure, classifies
   `CURRENT_QWEN_SCORED_PATH_CLOSED` (section 7).
2. **Subsequent V3 scored pilot.** A dynamic turn-2 or turn-3
   `INCONCLUSIVE_CONTEXT_BOUND` is an inconclusive observation for that arm.
   It counts toward the design gate's existing rule that more than 10%
   inconclusive eligible tasks pauses the study for redesign. A single later
   pilot context-bound case is **not** reclassified as a failure of the
   feasibility gate.

If BASELINE or NO_OP becomes context-bound while INTERVENTION fits because
Prefixity reduced carried context, the difference is recorded descriptively
as mechanism evidence. It does not by itself count as preservation of task
success or as a scored Prefixity win, because the control task result is
incomplete.

## 6. Bounded feasibility gate

- **Cases.** The three frozen calibration cases `rbcal-001`, `rbcal-002`, and
  `rbcal-003`, in that order, with no new case material. Their request bodies
  differ from calibration only in `max_tokens`, so V3 preparation registers
  new request hashes. Only case material is reused, not consumed attempts or
  calibration identities.
- **Runtime.** Exactly the V3 runtime of section 4, on one fresh server with
  no warmup.
- **Context guard.** Every gate request passes the section 5 pre-dispatch
  guard before dispatch.
- **Limits.** One listener check and at most three inference requests; zero
  retries, fallback requests, adaptive replicates, and warmup requests.
- **Case predicate.** Unchanged from calibration: HTTP 200, `finish_reason`
  not `length`, non-empty terminal final content, content exactly equal to the
  registered structural JSON object, and unambiguous transport. The
  predicate is fixed before any V3 outcome and must not be relaxed afterwards;
  for example, code-fenced JSON remains a failure.
- **Guarantees.** Reuse the Attempt-010/011 guarantees: frozen supervisor and
  child executables bound by SHA-256, size, and file ID; one shared
  live-prerequisite function used by both the offline traversal and the live
  path; the frozen-supervisor traversal before any live boundary; evidence
  sealing before interpretation; and integrity classification before any
  outcome is read.

### Implementation constraint

The V3 gate runner must be driven by one compact data/spec structure holding
the runtime identity, cases, ceiling, deadlines, evidence root, and execution
constraints. Validators must be generic functions of that spec. **No new
hand-cloned attempt-validator family** (`validate_attempt_0NN_*`) may be
introduced. The existing attempt-specific code remains frozen as historical
implementation.

## 7. Programme-level stopping rule

Pre-registered before any V3 inference:

| Gate result | Classification | Consequence |
| --- | --- | --- |
| All three cases PASS | `V3_FEASIBILITY_PASSED` | Authorizes only V3 pilot preparation; the pilot requires its own live authorization |
| Any model-output `length`, timeout, or `INCONCLUSIVE` case | `CURRENT_QWEN_SCORED_PATH_CLOSED` | Qwen scored path closed |
| Any structural/schema failure | `CURRENT_QWEN_SCORED_PATH_CLOSED` | Qwen scored path closed |
| Any statically provable request fails the section 5 inequality | `CURRENT_QWEN_SCORED_PATH_CLOSED` | Qwen scored path closed |
| Any gate request fails the pre-dispatch guard (`INCONCLUSIVE_CONTEXT_BOUND`) | `CURRENT_QWEN_SCORED_PATH_CLOSED` | Qwen scored path closed |
| Genuine pre-inference integrity failure with zero inference | gate identity consumed | At most one replacement gate identity; any further failure closes the path |

After `CURRENT_QWEN_SCORED_PATH_CLOSED`:

- no second output ceiling, reasoning budget, reasoning-off variant,
  prompt-format change, or other Qwen remediation;
- the next permitted design review is for a **different capable
  model/runtime** (option 3), which must then satisfy the design gate's
  provider/model boundary on its own.

Passing the gate is necessary but not sufficient. The calibration cases are
simpler than h001, so a V3 pilot remains subject to the design gate's rule
that more than 10% inconclusive eligible tasks pauses the study for redesign.
Pilot `INCONCLUSIVE_CONTEXT_BOUND` observations count toward that limit and
never reopen or reclassify the feasibility gate (section 5).

## 8. Scientific claim

If the gate passes and the V3 pilot completes, V3 can support only:

> On the frozen six-case pilot (h001, h004, h006, h007, h009, h010), using
> `ggml-org/Qwen3.5-0.8B-GGUF:Q4_0` on the frozen local llama.cpp runtime with
> reasoning on (budget flag absent), a 4096-token output ceiling, context
> 8192, temperature 0, and seed 1, a frozen Prefixity-selected intervention
> does or does not reduce context burden while preserving task success
> relative to BASELINE and NO_OP.

No claim extends to other models, quantizations, reasoning settings,
ceilings, workloads, or Prefixity as a general mechanism.

## 9. Versioning and lineage

- **Identity.** `PHASE_1C_SCORED_RUNTIME_CONTRACT_V3` (experiment
  `phase-1c-scored-capability-v3`), parent V2, with the recorded reason: V2 is
  terminally infeasible per the h001 V2 BASELINE and the terminal
  calibration.
- **V2 stays valid.** V2 is a correctly executed record of the reasoning-on /
  2048 instrument. V3 changes the instrument; it does not reinterpret V2.
- **Fresh start.** Every V3 case starts from BASELINE, then NO_OP, then
  INTERVENTION, under new V3 identities and a new evidence root. V3's h001 is
  a new predeclared cohort under a new contract, not a retry of V1 or V2.
- **No mixing.** No V1, V2, or calibration observation enters the V3 causal
  comparison; they appear only as descriptive lineage.

## 10. Reasoning budget

The reasoning budget stays unspecified: `reasoning_budget_flag = ABSENT`, as
in scored runtime V2. Adding a budget would introduce a new independent
variable. The evidence in section 2 associates forced end-of-thinking with
runaway final output.

## 11. Not authorized

- any inference, Qwen or other model startup, port 8080 contact, readiness
  check, or HTTP request;
- Attempt 012, or any reasoning budget (128, 64, or otherwise);
- a second output ceiling or any ceiling ladder;
- a reasoning-off fallback;
- a context change;
- changes to cases, prompts, tool contract, evaluator, thresholds, or the
  case predicate, including accepting code-fenced JSON;
- editing or reinterpreting V1, V2, or calibration evidence;
- reuse of consumed attempts or calibration identities;
- mixing V1/V2/calibration observations into the V3 comparison;
- a new hand-cloned attempt-validator family.

## 12. Next task and authorization boundary

This record authorizes only the next offline **PREPARATION** task:

1. freeze the llama.cpp executable identity and resolve and hash the GGUF
   model file offline;
2. prove the section 5 inequality for every statically determined pilot and
   full-cohort request with the frozen tokenizer, and specify the pre-dispatch
   guard;
3. create the V3 scored-runtime contract, V3 pilot manifest, and V3
   feasibility-gate spec, each with an SHA-256 sidecar;
4. implement the spec-driven gate runner and freeze its executables, with
   offline validation and a frozen-supervisor traversal.

Before any V3 scored BASELINE case can run, all of the following must hold:

1. that preparation completes offline, including a successful input proof;
2. the V3 feasibility gate runs under a separate live authorization and
   classifies `V3_FEASIBILITY_PASSED`;
3. V3 pilot preparation completes;
4. a separate live authorization is granted for the pilot.

Files expected in that later preparation task:

- `docs/phase-1/PHASE_1C_SCORED_RUNTIME_CONTRACT_V3.json` and `.sha256`;
- `docs/phase-1/PHASE_1C_SCORED_PILOT_MANIFEST_V3.json` and `.sha256`;
- a V3 feasibility-gate spec with its identity and sidecar;
- the spec-driven gate runner and its tests.

## 13. Amendment 1 — model loading and authoritative token counting

Accepted after the offline preparation blocker
`EXACT_OFFLINE_TOKENIZER_UNAVAILABLE`: the installed b10217 build has no
standalone tokenizer (its commands are `serve`, `cli`, `completion`, `bench`,
and similar), and its binary contains no `/apply-template` or `/tokenize`
route. Exact counts would otherwise require recreating llama.cpp's chat
template and tokenizer logic.

**Model loading.** `-hf` is replaced by direct loading of the frozen file with
`-m`:

```text
D:\Prefixity-Lab\models\models--ggml-org--Qwen3.5-0.8B-GGUF\snapshots\8fea620810c4afa23dd6443f999a48574c1611a3\Qwen3.5-0.8B-Q4_0.gguf
SHA-256 57d1997790d1744fba5b40a7317df71ea5e2acee28c47e78f0cce39c0703f8cf, 563036064 bytes
```

`-hf` re-resolved the Hugging Face reference at every launch (the cache
`refs/main` was rewritten when Attempt 011's server started) and may fetch a
multimodal projector. The frozen b10217 `--offline` flag ("forces use of
cache, prevents network access") is added as a recorded network-safety
property. No `LLAMA_ARG_*` environment variable may be present at launch.
The GGUF hash, size, and file identity must match before any token-count or
inference request.

**Authoritative token counter.**

```text
AUTHORITATIVE_V3_TOKEN_COUNTER =
frozen llama.cpp b10217 + frozen GGUF + POST /v1/chat/completions/input_tokens
```

In the frozen source (`ddd4ec1428a6201e18975ea52b07c71e0f9aef26`,
`tools/server/server-context.cpp`), `handle_count_tokens` parses the body with
`oaicompat_chat_params_parse(body, meta->chat_params, files)` and counts with
`tokenize_mixed(vocab, prompt, true, true)`, returning
`{"input_tokens": N, "object": "response.input_tokens"}` without generation.
`post_chat_completions` uses the same parse and the same text tokenization.
With `-m` and no multimodal projector the runtime is text-only, so the counter
receives the **exact inference request body with no projection**. This
replaces section 5's standalone offline tokenizer requirement; no approximate
tokenizer is used. Token counting is a model-loaded, non-inference operation.

**Accounting.** Recorded separately, and token-count contacts are never
counted as inference:

```text
operator_server_startups = 1
readiness_contacts       = 1
token_count_contacts     = 3
inference_requests      <= 3
retry_requests = fallback_requests = adaptive_replicates = warmup_requests = 0
```

**Static gate proof at the pre-inference boundary.** No separate model instance
is started for preparation. At the live gate, after runtime identity and
exclusivity checks and before any inference, the three frozen gate request
bodies are counted and each must satisfy `input_tokens + 4096 <= 8192`. Any
failure records `INCONCLUSIVE_CONTEXT_BOUND` and
`CURRENT_QWEN_SCORED_PATH_CLOSED` with `inference_requests = 0`. The static
fit analysis for pilot and full-cohort requests moves to V3 pilot
preparation; later pilot turns use the same endpoint immediately before
dispatch (section 5).

**Pre-inference failures.** A readiness failure, or a token-count request that
fails at the transport or HTTP level, is a genuine pre-inference failure
(`GATE_PRE_INFERENCE_FAILURE`, zero inference). It consumes the gate identity
and may receive the single replacement identity of section 7. A counted
request that exceeds the bound is not a pre-inference failure; it closes the
path.

**Supervisor deadline.** Resolved by Amendment 2 (section 14).

## 14. Amendment 2 — derived child-lifecycle supervisor deadline

The `2520000` ms supervisor deadline is rejected. It was V2's single-request
rule (request bound + 120 s), while the V3 child may make three sequential
inference requests plus readiness and token-count contacts. This amendment is
a preparation/spec correction only; it does not change the model, cases,
ceiling, reasoning semantics, or pass/fail rule.

**Inherited timeouts before this amendment (inspected in code).** Readiness is
one TCP listener connect (`TcpStream::connect_timeout`), not an HTTP request;
it inherited `connect_timeout_ms = 1000`. The three
`/v1/chat/completions/input_tokens` requests used the same HTTP client as
inference and therefore silently inherited the `2400000` ms generation bound.

**Explicit per-contact bounds.** `V3GateSpec.deadlines` and the contract's
`timeout_policy` now carry:

| Component | Value (ms) | Enforced by |
| --- | ---: | --- |
| `connect_timeout_ms` | 1000 | TCP connect inside each HTTP contact bound |
| `readiness_timeout_ms` | 1000 | the single listener connect |
| `token_count_request_timeout_ms` | 60000 | per-request override on each token-count POST |
| `inference_request_timeout_ms` | 2400000 | client default, used only by inference |
| `non_request_margin_ms` | 120000 | explicit; no network contact inside it |

Each HTTP bound is reqwest's complete-request bound (connect through end of
body). The connect bound must not exceed any contact bound. Token counting is a
non-generating tokenization of the exact request body, so 60000 ms is a
deliberately generous bound, not a measurement.

**Derivation.** The supervisor deadline is not an independent value:

```text
supervisor_deadline_ms =
    readiness_contacts   * readiness_timeout_ms              1 * 1000
  + token_count_contacts * token_count_request_timeout_ms    3 * 60000
  + inference_requests   * inference_request_timeout_ms      3 * 2400000
  + non_request_margin_ms                                    120000
  = 1000 + 180000 + 7200000 + 120000
  = 7501000 ms (about 125 min)
```

The derivation uses checked arithmetic and rejects zero components. A recorded
`supervisor_deadline_ms` that differs from it in either direction is rejected
by contract validation, by gate-spec validation, by gate-identity registration,
and by the supervisor's deadline reader. The superseded
`supervisor_timeout_ms` and `complete_request_timeout_ms` fields are rejected
in the V3 contract and, through `deny_unknown_fields`, in the V3 spec. The
supervisor re-derives the deadline from the bound identity's `spec.limits` and
`spec.deadlines`; the value is not compiled into the frozen executables.
Historical identity kinds (h001 V2, calibration attempts, workflow
certification) keep the fixed `1320000` ms production deadline even if they
carry V3-shaped fields.

**What the margin covers.** The supervisor's clock starts before it spawns the
child. Everything the child does outside the three contact classes falls in
`non_request_margin_ms`: process start, handoff parsing, prerequisite
validation (source hashes, frozen supervisor/child hashes, workflow
certification V2 hashes, llama.exe and GGUF identity), post-start ownership
inspection, evidence writes, and exit. None of these makes a network contact
and none has its own timer; they are bounded by file size.

Evidence recorded while preparing this amendment (2026-09-25, same host,
unoptimized `dev` profile as the frozen executables, offline, no server):

| Measured object | Bytes | `inspect` wall time (3 runs) |
| --- | ---: | --- |
| GGUF model | 563036064 | 47545, 46318, 46016 ms |
| llama.exe | 15277056 | 1297, 1305, 1219 ms |

Estimate, not measured: the child hashes about 617 MB in total outside
requests (GGUF 563 MB, llama.exe 15.3 MB, the frozen supervisor/child about
9.7 MB three times, and the certification V2 executables about 9.8 MB). At the
measured rate of about 11.8 MB/s that is about 52 s, leaving about 68 s of the
120000 ms margin for process start, process-table inspection, and evidence
writes. The margin is therefore sufficient on the observed host but not by a
large factor; a materially slower disk or memory pressure during the live run
could consume it. If the margin is exhausted, the supervisor kills the child
once and records `SUPERVISOR_TIMEOUT`; it never retries.

## 15. Amendment 3 — llama.exe Windows file identity

Classification: `HISTORICAL_LLAMA_FILE_ID_RECORDING_DEFECT`.

**Observation.** The V3 contract carried the llama.exe file ID
`volume=ba2f80f4;index=00060000001ea970` from the Attempt 009-011 identities.
`ba2f80f4` is the D: volume serial; llama.exe is on C: (`c4c93b54`). SHA-256
and size matched; `v3-dry-run` failed closed on the file ID.

**Helper check.** The repository `inspect()` opens the target with
`CreateFileW` and takes both the volume serial and the file index from
`GetFileInformationByHandle` on that handle; no cwd, drive letter, or other
object is consulted. A harness calling only the exported
`inspect_executable_identity` gave, on 2026-09-25:

| cwd | object | `file_id` |
| --- | --- | --- |
| `D:\Users\fleur\Prefixity` | C: llama.exe | `volume=c4c93b54;index=00060000001ea970` |
| `C:\Users\USER` | C: llama.exe | `volume=c4c93b54;index=00060000001ea970` |
| `D:\Users\fleur\Prefixity` and `C:\Users\USER` | D: `Cargo.toml` | `volume=ba2f80f4;index=00160000000f03dc` |
| `C:\Users\USER` | D: GGUF | `volume=ba2f80f4;index=00030000002035c4` |

`vol` reports C: `C4C9-3B54` and D: `BA2F-80F4`. The helper is correct and
cwd-independent.

**Root cause.** Attempts 007 and 008 recorded the llama.exe file ID in
`fsutil` form, `0x000000000000000000060000001ea970`, which has no volume
component. The Attempt 009 identity (commit `d389500`, documents only)
rewrote it by hand into the helper's `volume=...;index=...` form, keeping the
index and filling the volume with `ba2f80f4`, the serial shared by every
other recorded object (frozen executables and the GGUF, all on D:). Attempts
010 and 011 copied the value; their `reverification` notes re-read only
"sha256, file_size and NTFS file index". No code reads
`server_executable.windows_file_id`, so nothing detected it.

**Historical records.** The Attempt 009, 010, and 011 identities, sidecars,
preparation records, and execution evidence are unchanged. Their llama.exe
`windows_file_id` has the wrong volume component (`ba2f80f4` instead of
`c4c93b54`); the index, SHA-256, size, and path are correct. Their accepted
execution evidence remains historical and consumed; this amendment does not
reinterpret any result.

**V3 correction.** The V3 contract records
`volume=c4c93b54;index=00060000001ea970` for
`C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe`
(SHA-256 `cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617`,
15277056 bytes), and contract validation pins that value and rejects the
historical one. Regression tests bind each recorded volume to the volume that
contains the object, resolved independently through `GetVolumePathNameW` and
`GetVolumeInformationW`.
