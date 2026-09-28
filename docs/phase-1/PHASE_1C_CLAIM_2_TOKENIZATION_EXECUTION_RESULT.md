# Phase 1C Claim-2 non-inference tokenization execution result

Status: `WORKLOAD_TOKEN_ADMISSION_FAILED`.

This is the single fixed, non-adaptive tokenization pass over the promoted
preparation at `0177b2a7277aeb5cf1c75974f95fcb8259677b7f`. The accepted
experiment identity is
`claim2-tokenization-v1-a5a6b896555db8296318f38010b7120dad8ad191e1329f030e0f738f31b90b91`,
with canonical identity seal
`4b7265e8a509829b3109947302ec82c2efec4f05cedd3aeac926324fa2a11f16`.
No completion inference or scored pilot occurred. The workload is not admitted
because two positive cases miss the frozen absolute saving threshold; fixtures
and thresholds remain unchanged.

## Evidence and contact integrity

The [raw client evidence](../../claim2-tokenization-pass-evidence-v1.json) is
preserved separately from the [interpreted result](PHASE_1C_CLAIM_2_TOKENIZATION_RESULT_V1.json).
Its exact bytes are 14,789 bytes, SHA-256
`caf62ccaba1130a0e75d55316a1828cdcb17a8df4cc73f839f96f31a6110c264`.
The interpreted result has [canonical seal](PHASE_1C_CLAIM_2_TOKENIZATION_RESULT_V1.sha256)
`03263b532a13619f9e6e38a447bf984651a712944d7c9aa83df9a86764821040`.
The latter binds all 22 exact count records, all 54 mapped logical requests,
the source identities, pre-contact server identity, contact accounting,
context and case metrics, CPU planning arithmetic, and terminal result.

Before contact, repository `HEAD`, local `main`, `origin/main`, and GitHub
`main` matched the promoted SHA, and the tracked tree was clean. The identity
and sidecar, exact request ledger
`739205fb56e4f40bd55245f37d0768b8ca73c891b2f8d28bdb8f284e9f811d45`,
contact plan `6b7634a3fc7a3d4b76289aea1772c1df686a6641309acc9e307e5f330dfefdf6`,
materialization successor, and advancing-output-domain hashes matched their
accepted identities. The frozen read-only client remained 3,792,896 bytes,
SHA-256 `113ba4d5c9e7b8e01efa451427679572481a6196fb14aac73863e30d36d96e81`,
file ID `0x00000000000000000006000000459c59`. The accepted GGUF and `llama.exe`
files independently re-hashed to their registered identities.

Exactly one `llama.exe` process, PID `10416`, parent PID `26620`, creation
`2026-09-28T08:07:10.9765990Z`, owned the sole `127.0.0.1:8080` listener.
Its executable path and complete command line matched the accepted
`serve -m ... -c 8192 -np 1 --metrics --reasoning off --offline --host
127.0.0.1 --port 8080` contract. No competing tokenization client or
`LLAMA_ARG_*` environment variable was present, and the evidence target was
fresh. The frozen client's offline preflight passed before its sole `execute`
invocation.

The client recorded one successful readiness contact and 22 successful HTTP
200 `/v1/chat/completions/input_tokens` responses, each parsed as
`response.input_tokens`. It recorded zero inference requests. An independent
audit matched the 22 response hashes and counts to the exact ordered contact
plan, with no missing or duplicate unique hash, and mapped them to all 54
logical requests. The 32 duplicate contacts were avoided. The frozen client
registers only `GET /health` and `POST /v1/chat/completions/input_tokens`; no
generation endpoint is registered. This boundary finding rests on the frozen
interface and recorded contact accounting, not a separate packet capture.

## Authoritative counts and admission

Each triplet below is request slots 1, 2 and 3. A shared `BASELINE / NO_OP`
triplet represents both arms' three logical requests. The interpreted JSON
records every one of the 54 logical request IDs and each of the 22 full hashes.

