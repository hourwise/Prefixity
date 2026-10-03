# Claim 2 V2 non-inference tokenization result

Terminal classification: `WORKLOAD_V2_TOKEN_ADMISSION_FAILED`.
The single authorized `HYBRID_8_NEW` pass completed without an integrity or
HTTP failure. Both new positive cases exceeded their preregistered reduction
thresholds, but the frozen 6,000-input-token context gate failed. The V2
workload, contact plans, experiment identity, and thresholds were not changed.
No completion inference or scored pilot was performed.

## Fixed identities and pre-contact selection

The starting promoted `main`, local `main`, `origin/main`, and GitHub main all
matched `3892a6051de799bd97e05a7e5f382fc3df0296f8`, with a clean tracked
tree. The source-provenance commit was
`94276e930f87b4a38beb46f40892f3fef95078ab`. The sealed experiment was
`claim2-tokenization-v2-fc236cecc0f650ad8678079eaad7000b12d2d7151a2d67e4b84a58183b706138`
(canonical seal
`09c7955969daeb7ef3fec19e4aecd2b38b234f71e40cb7ae21a46e66a0338791`).
The V2 ledger, hybrid plan, full-fresh plan, and frozen executable matched
their preparation hashes. The frozen executable also matched its 3,766,784
byte size and Windows file ID `0x00000000000000000004000000459843`.

The frozen offline preflight and independent plan regeneration confirmed 54
logical requests, 22 unique bodies, 14 sealed V1 count hashes over 36
retained logical requests, eight new hashes over 18 logical requests, exact
retained bytes and hash-to-count joins, V1 identity/raw/result seals and
LF/CRLF physical rules, and the accepted GGUF/llama hashes and runtime
semantics. Thus the immutable pre-contact selection was `HYBRID_8_NEW`.
The full-fresh contingency was not selected or contacted.

The only pre-contact `llama.exe` was PID `12132`, parent PID `2356`, created
`2026-10-03T18:27:03.7898810Z`, at
`C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe`. Its complete
command line was:

```text
"C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe" serve -m D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf -c 8192 -np 1 --metrics --reasoning off --offline --host 127.0.0.1 --port 8080
```

That PID solely owned the `127.0.0.1:8080` listener. There was no competing
llama process or listener, and the client execution environment contained no
`LLAMA_ARG_*` variable. The GGUF SHA-256 was
`cd76ec205963b3b33350093e6904d9de16c4e666fd104e1f632d25c7f15f2a13`;
the llama build was `b10217-ddd4ec142`, SHA-256
`cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617`.

## Raw collection and authoritative counts

The [raw client evidence](../../claim2-v2-hybrid-tokenization-pass-evidence.json)
is preserved byte-for-byte: 15,972 bytes, SHA-256
`169c8d13dcad89b2ba9c20da46d25073d9af37abf1fe34f8358f728498f0a4ab`.
It reports one HTTP 200 readiness contact, exactly eight HTTP 200
`/v1/chat/completions/input_tokens` contacts, and zero inference requests.
Every fresh result is `COUNTED`; the raw status is
`TOKENIZATION_COUNTS_COMPLETE_PENDING_REVIEW`. There was one client
execution, no retry, no additional probe, and no mode switch.

| New exact-body class | Input tokens |
| --- | ---: |
| CP07 slot 1, all arms | 3,415 |
| CP07 slot 2, all arms | 3,784 |
| CP07 slot 3, BASELINE/NO_OP | 6,811 |
| CP07 slot 3, INTERVENTION | 4,290 |
| CP08 slot 1, all arms | 3,913 |
| CP08 slot 2, all arms | 4,495 |
| CP08 slot 3, BASELINE/NO_OP | 7,451 |
| CP08 slot 3, INTERVENTION | 4,999 |

The separate [interpreted result](PHASE_1C_CLAIM_2_TOKENIZATION_RESULT_V2.json)
maps these eight fresh counts and the 14 inherited V1 unique counts by exact
request-body SHA-256 onto all 54 logical IDs. It contains all 22 unique
records, all 54 logical records, case gates, context findings, provenance,
and descriptive CPU arithmetic. The interpreted file SHA-256 is
`136f0600103d3a150497382553e0cfc166bb968f3295ff07f72bc29dc685bd66`;
its canonical JSON seal is
`9c99952a88e90cf8d3a3629758748dcaa1fe240e73f20c6a08826a7ca333f70e`.
The seal sidecar and deterministic result regeneration verified. The raw
evidence was never rewritten during interpretation.

## Frozen admission gates

| Case | Slot-3 baseline / intervention | D3 | R3 | Rsum | Reduction/control | Context | Case admission |
| --- | ---: | ---: | ---: | ---: | --- | --- | --- |
| CP02 | 4,078 / 3,093 | 985 | 24.154% | 17.104% | PASS | PASS | PASS |
| CP03 | 2,488 / 1,590 | 898 | 36.093% | 22.217% | PASS | PASS | PASS |
| CP07 | 6,811 / 4,290 | 2,521 | 37.014% | 17.994% | PASS | FAIL | FAIL |
| CP08 | 7,451 / 4,999 | 2,452 | 32.908% | 15.461% | PASS | FAIL | FAIL |
| CP05 | 3,101 / 3,101 | 0 | 0 | 0 | zero mutation PASS | PASS | PASS |
| CP06 | 3,167 / 3,167 | 0 | 0 | 0 | zero mutation PASS | PASS | PASS |

Each positive independently met `D3 >= 800`, `R3 >= 0.20`, and
`Rsum >= 0.08`; CP05 and CP06 retained BASELINE = NO_OP = INTERVENTION at
each slot and legal reduction zero. CP07 BASELINE and NO_OP slot 3 each had
6,811 input tokens. CP08 BASELINE and NO_OP slot 3 each had 7,451. These four
valid logical requests exceed `input_tokens <= 6000`. The maximum is 7,451
at CP08 BASELINE/NO_OP slot 3; with the reserved 1,024 output tokens it is
8,475, also exceeding the 8,192 context bound. CP07's corresponding
7,835 including reserve remains within 8,192 but still breaches the stricter
6,000-input gate. The observed failure is scientific admission, not an
identity, HTTP, parsing, or evidence-integrity failure. No input was
truncated and no threshold was adjusted.

## CPU planning, shutdown, and boundary

The complete logical input burden is 52,034 BASELINE, 52,034 NO_OP, and
45,178 INTERVENTION tokens, or 149,246 total. At historical rounded 9–10
tokens/s prompt processing, cold-prefill arithmetic is 4.146–4.606 hours.
At assumed 50–150 output tokens per request and historical 1.2–2 tokens/s
decode, input plus decode arithmetic is 4.521–6.481 hours before 18 fresh
starts, tokenization, verification, and orchestration. At the full
1,024-token output ceiling it is 11.826–17.406 hours before those extras.
These are planning scenarios, not measured V2 runtime. The separate
classification is `CPU_RUNTIME_PRACTICALITY_REVIEW_REQUIRED`;
`MODEL_CAPABILITY_ACCEPTED` remains historical and distinct. CPU practicality
does not change token admission.

After the raw hash and interpreted result seal were verified, PID `12132`
was re-identified by PID, parent PID, executable, creation time, exact
command line, and sole port ownership. Only that process was terminated.
Read-only post-shutdown checks returned zero `llama.exe` processes, zero
port-8080 listeners, and zero V2 tokenization clients. No unrelated process
was terminated.

The result publication includes no V2 redesign and no scored-pilot
preparation. The next authorized task is
`CLAIM_2_WORKLOAD_V2_DESIGN_REVIEW`, which is not started here.
