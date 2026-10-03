# Claim 2 V3 non-inference tokenization preparation

Status: `CLAIM_2_WORKLOAD_V3_TOKENIZATION_PREPARATION_READY_FOR_OPERATOR_SERVER`.
This is an offline preparation record. No server was started or contacted, no
token count was requested, and no inference occurred. V1 remains
`WORKLOAD_TOKEN_ADMISSION_FAILED`; V2 remains
`WORKLOAD_V2_TOKEN_ADMISSION_FAILED`. CP09/CP10 admission is unknown.

## Source and experiment identity

- Accepted starting and V3 materialization `main`:
  `64d732dad2448e768ce12bc13a15bf3081f63205`.
- Preparation branch: `codex/phase1c-claim2-v3-tokenization-preparation`.
- Final source-provenance commit:
  `d7d6af170748e570e03326835916266efc90b531`. Earlier source commits
  were superseded before publication to normalize Git LF/CRLF source forms
  and bind the frozen Windows file ID;
  neither changed any historical evidence or the V3 materialization.
- Distinct experiment ID:
  `claim2-tokenization-v3-5ef8665ed781aa967d2f56ba2a78e74761827d86c7e6a16547835d8417bfbc84`.
- [V3 identity](PHASE_1C_CLAIM_2_TOKENIZATION_IDENTITY_V3.json) canonical
  SHA-256 seal:
  `da04c940b6bdb1e9e3d033f481e725803cab3771d639cf6328971bc2aa8c1cf9`.
  The `.sha256` sidecar and [offline identity builder](../../scripts/claim2_v3_tokenization_identity.py)
  reproduce it. It binds the 54 logical identities, 22 unique body hashes,
  14 V1 mappings, eight new hashes, both plans, accepted artifacts and
  instrument, current source and executable, zero inference, and immutable
  pre-contact mode choice.

The final frozen client is
`D:\Users\fleur\Prefixity\target\claim2-v3-tokenization-freeze\d7d6af170748e570e03326835916266efc90b531\prefixity-phase1c-claim2-v3-tokenization.exe`.
It is a one-time copy of the release build from that source commit. SHA-256:
`fd351fdd32845dddc1922a817ea1528a1acf384c1dfffcd4a05c79796e6e9eff`;
size: `4,984,832` bytes; Windows file ID:
`0x0000000000000000000700000045d923` (volume/index
`volume=ba2f80f4;index=000700000045d923`). The copy is ignored build material;
its identity is tracked. The accepted GGUF SHA-256 is
`cd76ec205963b3b33350093e6904d9de16c4e666fd104e1f632d25c7f15f2a13`;
the accepted llama build `b10217-ddd4ec142` SHA-256 is
`cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617`.
Both current external files were hashed and matched. No llama process was
launched or contacted.

## Frozen request population and plans

The V3 contract is `prefixity.phase1c.claim2-workload-authoring-contract`
version 3, SHA-256
`28d48448719433a9baa28c0b668cafade63c7858655be00a1fd9effbb638ac5e`.
The [request ledger](../../fixtures/claim2/workload-request-ledger-v3.json)
SHA-256 is
`406586d71839d91bc565b9db5da69a3d4152193c4109f7876e6a1dc56d89dce2`.
Regenerating its accepted renderer yields exactly the same bytes. Six cases in
order CP02/CP03/CP09/CP10/CP05/CP06, three arms, and three request slots
produce 54 logical requests and 22 unique exact bodies. The [V3 inheritance
map](../../fixtures/claim2/token-evidence-inheritance-map-v3.json) verifies
14 sealed V1 hashes covering all 36 retained CP02/03/05/06 requests. The V1
identity, raw, and result seals remain respectively
`4b7265e8a509829b3109947302ec82c2efec4f05cedd3aeac926324fa2a11f16`,
`caf62ccaba1130a0e75d55316a1828cdcb17a8df4cc73f839f96f31a6110c264`,
and `03263b532a13619f9e6e38a447bf984651a712944d7c9aa83df9a86764821040`.
The retained values are labelled **V1 measurements inherited into V3**,
conditional on exact body and instrument semantics. No V3 contact is imputed.

CP09 and CP10 contribute 18 logical requests and eight new unique hashes,
four per case. Each has one all-arm body at slots 1 and 2, one shared
BASELINE/NO_OP body at slot 3, and one INTERVENTION-only slot-3 body. None
collide with V1. Canonical prior outputs and receipts reproduce; hidden
evaluator answers are outside the request bodies. The intervention slot-3
body differs by the frozen one-target `EXACT_DUPLICATE_PRUNE` treatment.

