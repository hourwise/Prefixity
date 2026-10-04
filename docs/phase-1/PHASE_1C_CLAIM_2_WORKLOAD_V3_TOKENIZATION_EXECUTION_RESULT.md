# Claim 2 V3 non-inference tokenization result

Terminal classification: `WORKLOAD_V3_TOKEN_ADMISSION_FAILED`. The one
authorized `HYBRID_8_NEW` pass completed with valid collection integrity.
CP09 failed the frozen input/context gates, and CP10 failed the frozen
request-3 20% reduction gate. This is an observed workload result, not an
HTTP, parser, identity, or evidence failure. No inference, redesign, scored
pilot, threshold change, or second tokenization pass occurred.

## Identity and pre-contact selection

Before contact, HEAD, local `main`, `origin/main`, and GitHub `main` were all
`3653035dbb85548569fd124f3feb3f0efae721de`, with a clean tracked tree.
The V3 source-provenance commit was
`b0cc0e66544b5428a6c67a4e876d6b7302031675`. The distinct experiment
was `claim2-tokenization-v3-570cc81cbc7ddbba5c3e3128571f1197b79704b0e7af3868e6cecc1db443960f`
with canonical identity seal
`6dfca31b408b780e5aa318fdd1caa896bda6f5f91b72176555edc69255e51d01`.
The contract, ledger, hybrid plan, and full-fresh plan matched their frozen
SHA-256 values. The [preparation record](PHASE_1C_CLAIM_2_WORKLOAD_V3_TOKENIZATION_PREPARATION.md)
records those identities and exact client provenance.

The frozen client at
`D:\Users\fleur\Prefixity\target\claim2-v3-tokenization-freeze\b0cc0e66544b5428a6c67a4e876d6b7302031675\prefixity-phase1c-claim2-v3-tokenization.exe`
matched SHA-256
`8d2a53516120f7f94d3c249bd354be443231a54fdcea4066566d0efb87906d91`,
4,984,832 bytes, and Windows file ID
`0x0000000000000000000200000045d93a`. The registered GGUF matched
`cd76ec205963b3b33350093e6904d9de16c4e666fd104e1f632d25c7f15f2a13`;
the llama build `b10217-ddd4ec142` matched
`cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617`.

The operator-started server was the only relevant `llama.exe`: PID `20260`,
parent PID `2356`, created 2026-10-04 08:39:11 local time. It was the sole
owner of `127.0.0.1:8080`. Its executable was
`C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe`, with this
complete command line:

```text
"C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe" serve -m D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf -c 8192 -np 1 --metrics --reasoning off --offline --host 127.0.0.1 --port 8080
```

There were no `LLAMA_ARG_*` variables in the client environment. Offline
preflight regenerated the complete 54-request/22-body ledger and both
plans, verified all 14 V1 unique measurements over 36 exact retained
requests, V1 identity/raw/result seals, source forms, and the matching
instrument. It found eight new bodies over 18 CP09/CP10 requests. Thus
inheritance was eligible and the immutable pre-contact selection was
`HYBRID_8_NEW`; `FULL_FRESH_22` was not contacted.

## Raw collection and mapping

The [raw client evidence](../../claim2-v3-hybrid-tokenization-pass-evidence.json)
is preserved byte-for-byte: **10,742 bytes**, SHA-256
`b081e435982e9354d0b72a16964c0f5e62b2b5beb25383be0553b9647b21ee83`.
It records one HTTP 200 readiness contact, exactly eight HTTP 200
`/v1/chat/completions/input_tokens` contacts, and zero inference requests.
Every result is `COUNTED`; the client status is
`TOKENIZATION_COUNTS_COMPLETE_PENDING_REVIEW`. There was no retry, manual
probe, or mode switch. The separate [interpreted result](PHASE_1C_CLAIM_2_TOKENIZATION_RESULT_V3.json)
contains the full 22-unique-body and 54-logical-request hash/count mappings.
It joins exclusively by exact frozen request-body SHA-256: 14 unique V1
measurements across 36 logical requests and eight fresh V3 measurements
across 18. Its physical SHA-256 is
`cc78daf802403bfb799c8e75578ab8d19dee158eb07ad3f71ac974b7c51f5c2d`;
its canonical JSON seal is
`eff55318796fc345aa52075c4425918674d58cf1b36ac44130f58322d21435ad`.
The [result generator](../../scripts/claim2_v3_tokenization_result.py)
reproduces the result and sidecar without rewriting the raw evidence.

