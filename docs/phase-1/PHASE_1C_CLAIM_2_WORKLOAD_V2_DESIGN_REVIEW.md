# Phase 1C Claim-2 workload V2 design review

Status: `CLAIM_2_WORKLOAD_V2_DESIGN_READY` — offline design review accepted by
the supervisor. The next authorized task is
`CLAIM_2_WORKLOAD_V2_MATERIALIZATION_PREPARATION` only.

This record follows the immutable V1 token-admission result
[`PHASE_1C_CLAIM_2_TOKENIZATION_EXECUTION_RESULT.md`](PHASE_1C_CLAIM_2_TOKENIZATION_EXECUTION_RESULT.md)
and the accepted V1 cohort design
[`PHASE_1C_CLAIM_2_CONTEXT_PRESSURE_WORKLOAD_DESIGN.md`](PHASE_1C_CLAIM_2_CONTEXT_PRESSURE_WORKLOAD_DESIGN.md).
It records a V2 design decision. It does not materialize fixtures, establish
new request bodies or token counts, prepare a scored pilot, or authorize
inference.

## Twenty review decisions

1. **V1 outcome stays immutable.** V1 remains
   `WORKLOAD_TOKEN_ADMISSION_FAILED`. Its fixtures, raw evidence, interpreted
   result, identity, hashes, measurements, and frozen thresholds remain
   historical and unchanged. V2 is a new workload version with separate
   artifacts and identity.

2. **CP01 and CP04 fail formally.** Classify each as
   `V1_POSITIVE_FAILED_CONTEXT_PRESSURE_ADMISSION`. Each measured a 775-token
   saving against the fixed 800-token absolute gate. Each misses it by 25
   tokens.

3. **Their ratio gates passed.** CP01 measured `R3 = 36.013%` and
   `Rsum = 18.171%`; CP04 measured `R3 = 36.232%` and `Rsum = 18.002%`.
   Both satisfy the V1 ratio gates of 20% at request 3 and 8% cumulatively.
   The absolute gate still fails independently for each case.

4. **No implementation bug is evident in this result.** The sealed request
   identities, measured counts, case arithmetic, and threshold comparisons
   were independently checked. The current evidence supports a valid
   workload-admission failure; it does not identify a tokenizer or policy
   defect.

5. **Natural body bytes did not guarantee the token gate.**
   CP01's selected attachment body was 2,954 UTF-8 bytes and CP04's was
   3,342 bytes. Those removals corresponded to whole-request tokenizer
   deltas of 775 tokens each. Both bodies fell within the original rough byte
   range, yet below its expected 1,000–1,400-token range. The byte estimate
   was not a Qwen tokenizer measurement or an admission guarantee.

6. **The attachment-only token effect is unknown.** The 775-token deltas
   compare complete rendered requests. Tokenization across the surrounding
   prompt and template boundaries means the attachment contribution cannot
   be isolated from those measurements. Do not report 775 as a standalone
   attachment token count.

7. **Retain CP02 and CP03 as V1 evidence.** CP02 saved 985 tokens and CP03
   saved 898; both passed the V1 absolute and ratio gates. They remain
   eligible candidates for reuse in V2 under the exact evidence-reuse rule
   below. Their old measurements remain labeled V1 measurements.

8. **Retain CP05 and CP06 as unchanged controls.** Both had zero legal
   reduction and equal request bodies and counts across arms. V2 keeps them
   as zero-mutation dependency controls and does not treat their equality as
   positive reduction.

9. **Do not repair V1 after seeing its result.** No post-hoc attachment
   patch, threshold relaxation, case exclusion, or fixture mutation is
   allowed. CP01 and CP04 remain failed V1 positives even if future V2 cases
   pass.

10. **Keep the case families distinct.** CP02 is repository symbol/path
    investigation and CP03 is build/dependency metadata reconciliation. The
    proposed CP07 benefits workflow and CP08 cold-chain workflow below have
    different evidence, rules, state, and deterministic answer schemas from
    one another and from CP02/CP03.

