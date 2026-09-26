# Phase 1C capable measuring-model design decision

Status: `DESIGN DECISION — NO INFERENCE AUTHORIZED`

```text
LOCAL_9B_FEASIBILITY_RECOMMENDED
CLAIM_2_PRIMARY_REASONING_OFF
LOCAL_9B_SINGLE_GATE_ONLY
PILOT_CONTEXT_ADEQUACY_REVIEW_REQUIRED_AFTER_PASS
NO_LIVE_INFERENCE_AUTHORIZED
```

This record selects the next Claim-2 measuring instrument after the V3
feasibility gate closed the Qwen3.5-0.8B scored path. It authorizes no model
download, llama.cpp startup, port 8080 contact, readiness check, HTTP request,
or inference, and no harness implementation. The goal is not to make the
0.8B model work; it is to select a model competent enough at BASELINE that
Prefixity's effect on context burden can be measured without a
capability-floor confound.

## 1. Baseline and inherited state

```text
repository                 b35796d2f3e35266afd011ee4114fbe60ddfe2bb (main)
V3 result                  PHASE_1C_V3_FEASIBILITY_RESULT_ACCEPTED
                           V3_FEASIBILITY_FAILED
                           CURRENT_QWEN_SCORED_PATH_CLOSED
permitted next work        DIFFERENT_CAPABLE_MODEL_DESIGN_REVIEW_ONLY
```

The Qwen3.5-0.8B scored path remains permanently closed
(`PHASE_1C_V3_FEASIBILITY_GATE_EXECUTION_RECORD.md`). Nothing in this record
reopens it.

## 2. Research-claim separation

The programme is recorded as three distinct claims.

| Claim | Question | Instrument |
| --- | --- | --- |
| 1 — deterministic mechanism | Does Prefixity deterministically reduce unnecessary carried context while preserving required dependencies, state, and evidence? | Largely offline, on recorded trajectories; does not require a highly capable live model. |
| 2 — downstream efficacy | On a sufficiently competent measuring model, does Prefixity reduce context/input burden without materially reducing task success? | The capable measuring model selected here. |
| 3 — constrained-model benefit | Can Prefixity later let smaller or resource-constrained models operate more effectively or for longer? | Later work; the 0.8B, and potentially 2B/4B, models may return here. They are not the Claim-2 ruler. |

## 3. Evidence informing the decision

Measured on this workstation (existing sealed evidence, Qwen3.5-0.8B Q4_0,
frozen llama.cpp b10217, CPU):

```text
V3 gate (2026-09-25)   prompt 98-154 tok/s   decode 11.6-15.5 tok/s
h001 V2 attempt 002    prompt 24 tok/s       decode 3.5 tok/s
h001 turn-1 prompt     389 tokens
```

The same model and machine differed by about 4x between sessions; the cause
is not established.

Estimates, not measurements (extrapolated from the figures above; decode
scaled by model bytes, prompt processing by parameter count):

| Model (Q4_K_M) | Weights | RAM at 8k context | Decode | Prompt processing |
| --- | ---: | ---: | ---: | ---: |
| Qwen3.5-4B | ~2.7 GB | ~4 GB | ~2.5-3.4 tok/s | ~20-35 tok/s |
| Qwen3.5-9B | ~5.7 GB | ~7-8 GB | ~1.2-1.6 tok/s | ~9-14 tok/s |

Both fit comfortably in 32 GB of system RAM. At the slower session rate,
throughput would be about four times lower. These estimates inform the
decision only; they are not contract values.

## 4. Primary model decision

```text
model      Qwen3.5-9B
quant      Q4_K_M
runtime    local llama.cpp
execution  CPU / system RAM initially
```

4B is not selected as an intermediate Claim-2 gate:

- 0.8B proved too close to the capability floor;
- a 4B-first ladder risks another model-selection sequence rather than a test
  of Prefixity;
- 9B provides materially greater capability headroom while fitting
  comfortably in 32 GB of system RAM;
- CPU execution may be slow, but the competence gate has at most six
  requests;
- the engineering time saved by avoiding a 4B -> 9B ladder outweighs the
  additional inference time.

Qwen3.5-9B uses the same `qwen35` architecture that the frozen b10217
`llama.exe` already loaded, so the runtime executable identity is expected to
carry over and only the model identity is new. It also shares a tokenizer and
chat template with the smaller Qwen3.5 models, which helps a later Claim-3
comparison. 4B remains a useful later constrained-model candidate for
Claim 3.

