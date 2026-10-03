# Claim 2 V2 non-inference tokenization preparation

Status: `CLAIM_2_WORKLOAD_V2_TOKENIZATION_PREPARATION_READY_FOR_OPERATOR_SERVER`.
This is an offline preparation record. No V2 server was started or contacted;
no `/health`, `/input_tokens`, tokenizer, completion, or inference call occurred.
The V1 result remains `WORKLOAD_TOKEN_ADMISSION_FAILED`. The V2 workload is
materialized but its CP07/CP08 token admission is unknown.

## Fixed experiment and provenance

- Accepted starting main and materialization commit:
  `98ef8a1edeb3097b6bdca264ea52fccf7cb14be6`.
- Preparation branch: `codex/phase1c-claim2-v2-tokenization-preparation`.
- Source-provenance commit:
  `94276e930f87b4a38beb46f40892f3fef95078ab`.
- Distinct V2 experiment:
  `claim2-tokenization-v2-fc236cecc0f650ad8678079eaad7000b12d2d7151a2d67e4b84a58183b706138`.
- Canonical identity seal:
  `09c7955969daeb7ef3fec19e4aecd2b38b234f71e40cb7ae21a46e66a0338791`.
- The [identity](PHASE_1C_CLAIM_2_TOKENIZATION_IDENTITY_V2.json)
  binds the source commit and source-file hashes, exact V2 contract/ledger,
  advancing domain, v3 successor, inheritance map, 54 logical identities, 22
  body hashes, 14 inherited count mappings, eight new hashes, both plans,
  V1 seals, accepted runtime and executable, no-inference rule, and pre-contact
  mode-selection rule. Its `.sha256` sidecar seals canonical sorted-key JSON.

The frozen client is
`D:\Users\fleur\Prefixity\target\claim2-v2-tokenization-freeze\94276e930f87b4a38beb46f40892f3fef95078ab\prefixity-phase1c-claim2-v2-tokenization.exe`.
It is a one-time copy of the release build from the source-provenance commit.
SHA-256: `de1a4bd1d185a440568159d75db150fcaa841ae3d8713dcf04c91d770cfe61be`;
size: `3,766,784` bytes; Windows file ID:
`0x00000000000000000004000000459843`. The frozen copy is ignored build
material, while its identity is tracked.

The accepted GGUF is
`D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf`, SHA-256
`cd76ec205963b3b33350093e6904d9de16c4e666fd104e1f632d25c7f15f2a13`.
The accepted llama executable is
`C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe`, build
`b10217-ddd4ec142`, SHA-256
`cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617`.
The final frozen-binary preflight rehashed both local artifacts successfully.

## Reproduced request population

The V2 ledger SHA-256 is
`a539c33355827ef574912ee72e235bf6301bd0acdc6666fa30effebed242a5eb`.
Its order is CP02, CP03, CP07, CP08, CP05, CP06. Every one of the six cases
has BASELINE, NO_OP, and INTERVENTION arms at three slots: 54 unique logical
IDs. Hashing each exact UTF-8 future token-counter body gives 22 unique hashes.
Four retained cases cover 36 logical requests and 14 measured V1 hashes.
CP07/CP08 cover 18 logical requests and eight previously unmeasured hashes;
none collide with a V1 measurement. For each new case, slots 1 and 2 are
shared across all arms, slot-3 BASELINE equals NO_OP, and slot-3 INTERVENTION
is distinct. The existing V2 materialization tests reproduce native histories
and the one selected duplicate-prune difference.

