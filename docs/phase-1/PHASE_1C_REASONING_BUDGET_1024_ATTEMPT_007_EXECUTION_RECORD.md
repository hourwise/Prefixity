# Phase 1C Reasoning-Budget 1024 Attempt 007 Execution Record

## Terminal classification

The authorized live boundary was crossed once. The supervisor completed, all
three frozen requests were dispatched in order, and the raw evidence was
preserved. The live supervisor and child passed the repository's runtime
handoff and parent/child identity checks.

The run is not execution-accepted, however, because the executable objects
recorded at runtime did not match the executable hashes and file identities
bound into the accepted preparation identity. The preparation-time target
artifacts were replaced before the live process launch. The existing runtime
validator records and validates the live objects but does not compare them to
the preparation identity's cached target-binary fields. This is recorded as a
material integrity deviation; no result is promoted to a valid calibration
disposition.

```text
PHASE_1C_ATTEMPT_007_EXECUTION_INVALID - STOP FOR REVIEW
REASON: FROZEN_EXECUTABLE_IDENTITY_MISMATCH
ATTEMPT_007_EXECUTED_ONCE
```

This record does not authorize a rerun. The raw candidate protocol result is
reported separately below and must not be interpreted as a wider Phase 1C
approval.

## A. Starting baseline

Before startup, `HEAD`, local `main`, and `origin/main` all resolved to:

```text
c3059a1f2933ce0a4c9107ae03ec634b55c3fd69
```

The worktree was clean and there were no staged, unstaged, or untracked
changes affecting the Phase 1C boundary. The accepted Attempt 007 identity,
sidecar, preparation record, workflow-certification material, and preserved
Attempt 006 artifact were present.

## B. Pre-execution identity verification

The following canonical or preserved hashes matched before startup:

| Artifact | SHA-256 |
| --- | --- |
| Attempt 007 canonical identity | `fd536cb507aab2b76511e1731d3860bbbc0074b940e4e21067ec7c6b1f4bbbe8` |
| Workflow certification manifest | `824b65a0f93e18a12e917fd49662790bdb2dce945c6e4f8f0c8b72a9c41524a4` |
| Calibration manifest | `4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b` |
| Attempt 006 preserved supervisor artifact | `c8feaf01a4083d609dd5fdf5ccaf96b40db56c1cbb3ef0b7bb88f52b67724b07` |

The offline `attempt-007-fingerprint` check passed before execution and also
passed after cleanup. It reproduced the three frozen request hashes without
network contact. The model reference remained
`ggml-org/Qwen3.5-0.8B-GGUF:Q4_0`; no local GGUF path or hash was frozen or
observed.

The accepted preparation identity bound these target artifacts:

| Artifact | Preparation-bound size / SHA-256 / file ID | Runtime-recorded size / SHA-256 / file ID |
| --- | --- | --- |
| Supervisor | `1197056` / `8427e2ca64fce890e24d8d0474ee9e964e367b7d805caf0dbdbfb3abecc592db` / `0x0000000000000000000100000039821c` | `1187840` / `f5c7e7eed98be6dcb8976f33b2bff6759c05e350d507e5b0914515fa64faadba` / `volume=ba2f80f4;index=001700000036a34d` |
| Child | `8448000` / `844e02c7aab8d6223c93c4f9e1c94b74877c1ec766066037bf999d9f05745a39` / `0x00000000000000000006000000393382` | `8445440` / `efedfa712758d5be9a0a0f24bce401f69de2153774f86ce33a685990cb2ffad0` / `volume=ba2f80f4;index=000600000039337f` |

This mismatch is why the terminal classification is invalid rather than
execution-accepted. It is not repaired in this record and does not alter the
frozen identity file.

## C. Virgin-attempt gate

The prestart result was:

```text
ATTEMPT_007_VIRGIN
```

Evidence at the consumption boundary showed no Attempt 007 result root, no
execution record, no request record, no partial result, no retry record, no
Attempt 007 process, no stale handoff, and no execution lock. Certification
identity `900` remained certification-only. The required native exclusivity
result was `EXCLUSIVE_PRESTART`.

## D. Runtime launch

The exact command from the accepted preparation was used once:

```text
C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe serve -hf ggml-org/Qwen3.5-0.8B-GGUF:Q4_0 -c 8192 -np 1 --metrics --reasoning on --reasoning-budget 1024 --host 127.0.0.1 --port 8080
```

| Field | Observed value |
| --- | --- |
| Executable | `C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe` |
| Server PID | `21196` |
| Startup UTC | `2026-09-06T19:07:07.1810174Z` |
| Build | `b10217-ddd4ec142` |
| Server SHA-256 | `cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617` |
| Context / slots | `8192` / `1` |
| Metrics / reasoning | enabled / `on` |
| Reasoning budget | `1024` |
| Temperature / top-p / seed | `0` / `1` / `1` |
| Output limit / stream | `2048` / `false` |
| Endpoint | `127.0.0.1:8080` |