The primary [HYBRID_8_NEW plan](../../fixtures/claim2/tokenization-contact-plan-v3-hybrid.json)
SHA-256 is
`7897b44d1c6e3c43d387f760fe4994ff5ee94ca1d258e86a3bc2c91b6033ce7e`.
It allows one future readiness and eight unique input-token contacts. The
separate [FULL_FRESH_22 contingency](../../fixtures/claim2/tokenization-contact-plan-v3-full-fresh.json)
SHA-256 is
`acb927d97df1a00f2e9a33d4fb7569ba1fc3eca89a0bd720042a28337b06fe5e`.
It allows one future readiness and 22 unique input-token contacts. Both have
zero inference allowance. Each sorted entry binds exact body bytes, hash,
length, representative and all logical IDs, case, slot, and arm equality
class. The plan builder regenerates the accepted V3 ledger through its
renderer and requires exact frozen ledger/map hashes before emitting a plan.

The client selects exactly one plan **before first contact**. Verified V1
inheritance under the registered matching instrument selects HYBRID_8_NEW;
unverifiable inheritance with that same matching instrument selects
FULL_FRESH_22. Instrument drift aborts and requires a new identity. A
requested mode contrary to the pre-contact selection fails. There is no
adaptive expansion after contact.

Only static GET `http://127.0.0.1:8080/health` and POST
`http://127.0.0.1:8080/v1/chat/completions/input_tokens` are available in
production transport. The client has no generation endpoint, arbitrary URL,
warmup, retry, redirect, proxy, server-start, or server-stop capability.
Before constructing transport it regenerates the V3 domain, ledger, and
successor, validates the plans, V1 inheritance, sealed experiment, exact
source/executable and GGUF/llama identities, operator confirmation, empty
`LLAMA_ARG_*` environment, and a fresh evidence path.

## Offline validation and future calculation

The final frozen executable's preflight reported HYBRID with 54 logical
requests, 22 unique bodies, 14 inherited counts, eight fresh bodies, and
FULL_FRESH with 54 logical requests, 22 fresh bodies. Both reported zero
server contacts and zero inference. The identity builder reproduced the
tracked identity/sidecar; both plan files reproduced byte-for-byte. Full
locked/offline workspace tests, formatting, warnings-denied all-features
Clippy, Rust 1.86 locked/offline check, materialization regression tests,
endpoint/path/mode/runtime denial, body and plan tamper rejection, frozen
executable substitution, and source text-form tests passed. CI and promotion
are recorded in the supervisor closeout.

Future CP09/CP10 arithmetic is preregistered but unevaluated:
`D3 = B3 - I3`, `Bsum = B1+B2+B3`, `Isum = I1+I2+I3`,
`Dsum = Bsum-Isum`, `R3 = D3/B3`, `Rsum = Dsum/Bsum`.
Positive gates remain `D3 >= 800`, `R3 >= 0.20`, and `Rsum >= 0.08`.
Context gates remain `input_tokens <= 6000` and
`input_tokens + 1024 <= 8192`. Controls require zero legal reduction.
CP09/CP10 byte lengths do not establish token counts, reduction, or context
admission. No threshold was changed.

## Operator boundary and future commands

This task stops here. Only the human operator may later launch the accepted
server, after `Get-ChildItem Env:LLAMA_ARG_*` returns nothing. The future
command, **reported but not executed**, is:

```powershell
C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe serve -m D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf -c 8192 -np 1 --metrics --reasoning off --offline --host 127.0.0.1 --port 8080
```

After separate authorization and that operator-started server, the future
hybrid command is:

```powershell
& 'D:\Users\fleur\Prefixity\target\claim2-v3-tokenization-freeze\d7d6af170748e570e03326835916266efc90b531\prefixity-phase1c-claim2-v3-tokenization.exe' execute --mode HYBRID_8_NEW --operator-started --model-path 'D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf' --model-sha256 cd76ec205963b3b33350093e6904d9de16c4e666fd104e1f632d25c7f15f2a13 --llama-path 'C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe' --llama-sha256 cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617 --llama-build b10217-ddd4ec142 --context 8192 --slots 1 --reasoning off --offline --host 127.0.0.1 --port 8080 --evidence 'D:\Users\fleur\Prefixity\claim2-v3-hybrid-tokenization-pass-evidence.json'
```

If pre-contact inheritance verification instead selects the separate
contingency under the same registered instrument, the future command is:

```powershell
& 'D:\Users\fleur\Prefixity\target\claim2-v3-tokenization-freeze\d7d6af170748e570e03326835916266efc90b531\prefixity-phase1c-claim2-v3-tokenization.exe' execute --mode FULL_FRESH_22 --operator-started --model-path 'D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf' --model-sha256 cd76ec205963b3b33350093e6904d9de16c4e666fd104e1f632d25c7f15f2a13 --llama-path 'C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe' --llama-sha256 cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617 --llama-build b10217-ddd4ec142 --context 8192 --slots 1 --reasoning off --offline --host 127.0.0.1 --port 8080 --evidence 'D:\Users\fleur\Prefixity\claim2-v3-full-fresh-tokenization-pass-evidence.json'
```

Neither execution command was run. No server contact or tokenization is
authorized by this preparation record.