11. **CP07 is a proposed benefits/coverage adjudication family.** Use a
    controlled, synthetic, deterministic case-management workflow. A
    complete, versioned plan/policy document is attached at the start. The
    proposed ordinary final adjudication packet reattaches that exact document
    after two read-only checks of an unchanged case-source snapshot. The
    final hidden-key evaluation requires
    exact JSON containing coverage status, governing clause IDs,
    deductible/limit arithmetic, and the applicable exclusion.

12. **CP08 is a proposed cold-chain lot-disposition family.** Use a
    controlled, synthetic, deterministic quality workflow with a complete,
    versioned SOP. The proposed ordinary final quality-review packet
    reattaches the SOP unchanged after read-only logger and calibration
    verification of an unchanged lot-source snapshot. The final hidden-key
    evaluation requires
    exact JSON containing release/hold/escalate, governing rule IDs,
    time-temperature and cumulative-exposure calculations, and calibration
    validity.

13. **Both proposed positives must be genuine native reattachments.** In
    each case, the repeated object must be a non-protocol native `Message`,
    byte-identical to its original, tied to the same world-state revision,
    with the original retained. The later copy must have no consumer and no
    protected relation. Preserve source occurrence IDs, provenance, and the
    workflow record proving ordinary reattachment before any token
    inspection.

14. **Keep the evidence and policy contract closed.** Do not relabel a tool
    result, assistant response, receipt, or protocol event as an attachment.
    Materialize closed-world dependency graphs, isolate hidden answer keys,
    and use deterministic source generators and evaluators. Add no padding,
    filler, artificial repetition, or synthetic workflow step whose only
    purpose is to create an eligible duplicate.

15. **Apply the existing blinded policy unchanged.**
    `controlled-evidence-policy-v1` must select the sole eligible latest
    duplicate using `EXACT_DUPLICATE_PRUNE` and its existing sequence rule;
    selection never ranks by size. No new rule, policy order, target
    cardinality, or selection semantics are authorized. A case is rejected
    at materialization if eligibility requires a fabricated reattachment or
    policy change. If the workflow semantics cannot fit the existing
    predicate, return `MECHANISM_CHANGE_REQUIRED`.

16. **Freeze the proposed V2 roster and trajectory ceiling.** The
    case-major roster is exactly `CP02, CP03, CP07, CP08, CP05, CP06`:
    four positives followed by two controls. Each case has three sequential
    request slots and three independently generated arms
    (`BASELINE`, `NO_OP`, `INTERVENTION`). That is 54 maximum inference
    requests, solely as a ceiling for separately authorized future scored
    work. This design review authorizes none of those requests.

17. **Keep V2 admission criteria fixed.** Every positive must satisfy
    `D3 >= 800`, `R3 >= 0.20`, and `Rsum >= 0.08`; every rendered input must
    be at most 6,000 tokens and satisfy `input_tokens + 1,024 <= 8,192`.
    Both controls must preserve exact body/count equality and have zero
    mutation. Do not relax V1 thresholds or patch CP01/CP04 after observing
    counts.

18. **Reuse V1 counts only under conditional decision A.** The 14 sealed V1
    unique count hashes represented by the 36 logical requests for retained
    cases CP02, CP03, CP05, and CP06 may be inherited only when each V2
    request has identical exact UTF-8 body bytes and the GGUF/tokenizer,
    llama build, reasoning-off setting, template behavior (including absent
    `chat_template_kwargs`), endpoint, and other relevant runtime settings
    match. Bind the inherited evidence to V1 identity seal
    `4b7265e8a509829b3109947302ec82c2efec4f05cedd3aeac926324fa2a11f16`,
    raw evidence SHA-256
    `caf62ccaba1130a0e75d55316a1828cdcb17a8df4cc73f839f96f31a6110c264`,
    and interpreted-result seal
    `03263b532a13619f9e6e38a447bf984651a712944d7c9aa83df9a86764821040`.
    Label these as inherited V1 measurements, not new V2 contacts. V1 itself
    remains failed.