The startup log recorded the non-fatal Hugging Face repository lookup warning,
then model loading and `listening on http://127.0.0.1:8080`. No local GGUF
path was printed. The stderr log is 20,613 bytes with SHA-256
`ad852698c97bfd315bf5c9662a73e10717ec2823bb39f50f4687412779b5e760`; the
empty stdout log has SHA-256
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

Readiness used the established single `tcp_listener_connect` check against
`127.0.0.1:8080`. It passed on the first attempt, took 2 ms, made one
readiness contact, and made zero inference requests.

## E. Certified supervisor execution

The exact command from the accepted preparation was used:

```text
.\target\debug\prefixity-phase1c-live-supervisor.exe --attempt-identity docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_007_IDENTITY_V1.json --evidence experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-007/supervisor.json -- .\target\debug\prefixity-phase1c-reasoning-budget-calibration.exe run-attempt-007
```

The supervisor completed with child exit code `0`, one child launch, zero
child retries, and no termination. The registered handoff was:

| Field | Observed value |
| --- | --- |
| Attempt / budget | `7` / `1024` |
| Candidate identity | `phase1c-reasoning-budget-1024` |
| Supervisor PID | `23196` |
| Child PID | `7864` |
| Child parent PID | `23196` |
| Launch identity | `phase1c-attempt-7-budget-1024-fd536cb507aab2b76511e1731d3860bbbc0074b940e4e21067ec7c6b1f4bbbe8` |
| Handoff transport | single supervisor-generated serialized environment metadata |
| Workflow result | `COMPLETED`; poststart `READY` / `EXCLUSIVE_POSTSTART` |

The live supervisor and child executable objects passed the repository's
runtime identity and parent/child ownership checks. Their mismatch against the
preparation-bound target artifacts is the separate integrity failure recorded
in section B.

## F. Request ledger

Exactly three requests were dispatched, in the frozen order. There was no
fourth request, retry, fallback, repair request, or adaptive replicate.

| Ordinal | Case | HTTP / state | Finish | Prompt / completion / cached tokens | Response bytes | Request SHA-256 | Response SHA-256 | Elapsed ms |
| ---: | --- | --- | --- | ---: | ---: | --- | --- | ---: |
| 1 | `rbcal-001` | `200` / `PASS` | `stop` | `332 / 1125 / 0` | `3794` | `f32863dfb1da27c00a61d54986d4984569c87e9636cf5c6263c69906cb336461` | `bc256c377e2b0dc9396b2935e52fb28bc23230ec169fa2b8792a974b3832b2f6` | `171347` |
| 2 | `rbcal-002` | `200` / `FAIL` | `length` | `335 / 2048 / 42` | `7595` | `e9cb29143ed1be27ce5c5b27bda4daa546ff63825189b170b37083624534c1b3` | `c82e5a4f0a8ec6647d24b75041da3d2404e210d5e2da1618ec46601884b3018b` | `222653` |
| 3 | `rbcal-003` | `200` / `FAIL` | `length` | `339 / 2048 / 42` | `8478` | `2c9839a9482080b3d03fa89d908c63d442d35ec64e142d5c573d276e801aec7e` | `4dbef142ea57aa5ec1024137fb817a17b993ae7723839d4ff26805006df30abb` | `134568` |

`rbcal-001` was structurally valid and terminal. `rbcal-002` and `rbcal-003`
returned HTTP 200 but ended at the output limit without terminal final
content and failed structural validation.

## G. Runtime accounting

```text
model/server startups:       1
readiness contacts:          1
HTTP requests:               3
inference requests:          3
automatic retries:           0
fallback requests:           0
adaptive replicates:         0
Attempt-007 executions:      1
supervisor network calls:    0
supervisor inference calls:  0
```

The candidate result reports `network_calls: 4`, comprising the one TCP
readiness contact and the three HTTP requests.

## H. Evidence freeze

The raw evidence root is:

```text
experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-007/
```

The root was frozen before interpretation. It contains 23 files: the
candidate result, preflight, readiness, runtime confirmation, supervisor
record, and the complete three-case request/response evidence. The core
artifacts are:

| Artifact | Size | SHA-256 |
| --- | ---: | --- |
| `candidate-result.json` | 2844 | `94ae771670872ba6abf7ffd7aecd309656db8897a692f82bc601c6a6c80b4ea2` |
| `preflight.json` | 4935 | `63f32ce3cd1c5a2cc951d2e3d76769c64402532b442f0e91be2b1d2fb15e78df` |
| `readiness.json` | 194 | `dcccd1282bbe076dd6ae2e1461d7c7ed9a3556f39a22e78d1688e6698fdd49ad` |
| `runtime-confirmation.json` | 854 | `6901b717ac37616a67d9a4ea5680353af8507092d2a8eb1148ded1b77aa1e267` |
| `supervisor.json` | 2644 | `ba2b913d09ee8cb0b5c9646b0aa6f7ceb5b04279c1e58dbf6f385cae248a72a6` |

The per-case inventory, including every raw request and response hash, is:

```text
rbcal-001/case-result.json             09441f8fd11a4353f0c7833b488d05669308da2af99b4572fb2aa0770ef06054
rbcal-001/normalized-response.json    5daf367bdf6eaa1347b50a2e759426b7ac6b107cf6fcc34e9857791ef2ed5c8a
rbcal-001/reasoning-turn-1.json       17714a117c0457c236918dba11b35a3926f6c07816d89088117a622586c7f514
rbcal-001/request-record.json         9b26f3016b301bda164c2dccf7c420c0cdba638dda4d5e189bd35efd40a16f23
rbcal-001/request-turn-1.json         f32863dfb1da27c00a61d54986d4984569c87e9636cf5c6263c69906cb336461
rbcal-001/response-turn-1.bin         bc256c377e2b0dc9396b2935e52fb28bc23230ec169fa2b8792a974b3832b2f6
rbcal-002/case-result.json             f99f1a92ebbed516c0adb3d153a8d8ea9878fe3a4157ecea0bb3ce27d54e4217
rbcal-002/normalized-response.json    d84857a2e76add574e75d89693031d226bf74acf9d43833c596232bc6e6ea865
rbcal-002/reasoning-turn-1.json       4227cb157baf9701146d7d37b3c8162a4ef3854e169074f0377e9632f52f0b7c
rbcal-002/request-record.json         352a5314551fb557f5b94551e0927aba38404e1a5e8c53de2db774ae5881850a
rbcal-002/request-turn-1.json         e9cb29143ed1be27ce5c5b27bda4daa546ff63825189b170b37083624534c1b3
rbcal-002/response-turn-1.bin         c82e5a4f0a8ec6647d24b75041da3d2404e210d5e2da1618ec46601884b3018b
rbcal-003/case-result.json             b90f30faacc06068931ff34f9ad371c7f766873045805654513ef863c80627e6
rbcal-003/normalized-response.json    c499f547d4e81f05905d6b35182ff449d27a1720b1ef61359a89c36f697c9fac
rbcal-003/reasoning-turn-1.json       dd8bba5e38ff4dd2d14842acb5f7030080a6bc7cb9e7026a65e2af569828f29f
rbcal-003/request-record.json         cac553d38dd018115766b9402fe28a9d3481d5de4cea22c107ed3e0d7d63533f
rbcal-003/request-turn-1.json         2c9839a9482080b3d03fa89d908c63d442d35ec64e142d5c573d276e801aec7e
rbcal-003/response-turn-1.bin         4dbef142ea57aa5ec1024137fb817a17b993ae7723839d4ff26805006df30abb
```

The preserved Attempt 006 artifact was rehashed after the run and remained
`c8feaf01a4083d609dd5fdf5ccaf96b40db56c1cbb3ef0b7bb88f52b67724b07`. No
historical evidence was modified.

## I. Calibration result

The frozen candidate result has schema state `FAIL`, `case_set_complete: true`,
and `selection.next_budget: 512` with no selected budget or terminal protocol
classification. Under the calibration protocol, the observed 1024 candidate
failed because two of three cases were non-terminal at the 2048-token limit.
The next design candidate is 512, but 512 and 256 were not run and are not
authorized by this task.

Because the preparation-bound executable identity was not the executable
identity consumed by the live run, the raw protocol `FAIL` is evidence only;
there is no admissible calibration disposition for budget 1024 from this
execution. No capability claim or wider Phase 1C conclusion is supported.

## J. Runtime cleanup

The exact launched server PID `21196` was stopped after the supervisor
completed. Post-run checks confirmed:

- no `llama.exe` or `llama-server.exe` process;
- no Phase 1C supervisor or calibration child process;
- no `LISTENING` owner on port 8080 (only a non-owning `TIME_WAIT` entry was
  observed during shutdown verification);
- no `PREFIXITY_PHASE1C_WORKFLOW_HANDOFF` environment handoff;
- no Attempt 007 execution lock.

## K. Publication and CI

The bounded closeout documents are published from a dedicated result branch:

```text
branch: codex/phase1c-attempt-007-result
parent: c3059a1f2933ce0a4c9107ae03ec634b55c3fd69
```

The exact result commit, tree, CI run, and final fast-forward state are
reported in the task handoff after publication. The bounded changed files are
this execution record, `docs/INDEX.md`, and `docs/tasks/ACTIVE.md`. No source
code, frozen identity, Attempt 006 evidence, certification evidence, or raw
ignored run evidence was modified.

## Validation record

Offline closeout checks included:

- post-run `attempt-007-fingerprint` passed with zero network or inference
  calls;
- evidence-root file inventory and SHA-256 recomputation;
- candidate request count/order, case-set completeness, retry/fallback state,
  and supervisor result inspection;
- server, process, listener, handoff, and lock cleanup checks;
- `git diff --check` and the normal repository validation/CI gate before
  publication.

The binary mismatch is deliberately preserved as a review issue. This record
does not authorize another live process, another request, a retry, or a
source remediation.