The inheritance-map SHA-256 is
`2d33c1bd253f586d0363620ae14379ee2de49e58429598eff8b5b1c780bd268d`.
The V1 identity seal
`4b7265e8a509829b3109947302ec82c2efec4f05cedd3aeac926324fa2a11f16`,
raw evidence SHA-256
`caf62ccaba1130a0e75d55316a1828cdcb17a8df4cc73f839f96f31a6110c264`,
and interpreted-result seal
`03263b532a13619f9e6e38a447bf984651a712944d7c9aa83df9a86764821040`
verified. The historical ledger and contact plan pins, exact retained V1/V2
body bytes, result counts by hash, model, llama build, reasoning-off/template
semantics, and both pinned Git LF/CRLF physical forms also verified. The
authoritative 14 hash/count/36-ID join is in the V2 identity and inheritance
map. Inherited counts are V1 measurements, not new V2 measurements.

The eight new hashes are:

| Case and equality class | Slot | SHA-256 | UTF-8 bytes |
| --- | ---: | --- | ---: |
| CP07 all arms | 1 | `35755aff459357bc821aafaea115a880a45e0aaf72199d4c8f5b8aebe98bf974` | 14,210 |
| CP07 all arms | 2 | `0f230794e1a0bedeac2c85665f4df6952234e088ca78cea33923c6395d1292ab` | 15,180 |
| CP07 BASELINE/NO_OP | 3 | `7bd750c3e9b851c0f904178600a896a2e9075a81147504d7e7db85a82a559738` | 27,875 |
| CP07 INTERVENTION | 3 | `279a7d4de2aa17af9e3e60406190701efe6a5c64835ec30cf667f3844446d805` | 16,680 |
| CP08 all arms | 1 | `fe9c2b274c0f82bddec9d3a7d0273016bf45cee6aac79d4558877cad6d6ed112` | 15,356 |
| CP08 all arms | 2 | `49c9e6e8b423a53f6d59fb35ecaad4378d74d7c2dd73d06f20b1796d0c15d387` | 16,965 |
| CP08 BASELINE/NO_OP | 3 | `9757637a4ae2c5f997ac288cc1f53d4865f8ff6a0984d4da145280ffdbd2e594` | 29,781 |
| CP08 INTERVENTION | 3 | `c5975a9d340985d322b94a90321ec9c9c74fe772b2ee3eee5b98ea17699c8f90` | 18,550 |

## Frozen plans and selection

The primary [HYBRID_8_NEW plan](../../fixtures/claim2/tokenization-contact-plan-v2-hybrid.json)
has SHA-256 `b694557f04016c04552a5f45cf398de3747fc6fc4eebb7887a17c66dcc951bf3`.
It permits at most one future readiness contact, eight unique input-token
contacts, and zero inference requests. The separate
[FULL_FRESH_22 plan](../../fixtures/claim2/tokenization-contact-plan-v2-full-fresh.json)
has SHA-256 `76333c07b90b1d6d94356e300dd2812532bdb3f01b6beef0dc608ca5c0ea1742`.
It permits at most one future readiness contact, 22 unique input-token
contacts, and zero inference requests. Every entry is sorted by lowercase
SHA-256 and carries exact request bytes, byte length, representative and all
logical IDs, case, slot, and treatment equality class.

Before any network contact, the client validates both frozen plans and the
V2 sources, then tests all 14 inherited V1 hashes, 36 logical mappings,
physical/canonical seals, exact bodies, and historical instrument semantics.
The primary mode is selected only if inheritance is eligible and the current
GGUF/llama hashes match the registered identity. If V1 inheritance is
unverifiable, the choice is FULL_FRESH_22 before contact. The client rejects
a requested mode that differs from that selection. It cannot start with eight
counts and expand to 22. A changed current instrument cannot be used under
this sealed identity; it requires a newly frozen instrument/experiment
identity before any full-fresh contact. Failure of inheritance is an
evidence-reuse issue, not a V2 workload failure.

Production transport exposes only static GET `http://127.0.0.1:8080/health`
and POST `http://127.0.0.1:8080/v1/chat/completions/input_tokens`.
Redirects and proxies are disabled; there is no retry, arbitrary URL,
generation endpoint, warmup, server-start, or server-stop operation. The
client requires the operator-started flag, the exact runtime confirmation,
an empty `LLAMA_ARG_*` environment, a fresh evidence filename, registered
artifact hashes, its own frozen hash/size, and the V2 identity seal before
constructing its HTTP transport. Partial contact evidence is persisted
after each state transition in a later authorized pass.

