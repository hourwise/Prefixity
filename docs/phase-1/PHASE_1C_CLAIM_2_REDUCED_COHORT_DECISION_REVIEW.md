# Claim 2 reduced-cohort scored-pilot decision review

Status: `CLAIM_2_REDUCED_COHORT_SCORED_PILOT_DESIGN_READY`.
Starting accepted `main`: `4ccc0927b2f3080b096e802c391176b99740ed43`.
This is the authorized `CLAIM_2_WORKLOAD_V3_DESIGN_REVIEW`, resolved as a
documentation-only decision. The next task is only the separate
`CLAIM_2_REDUCED_COHORT_SCORED_PILOT_PREPARATION`. No scored pilot, client,
server, tokenization, or inference is prepared or executed here.

## Decision and historical evidence

Stop seeking two more synthetic positive cases merely to fill the original
four-positive Claim-2 token-admitted cohort in this phase. Preserve the three
complete, sealed admission studies as calibration evidence. Advance only a
separately labelled **reduced-cohort downstream-efficacy feasibility pilot**
design using CP02, CP03, CP05, and CP06. The study label is
`CLAIM_2_REDUCED_COHORT_DOWNSTREAM_EFFICACY_FEASIBILITY_PILOT`.
This is a decision to study a narrower
question, not a reclassification or rescue of the failed cohorts.

| Study | Immutable terminal state | Replacement-positive observation | Interpretation |
| --- | --- | --- | --- |
| [V1](PHASE_1C_CLAIM_2_TOKENIZATION_EXECUTION_RESULT.md) | `WORKLOAD_TOKEN_ADMISSION_FAILED` | CP01 and CP04 each saved `D3 = 775`, 25 below 800; ratio and context gates passed. | Too little absolute removable mass in these two cases. |
| [V2](PHASE_1C_CLAIM_2_WORKLOAD_V2_TOKENIZATION_EXECUTION_RESULT.md) | `WORKLOAD_V2_TOKEN_ADMISSION_FAILED` | CP07/08 saved 2,521/2,452 tokens and passed ratios; baseline slot 3 reached 6,811/7,451. | Ample reduction, but complete baseline context exceeded 6,000; CP08 also exceeded 8,192 with reserve. |
| [V3](PHASE_1C_CLAIM_2_WORKLOAD_V3_TOKENIZATION_EXECUTION_RESULT.md) | `WORKLOAD_V3_TOKEN_ADMISSION_FAILED` | CP09 saved 2,317 with passing ratios but baseline slot 3 reached 7,569, or 8,593 with reserve. CP10 fitted context and saved 996 with `Rsum = 8.849%`, but `R3 = 19.325%`. | One context failure and one isolated request-3 ratio failure. |

These are complete, valid tokenization outcomes, not incomplete collections.
All three studies used exact-body mappings and their frozen gates. The
[earlier V3 design review](PHASE_1C_CLAIM_2_WORKLOAD_V3_DESIGN_REVIEW.md)
records CP01/04/07/08's selected legal targets; the
[V3 materialization](PHASE_1C_CLAIM_2_WORKLOAD_V3_MATERIALIZATION.md)
records CP09/10's sole `EXACT_DUPLICATE_PRUNE` targets, same-state original
retention, zero consumer/protected edges on the later occurrences, closed
dependencies, hidden-key isolation, and passing offline arm replays. CP05
and CP06 remained exact no-op controls, including CP06's protected-consumer
case. The available evidence therefore does **not indicate a defect in the
frozen selection or dependency mechanism**. It does not demonstrate that a
model will preserve task success after the deletion; no CP02/CP03 scored
BASELINE/NO_OP/INTERVENTION model result exists.

The cycles calibrated different edges of the admission envelope rather than
answering downstream efficacy. A CP11/CP12 design, fixture, generator,
evaluator, CI, frozen-client preparation, operator tokenization, evidence
interpretation, and publication cycle would primarily test whether two more
synthetic workloads can be engineered inside that envelope. Its possible
information about model task preservation remains indirect. The programme
cost and CPU burden make another such cycle disproportionate **before any
CP02/CP03 downstream score is known**. Resource proportionality supports
the sequencing decision; it is not evidence of efficacy or of mechanism
correctness. A later broader study can be proposed on its own evidence and
budget, without altering these results.

## Frozen reduced cohort and scope of inference

The new cohort is fixed in this case-major order:

| Order | Case | Role | Existing admission |
| ---: | --- | --- | --- |
| 1 | CP02 | repository symbol-search positive | PASS: `D3 = 985`, `R3 = 24.154%`, `Rsum = 17.104%`, context PASS |
| 2 | CP03 | build-metadata reconciliation positive | PASS: `D3 = 898`, `R3 = 36.093%`, `Rsum = 22.217%`, context PASS |
| 3 | CP05 | different-state zero-mutation control | PASS: exact corresponding arm equality, legal reduction 0, context PASS |
| 4 | CP06 | consumer-protected zero-mutation control | PASS: exact corresponding arm equality, legal reduction 0, context PASS |