## 5. Reasoning-mode decision

```text
CLAIM_2_PRIMARY_REASONING_OFF
```

For the primary Claim-2 experiment the measuring model runs with reasoning
off. This is a new explicit design decision for the new instrument; the
earlier reasoning-on scored setting (Option C) was made for the 0.8B runtime
and is not reused.

- Prefixity modifies input/context burden.
- The 0.8B investigation showed that output-side reasoning can dominate token
  use and failure behaviour.
- Claim 2 should first isolate the effect of Prefixity's context intervention
  rather than simultaneously measuring uncontrolled reasoning-output
  behaviour.
- The primary question is whether context reduction preserves successful task
  completion.

This does not claim that reasoning-mode agents are irrelevant. A later
external-validity experiment may test whether the result survives
reasoning-on execution. Only one primary mode is pre-registered; reasoning-on
and reasoning-off variants are not both prepared. The exact runtime
mechanism that disables reasoning (launch flag or chat-template setting) is
deferred to gate preparation, where the frozen b10217 behaviour must be
verified rather than assumed, and is then fixed in the runtime contract.

## 6. Model storage and runtime identity

All new Prefixity model artifacts are stored on D: (C: has limited free
space), under `D:\Prefixity-Lab\models\`, with the proposed path:

```text
D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf
```

The installed Windows `llama.exe` is not moved.

When the model is eventually acquired, in a separately authorized task:

- download once;
- record the exact source repository and snapshot/revision;
- verify the license;
- record the filename, SHA-256, file size, and Windows stable file identity;
- load with explicit `-m <frozen-local-path>` and `--offline`;
- never use `-hf` for the experiment.

The exact quantized artifact and its provider are selected and frozen before
preparation. There is no switching between quantizers (for example
Bartowski, Unsloth, LM Studio) after feasibility begins.

## 7. Single 9B competence gate

```text
LOCAL_9B_SINGLE_GATE_ONLY
maximum live inference requests = 6
```

**Structural component.** The three existing frozen `rbcal` probes, in
order, under the unchanged evaluator: exact required structure; code-fenced
JSON remains a failure; no repair; no retry.

**Task-level component.** Only if all three structural probes pass, one
representative h001 BASELINE trajectory of up to three turns under the
existing deterministic task evaluator. This tests whether the model can do
more than emit small JSON structures.

NO_OP and INTERVENTION are not executed in the competence gate. The gate
tests the ruler, not Prefixity.

## 8. Runtime envelope

```text
reasoning     off
context       8192
max_tokens    1024
temperature   0
top_p         1
seed          1
stream        false
```

No retries, fallback, adaptive replicates, warmup, reasoning-budget ladder,
or output-ceiling ladder. Authoritative token counting
(`/v1/chat/completions/input_tokens`) and the exact context guard precede
every request.

Timeouts are not frozen here. The final contract derives them with the V3
deadline formula from measured or conservatively bounded 9B CPU performance,
not from the 0.8B timeout values. Because the throughput of this workstation
has varied about 4x between sessions (section 3), preparation must record the
execution conditions that affect it (thread count, power plan, background
load) and bound the deadlines conservatively.

## 9. Feasibility stopping rule

The local-9B gate passes only if:

1. all three structural probes PASS; and
2. the h001 BASELINE trajectory completes successfully under its
   deterministic evaluator; and
3. no request is `length`, timeout, context-bound, or otherwise
   inconclusive.

```text
LOCAL_9B_FEASIBILITY_PASSED
```

This permits only the next design/preparation stage (section 11).

Any model-side feasibility failure closes the local 9B path. The response is
not to enable reasoning, change the quantization, raise the ceiling,
introduce a reasoning budget, step down to 4B, or rerun the gate. The next
permitted path is:

```text
CLOUD_GPU_CAPABLE_MODEL_DESIGN_REVIEW
```

**Replacement identity.** Exactly one replacement gate identity is
permitted, and only for a genuine pre-inference infrastructure or integrity
failure:

- `inference_requests` must equal `0`;
- no model output may have been generated;
- once any inference request has been dispatched, that gate identity is
  consumed;
- a model-output FAIL, `length`, timeout after dispatch, structural failure,
  or any other model-side failure does not permit a replacement identity.

No broader remediation is defined.

## 10. Pilot adequacy threshold (deferred)

No BASELINE adequacy threshold is set here.

Principle: before the first scored Claim-2 pilot request, a BASELINE
competence/adequacy threshold must be pre-registered that is sufficiently
demanding to prevent a capability-floor model from making Prefixity quality
preservation vacuous.

The exact threshold is deferred to `PREFIXITY_PILOT_CONTEXT_ADEQUACY_REVIEW`
(section 11), held after local-9B feasibility passes and before scored-pilot
preparation.

## 11. Mandatory pilot-context adequacy review after a pass

```text
PILOT_CONTEXT_ADEQUACY_REVIEW_REQUIRED_AFTER_PASS
```

`LOCAL_9B_FEASIBILITY_PASSED` does not lead directly to the scored pilot.
First, offline:

```text
PREFIXITY_PILOT_CONTEXT_ADEQUACY_REVIEW
```

It determines whether the planned pilot contains enough carried, repeated, or
stale context for Prefixity to have a meaningful intervention opportunity.
The current evidence includes an h001 turn-1 prompt of about 389 tokens, so
meaningful context pressure has not been established; the capability design
already marks context pressure near the usable limit as an unestablished
stratum.

For each proposed case and turn the review quantifies:

- total carried context;
- required context;
- repeated context;
- stale/obsolete context;
- candidate removable/deferable context;
- Prefixity's proposed reduction;
- dependency closure;
- expected percentage and absolute-token reduction.

Tasks are not changed after model outcomes are observed. The pilot workload
must be adequate to test Prefixity, not merely adequate to test the model.

## 12. Context-bound asymmetry rule (preserved)

If a future BASELINE or NO_OP arm becomes context-bound while an INTERVENTION
arm fits because of reduced carried context:

- it is recorded as descriptive mechanism evidence;
- it is not automatically counted as preserved task success;
- an incomplete control arm is not treated as a scored Prefixity win.

This preserves the existing rule
(`PHASE_1C_SCORED_RUNTIME_V3_DESIGN_DECISION.md`, section 5).

## 13. Reuse from V3

Preferred reuse or generalization:

- data/spec-driven gate architecture;
- source-provenance workflow;
- executable freeze and GGUF freeze;
- generated identity and sidecar;
- frozen supervisor and derived deadline formula;
- `/v1/chat/completions/input_tokens` and the exact context guard;
- Windows process exclusivity and post-start ownership;
- evidence sealing and manifests;
- substitution rejection;
- the rbcal evaluator.

Required generalizations include removing hard-coded 0.8B model identity from
reusable request/evidence code: for example, the calibration request record
currently writes the fixed model identifier
`ggml-org/Qwen3.5-0.8B-GGUF:Q4_0`, and the V3 contract validation pins
0.8B-specific values. No new per-attempt validator family is created.

## 14. Not carried forward

- the 0.8B model as the Claim-2 ruler;
- reasoning-budget calibration;
- output-ceiling escalation;
- 0.8B-derived deadlines;
- `-hf` loading;
- hand-written identity fields;
- Attempt-012-style numbering;
- per-attempt validator duplication;
- the assumption that the old reasoning-on decision applies to a new model;
- interpretation of floor-model failures as Prefixity evidence.

## 15. Cloud fallback

No cloud infrastructure is provisioned now. If local 9B fails its
competence gate, the next task is `CLOUD_GPU_CAPABLE_MODEL_DESIGN_REVIEW`.

If local 9B passes but later proves impractically slow for the real pilot,
the two findings are kept separate:

```text
MODEL_CAPABILITY          = ACCEPTED
CPU_RUNTIME_PRACTICALITY  = INADEQUATE
```

In that case the same model may be considered for a pinned rented-GPU runtime
under a separately versioned runtime identity, rather than declaring the model
scientifically unsuitable.

## 16. API and AWS boundary

AWS Bedrock is not used in this phase, and the existing AWS/hackathon
environment is not disturbed. Hosted API models remain a later
external-validity or fallback option, not the primary measuring instrument:
their weights cannot be pinned, and provider-side caching and token counting
confound the input-burden metric Prefixity measures.

## 17. Next permitted task

A separately authorized local-9B feasibility preparation task: select and
freeze the exact artifact (section 6), write the runtime contract with the
envelope of section 8 and derived deadlines, generalize the reusable V3
infrastructure (section 13), and prepare the single gate of section 7. No
download, startup, or inference is authorized by this record.
