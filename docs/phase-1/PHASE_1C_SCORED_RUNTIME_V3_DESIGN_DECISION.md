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
| `supervisor_timeout_ms` | 1320000 | 2520000 |

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