The frozen executable's offline preflight reported both plans, 54 logical
requests, 22 unique bodies, inheritance eligible, 8 or 22 fresh requests,
zero server contacts, and zero inference. No token count was obtained.

## Future admission calculation, still unknown

For each positive, calculate `D3 = B3 - I3`, `Bsum = B1+B2+B3`,
`Isum = I1+I2+I3`, `Dsum = Bsum-Isum`, `R3 = D3/B3`, and `Rsum = Dsum/Bsum`.
The frozen positive thresholds are `D3 >= 800`, `R3 >= 0.20`, and
`Rsum >= 0.08`. Context gates are `input_tokens <= 6000` and
`input_tokens + 1024 <= 8192`. The two controls require legal reduction zero
and equality of all frozen arms. CP07/CP08 counts, reductions, context fit,
and CPU practicality remain unknown. Their 27,875/29,781-byte slot-3
baseline bodies and 10,976/11,019-byte selected duplicate bodies are byte
sizes, not token measurements or admission evidence. Thresholds were not
changed after seeing the bodies.

## Validation and operator boundary

The complete locked/offline workspace tests, formatting check,
warnings-denied workspace Clippy, and Rust 1.86 locked/offline workspace
check passed. The V2 domain, ledger, and v3 successor were regenerated
without changes. The V1 inheritance map and both V2 plans reproduced
byte-for-byte. Focused tests reject changed body bytes, plan substitution,
generation/arbitrary paths, and invalid runtime/mode CLI arguments. The
frozen-binary preflight verified identity, registered model/llama hashes,
both plans, and zero contacts. CI and exact-SHA promotion are recorded in the
supervisor closeout; they do not authorize the live pass.

Only the human operator may later launch the accepted server. First require
`Get-ChildItem Env:LLAMA_ARG_*` to return nothing. The exact future command,
reported here but **not run**, is:

```powershell
C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe serve -m D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf -c 8192 -np 1 --metrics --reasoning off --offline --host 127.0.0.1 --port 8080
```

After a separately authorized server launch, the future hybrid command is:

```powershell
& 'D:\Users\fleur\Prefixity\target\claim2-v2-tokenization-freeze\94276e930f87b4a38beb46f40892f3fef95078ab\prefixity-phase1c-claim2-v2-tokenization.exe' execute --mode HYBRID_8_NEW --operator-started --model-path 'D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf' --model-sha256 cd76ec205963b3b33350093e6904d9de16c4e666fd104e1f632d25c7f15f2a13 --llama-path 'C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe' --llama-sha256 cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617 --llama-build b10217-ddd4ec142 --context 8192 --slots 1 --reasoning off --offline --host 127.0.0.1 --port 8080 --evidence 'D:\Users\fleur\Prefixity\claim2-v2-hybrid-tokenization-pass-evidence.json'
```

If the pre-contact inheritance check instead selects the full-fresh mode
under this registered runtime identity, the future contingency command is:

```powershell
& 'D:\Users\fleur\Prefixity\target\claim2-v2-tokenization-freeze\94276e930f87b4a38beb46f40892f3fef95078ab\prefixity-phase1c-claim2-v2-tokenization.exe' execute --mode FULL_FRESH_22 --operator-started --model-path 'D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf' --model-sha256 cd76ec205963b3b33350093e6904d9de16c4e666fd104e1f632d25c7f15f2a13 --llama-path 'C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe' --llama-sha256 cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617 --llama-build b10217-ddd4ec142 --context 8192 --slots 1 --reasoning off --offline --host 127.0.0.1 --port 8080 --evidence 'D:\Users\fleur\Prefixity\claim2-v2-full-fresh-tokenization-pass-evidence.json'
```

Neither execution command was run. This preparation stops before operator
server contact.