| New exact-body class | Input tokens |
| --- | ---: |
| CP09 slot 1, all arms | 4,400 |
| CP09 slot 2, all arms | 4,876 |
| CP09 slot 3, BASELINE/NO_OP | 7,569 |
| CP09 slot 3, INTERVENTION | 5,252 |
| CP10 slot 1, all arms | 2,754 |
| CP10 slot 2, all arms | 3,347 |
| CP10 slot 3, BASELINE/NO_OP | 5,154 |
| CP10 slot 3, INTERVENTION | 4,158 |

## Frozen gates and CPU arithmetic

| Case | Baseline slots 1/2/3 | Intervention slot 3 | D3 | R3 | Rsum | Reduction/control | Context | Admission |
| --- | --- | ---: | ---: | ---: | ---: | --- | --- | --- |
| CP02 | 241 / 1,440 / 4,078 | 3,093 | 985 | 24.154% | 17.104% | PASS | PASS | PASS |
| CP03 | 249 / 1,305 / 2,488 | 1,590 | 898 | 36.093% | 22.217% | PASS | PASS | PASS |
| CP09 | 4,400 / 4,876 / 7,569 | 5,252 | 2,317 | 30.612% | 13.755% | PASS | FAIL | FAIL |
| CP10 | 2,754 / 3,347 / 5,154 | 4,158 | 996 | 19.325% | 8.849% | FAIL: R3 < 20% | PASS | FAIL |
| CP05 | 1,390 / 1,608 / 3,101 | 3,101 | 0 | 0% | 0% | zero mutation PASS | PASS | PASS |
| CP06 | 1,443 / 1,655 / 3,167 | 3,167 | 0 | 0% | 0% | zero mutation PASS | PASS | PASS |

CP09 BASELINE and NO_OP slot 3 each exceed the 6,000-input-token limit.
Their 7,569 input tokens plus the frozen 1,024 output reserve total **8,593**,
also above the 8,192 context bound. This is the maximum input request.
CP10 passes `D3 >= 800` and `Rsum >= 0.08`, but its `R3 = 996/5154`
is below `0.20`. CP02 and CP03 independently pass all gates. CP05 and
CP06 retain BASELINE = NO_OP = INTERVENTION at every slot, with legal
reduction zero. No request was truncated and no threshold was changed.

The complete logical input totals are **50,265 BASELINE**, **50,265 NO_OP**,
and **45,069 INTERVENTION**, or **145,599** total. Historical rounded
9-10 tokens/s prefill gives descriptive arithmetic of 4.044-4.494 hours;
50-150 assumed output tokens per request plus historical decode rates gives
4.419-6.369 hours before startup and other overhead. These are planning
estimates, not measured V3 runtime. The separate CPU classification is
`CPU_RUNTIME_PRACTICALITY_REVIEW_REQUIRED`; historical
`MODEL_CAPABILITY_ACCEPTED` remains distinct. CPU convenience did not
affect token admission.

## Shutdown, validation, and boundary

After raw evidence hashing and interpreted-result seal verification, PID
`20260` was re-identified by PID, parent PID, creation time, executable,
complete command line, and sole port ownership. Only that server was
terminated. Post-shutdown checks found **zero** relevant `llama.exe`
processes, **zero** port-8080 listeners, and **zero** V3 clients.

Deterministic regeneration, independent raw/hash/join/gate arithmetic,
workspace checks, and exact candidate CI validate publication. The
supervisor closeout records the candidate and post-promotion CI runs and
exact promoted result SHA. The observed failure remains immutable in this
task. The only next authorized task is the separate
`CLAIM_2_WORKLOAD_V3_DESIGN_REVIEW`; it is not started here.