19. **Recount changed bodies without forced expectations.** Compute exact
    V2 request hashes only after offline materialization; do not force a
    presumed unique-body count. A V2 body that differs from all sealed V1
    bodies needs a new input-token count. An identical body may inherit its
    count only from another verified sealed measurement under the same
    runtime conditions. If a V1 seal cannot be verified or the counting
    instrument/runtime has drifted, perform a complete fresh V2
    tokenization pass. No count or inference runs in the present task.

20. **Use staged gates and stop at design.** The sequence is: accepted
    design; offline materialization and supervisor review; exact non-inference
    token admission and review; then only separately authorized scored-pilot
    preparation. The immediate next task is
    `CLAIM_2_WORKLOAD_V2_MATERIALIZATION_PREPARATION`, not a tokenization or
    inference task. The design alone changes neither model-capability status
    nor CPU practicality status.

## Evidence, estimates, and remaining questions

The authoritative V1 result measured 88,772 logical input tokens across its
54 requests. The retained V1 cases CP02, CP03, CP05, and CP06 account for
64,612 of those measured logical input tokens. Reuse of that total requires
their exact bodies and counting instrument to remain unchanged. The 18
logical requests for new V2 cases CP07 and CP08 have no measured token
counts. If all 18 later meet the 6,000-token input gate,
`64,612 + 18 × 6,000 = 172,612` is a
conditional upper-bound arithmetic scenario, not a measured V2 total or a
prediction of realized work.

At historical 9–10 tokens/second prompt processing, 172,612 input tokens
would imply approximately 4.80–5.33 hours of prefill arithmetic. At the full
1,024-token output ceiling for 54 requests, 55,296 output tokens at the
historical 1.2–2 tokens/second decode rate imply 7.68–12.80 hours of decode
arithmetic alone. These scenarios exclude fresh starts and other overhead;
they are not runtime measurements or forecasts. Keep the previous
`MODEL_CAPABILITY_ACCEPTED` instrument classification separate from
`CPU_RUNTIME_PRACTICALITY_REVIEW_REQUIRED`.

The CP07/CP08 naturalness, native reattachment, eligibility, and useful size
margin are design hypotheses. Materialization must establish them with
workflow provenance and exact bytes before any tokenizer inspection. No
estimated size demonstrates a token margin or admission. If the ordinary
workflow does not create an eligible duplicate, reject the case; do not
fabricate one to satisfy the hypothesis.

## Required work in the next task

`CLAIM_2_WORKLOAD_V2_MATERIALIZATION_PREPARATION` should add a versioned V2
authoring contract alongside the V1 restrictions in
[`fixtures/claim2/README.md`](../../fixtures/claim2/README.md), which currently
permit only CP01–CP06. Add the CP07/CP08 source assets,
manifests, finite action menus, receipts, hidden evaluation keys, source
generators, deterministic evaluators, and tests. Add canonical advancing
responses and a finite-domain successor for all V2 cases, while preserving
the V1 fixtures and artifacts. Produce offline rendered-request evidence for
the exact 54 V2 logical requests. A later, separately authorized tokenization
preparation should derive the V2 deduplication/contact plan and separate V2
identity from those accepted bytes. Verify policy parity, provenance, byte
identity, dependency closure, eligibility, controls, and deterministic
regeneration offline. If any
materialization requirement can be met only by invented repetition or a
policy/mechanism change, stop and report the prescribed rejection state.

The work in this review was documentation only. No fixture, code, ledger,
tokenization plan, evidence, threshold, or identity was changed; no server,
tokenizer, or inference endpoint was contacted; and no V2 inference was run.

`CLAIM_2_WORKLOAD_V2_DESIGN_READY`