Each case has BASELINE, NO_OP, and INTERVENTION arms, with three sequential
requests per arm: **4 cases × 3 arms × 3 requests = 36 future inference
requests maximum**. This ceiling is a design bound, not authorization to make
any request. The existing dependency-safety battery remains separate.

Selecting CP02/03 after three token-admission studies creates a selection
effect: they are examples known to fit the token/reduction envelope, so the
pilot cannot estimate success rates for arbitrary positives or the originally
proposed four-positive cohort. It does **not** select on observed Qwen task
success, BASELINE/INTERVENTION inference, or scored differences, because
those outcomes have not been observed for these cases. A separately
preregistered, transparently selected two-positive feasibility study is
therefore defensible. Its preparation must freeze scoring, completeness,
BASELINE competence handling, order, and stop rules before any inference;
failure or inconclusiveness must remain reportable.

The intended narrow claim is: **this reduced-cohort pilot tests whether
Prefixity's frozen context-reduction mechanism can preserve downstream task
success on two previously token-admitted positive workloads while leaving
two independently validated no-mutation controls unchanged.** Even a
positive result would not establish broad workload-family generalization,
performance on CP01/04/07/08/09/10, production deployment efficacy, a
constrained-model advantage, or statistical population-level efficacy. It
would not be the original four-positive Claim-2 efficacy proof.

No failed positive is promoted, repaired, truncated, or substituted. The
existing gates remain exactly `D3 >= 800`, `R3 >= 0.20`, `Rsum >= 0.08`,
`input_tokens <= 6000`, and `input_tokens + 1024 <= 8192`. The experiment
scope narrows under a new label; no threshold is retroactively tuned.

## Reuse, burden, and future boundary

The [sealed V1 result](PHASE_1C_CLAIM_2_TOKENIZATION_RESULT_V1.json) and
[V3 exact-body mapping](PHASE_1C_CLAIM_2_TOKENIZATION_RESULT_V3.json)
contain all 14 unique counts needed for these 36 logical requests. Direct
summation of the V3 logical mappings gives 22,165 BASELINE, 22,165 NO_OP,
and 20,282 INTERVENTION input tokens: **64,612** total. By case, the
three-arm totals are CP02 16,292; CP03 11,228; CP05 18,297; and CP06
18,795. This is measured-token planning arithmetic, not model execution.

No new `/input_tokens` pass is expected **if** preparation revalidates every
future request's exact UTF-8 body SHA-256 against the 14 V1 hashes and the
V1 identity (`4b7265e8a509829b3109947302ec82c2efec4f05cedd3aeac926324fa2a11f16`),
raw SHA-256 (`caf62ccaba1130a0e75d55316a1828cdcb17a8df4cc73f839f96f31a6110c264`),
and result seal (`03263b532a13619f9e6e38a447bf984651a712944d7c9aa83df9a86764821040`).
It must also match the registered GGUF, llama build/tokenizer, reasoning-off
and template semantics, endpoint behavior, context 8192, and 1,024-token
output reserve. Evidence stays labelled *V1 measurements inherited into the
reduced pilot*. Body or instrument drift invalidates reuse; preparation
must stop for a separately sealed identity and authorization rather than
copy counts or silently tokenize.

At historical rounded 9–10 input tokens/s, the 64,612 inputs imply
approximately 1.795–1.994 hours of cold prefill. Assumed 50–150 output
tokens per request at historical 1.2–2 output tokens/s give approximately
2.045–3.244 hours of combined arithmetic before startup and verification;
the 1,024-token output ceiling gives approximately 6.915–10.527 hours.
Twelve fresh arm starts, realized output, and this workload's actual runtime
remain unmeasured. `CPU_RUNTIME_PRACTICALITY_REVIEW_REQUIRED` remains
separate from historical `MODEL_CAPABILITY_ACCEPTED`. These estimates call
for bounded execution planning; they do not change token admission or,
alone, preclude a feasibility pilot.

The only next authorized task is
`CLAIM_2_REDUCED_COHORT_SCORED_PILOT_PREPARATION`. It may build a new
experiment identity, bind exact existing fixtures and sealed counts,
preregister scoring, order, safety and stop rules, prepare a frozen harness
if needed, and refine the execution estimate. It must stop before any
operator-started inference. This review changes no fixture, generator,
workload contract, mechanism code, historical evidence, or threshold and
makes **zero** server contacts, tokenization calls, and inference calls.