| Case | BASELINE / NO_OP | INTERVENTION |
| --- | --- | --- |
| CP01 | 987, 1126, 2152 | 987, 1126, 1377 |
| CP02 | 241, 1440, 4078 | 241, 1440, 3093 |
| CP03 | 249, 1305, 2488 | 249, 1305, 1590 |
| CP04 | 995, 1171, 2139 | 995, 1171, 1364 |
| CP05 | 1390, 1608, 3101 | 1390, 1608, 3101 |
| CP06 | 1443, 1655, 3167 | 1443, 1655, 3167 |

Request slots 1 and 2 are exactly equal across all arms. For positive slot 3,
BASELINE equals NO_OP and INTERVENTION has a separate exact body/count.
For both controls, all three arms retain exact body and count equality at
every slot. The largest request is 4,078 tokens at CP02 BASELINE and NO_OP
slot 3. All 54 requests satisfy `input_tokens <= 6000`; the maximum with the
reserved 1,024 output tokens is 5,102, within the 8,192 context.

| Case | B1 | B2 | B3 | I1 | I2 | I3 | D3 = Dsum | Bsum | Isum | R3 | Rsum | Frozen-gate result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| CP01 | 987 | 1126 | 2152 | 987 | 1126 | 1377 | 775 | 4265 | 3490 | 36.013% | 18.171% | FAIL: below 800 tokens |
| CP02 | 241 | 1440 | 4078 | 241 | 1440 | 3093 | 985 | 5759 | 4774 | 24.154% | 17.104% | PASS |
| CP03 | 249 | 1305 | 2488 | 249 | 1305 | 1590 | 898 | 4042 | 3144 | 36.093% | 22.217% | PASS |
| CP04 | 995 | 1171 | 2139 | 995 | 1171 | 1364 | 775 | 4305 | 3530 | 36.232% | 18.002% | FAIL: below 800 tokens |

All four positives exceed the 20% request-3 and 8% cumulative ratio
thresholds, but CP01 and CP04 each save only 775 tokens, 25 below the
preregistered 800-token absolute threshold. CP05 and CP06 each have exactly
zero legal reduction and no mutation. A cohort average or the two passing
positives cannot rescue the two failed cases. This is a complete, valid
experimental-design failure: `WORKLOAD_TOKEN_ADMISSION_FAILED`.

## CPU planning interpretation and shutdown

The 18 logical arms represent 30,735 BASELINE input tokens, 30,735 NO_OP
tokens, and 27,302 INTERVENTION tokens: 88,772 total. At historical rounded
9–10 tokens/s prompt processing, the illustrative cold-prefill burden is
2.466–2.740 hours. Assuming 50–150 output tokens per request and historical
1.2–2 tokens/s decode, input plus decode arithmetic is 2.841–4.615 hours
before 18 fresh starts, tokenization, verification, and orchestration. At the
full 1,024-token output ceiling it is 10.146–15.540 hours before those extras,
exceeding the documented eight-hour planning-review trigger. These are
scenario calculations from measured input counts and historical short-probe
rates, not measured workload runtime. Fresh-start duration, realized output,
and cache behavior remain unknown. The prior instrument classification
`MODEL_CAPABILITY_ACCEPTED` remains separate; this planning classification is
`CPU_RUNTIME_PRACTICALITY_REVIEW_REQUIRED` and does not change admission.

After the raw evidence and result seal were checked, the original PID was
re-identified by parent PID, executable, exact command line, creation time,
and listener ownership, then stopped. A follow-up check found zero llama
processes, zero port-8080 listeners, and zero tokenization client processes.

Validation: independent raw/result SHA-256 and canonical-seal checks passed;
22 unique responses and all 54 mappings matched the frozen plan; integer
threshold comparisons, positive arithmetic, controls, and both context
limits were rechecked. No source, accepted fixture, request ledger, contact
plan, or tokenization identity was altered. CI and exact-SHA promotion remain
the publication gates. The next substantive task is
`CLAIM_2_WORKLOAD_DESIGN_REVIEW`; it is not started here.
