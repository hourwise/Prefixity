# Phase 1C Stage 0 — Offline Replay Certification

## Certification result

**CERTIFIED — READY FOR SEPARATE STAGE 1 AUTHORIZATION**

This certification covers the offline replay procedure only. It certifies
runner construction, manifest freezing, deterministic mock transport,
evaluation, accounting, budget enforcement, abort handling, fail-open
behavior, redaction, leakage controls, and deterministic reporting. It does
not establish provider behavior, model quality, provider cache behavior,
pricing, live prompt safety, or production benefit.

No provider/model/API request, endpoint access, DNS/network probe, credential
read, credential provisioning, live schema smoke, real prompt replay, API cost,
production planner change, generic proxy, or policy promotion occurred.
Stage 1 and Stage 2 were not started.

## 1. Checkpoint and protected state

- Source checkpoint: `d20b3c09fa09cfcf403bdb57792ead55314ee6e6`
- Checkpoint message: `docs: add Phase 1C design authorization gate`
- CI identity: [CI #34](https://github.com/hourwise/Prefixity/actions/runs/31371467905)
- Design document hash: `ae2ae17a7eb1f12181e455a03129a8847cd3213a8ede5e606d950609344794c5`
- Protected `docs/tasks/ACTIVE.md` SHA-256:
  `D329C117BF346D65B2587B07EF9B13AA394E5796B580C623E71B1593853F17E2`

`ACTIVE.md` was not edited, staged, restored, reset, or included in the Stage
0 changes.

## 2. Runner and frozen cohort

- Runner version: `phase1c-stage0-runner-v1`
- Report schema: `phase1c-stage0-report-v1`
- Cohort count: 17 synthetic/self-authored sanitized tasks
- Cohort manifest hash:
  `3e75e1e2dddb5456f69b8a6470650ff27e88c6b187edfc307eb181e8c5f8d776`
- Arms, in fixed order: `BASELINE`, `NO_OP`, `INTERVENTION`
- Replicates: 1
- Maximum physical requests: 51
- Mock physical requests executed: 49
- Hard Stage 0 spend ceiling: `0`

The cohort consists of the 14 frozen Phase 1B.9 decision cases h001–h014 plus
three Stage 0 procedure cases:

- h001, h004, h007, and h009 exercise frozen `PRUNE`, `DEFER`, and
  `RELOCATE_CANDIDATE` transformations;
- h002, h003, h005, h006, h008, h010, h011, and h012 exercise no-op,
  dependency, protocol, ambiguity, repetition, and already-efficient paths;
- h013 exercises evaluator inconclusive handling;
- h014 exercises missing accounting-field handling;
- s015 exercises hard structural-safety abort;
- s016 exercises exact budget-boundary abort;
- s017 exercises a structurally valid intervention with no efficiency win.

The inherited Phase 1B.9 policy was used only as a frozen research input:

- policy version: `controlled-evidence-policy-v1`
- policy hash: `2139e084d97b16f3ae4ad36d95f0c73b4b1f448fe68f197139aa744dfe0e4`
- preregistration hash:
  `e12846776660960093f9208b099ca171dc4b9c9583150b58de340e965409cd3b`
- scope remains `CONTROLLED_ONLY`

No policy rule, order, threshold, or production planner path was changed.

## 3. Three-arm manifest certification

The immutable baseline and no-op payloads are byte-equivalent after excluding
run metadata:

- baseline manifest hash:
  `a3d8f3344808f46e2848f8626d06530002aca60f4b666f8bb730896c9ee8bd43`
- no-op manifest hash:
  `a3d8f3344808f46e2848f8626d06530002aca60f4b666f8bb730896c9ee8bd43`
- equivalence result: `PASS`

Intervention payloads are constructed from disposable copies. The declared
transformation validator recomputes the exact expected payload and rejects an
undeclared block/content mutation. Its tamper test passed.

- intervention manifest hash:
  `b17d8d96b5ca614b0095e245ffca84fed0052827024a63368de432ced5f4e5ac`
- intervention-diff integrity: `PASS`
- transformation hashes: recorded in the machine-readable report

The baseline payload remained byte-identical after all Stage 0 runs.

## 4. Mock transport and offline boundary

- Mock transport schema: `stage0-mock-transport-v1`
- Transport type: deterministic in-process mock only
- Network calls: `0`
- Credential reads: `0`
- Automatic retries: `0`
- Redirects followed: `0`
- Spend: `0`

The mock models sequential request order, arm identity, synthetic request IDs,
provider-native-style total/fresh/cache/output fields, logical latency,
tool outcomes, malformed schema, timeout, redirect, retry-required,
unexpected-tool, and missing-accounting paths. These values are explicitly
synthetic and are not provider evidence.

Stage 0 has no credential lookup, endpoint selection, DNS probe, HTTP client,
or live transport binding. The offline tests certify that Stage 0 execution
uses only the mock boundary.

## 5. Evaluator and leakage certification

- Evaluator version: `stage0-deterministic-evaluator-v1`
- Evaluator hash: `059caacecf53ccf26ffe6489b4fe9b1247bcaca382107f05b23a34359ed7f229`
- Evaluator certification: `PASS`
- Leakage certification: `PASS`

The deterministic evaluator checks baseline validity, task completion,
required blocks, required tool outcomes, dependency/protocol structure,
unexpected tools, and critical failure conditions. Evaluation keys remain
sidecar-only; the policy-facing projection contains opaque task/source data,
payload structure, and frozen selected decision metadata but not expected
benefit, evaluator outcome, gold state, or critical-failure labels.

Inconclusive outcomes are retained as inconclusive:

- h013: `INCONCLUSIVE_EVALUATOR`
- h014: `INCONCLUSIVE_MISSING_ACCOUNTING`

They are not converted into success or removed from the cohort denominator.

## 6. Accounting certification

The runner records per task/arm/request:

- task, arm, replicate, request order, and synthetic bounded request IDs;
- separate Prefixity-estimated and mock provider-native input units;
- total input, fresh input, cache read/write, output, rounds, tools, rereads,
  recovery, and logical latency;
- timeout, retry, redirect, schema, and evaluator state;
- physical request count.

Aggregate accounting records arm totals, no-op deltas, overhead, non-winning
and inconclusive cases. Missing required accounting is detected and reported,
not imputed. Cost remains exactly `0`; no provider pricing profile is invented.

Accounting certification: `PASS`.

## 7. Efficiency-gate logic

The runner implements and boundary-tests the frozen design logic:

- total input must not increase;
- fresh input must fall by at least 10%, or a frozen billed-cost branch must
  show at least 5% savings;
- output, rounds, tool calls, rereads, recovery, and physical requests must
  not increase;
- latency must remain within 10% unless explicitly non-comparable;
- cost branch is `UNAVAILABLE_NOT_APPLICABLE` because Stage 1 pricing is not
  authorized or frozen.

The mock pass cases exercise the logic only. They do not claim a real
efficiency improvement. s017 correctly records `NO_EFFICIENCY_WIN`.

Efficiency-gate logic certification: `PASS`.

## 8. Abort matrix

All 27 preflight and during-replay probes passed. Covered conditions include:

- source, design, policy, cohort, evaluator, transformation, and protected
  `ACTIVE.md` hash mismatches;
- undeclared arm diff and missing accounting requirement;
- nonzero spend, network, credential, retry, and redirect permissions;
- hard structural safety failure;
- baseline-pass → intervention-fail;
- malformed schema, unexpected tool, timeout, redirect, retry requirement;
- missing accounting, budget exhaustion, evaluator critical failure;
- artifact-redaction violation and ambiguous task/arm identity.

Every probe recorded zero automatic retries, preserved bounded evidence, and
preserved the baseline. The matrix result is `PASS`.

## 9. Rollback/fail-open certification

- Transformations operate on disposable copies only.
- A failed safety transformation sends no intervention request.
- A failed validation sends no request at that point.
- The immutable baseline remains unchanged.
- No automatic baseline fallback or retry is issued after a simulated send.
- Future recovery requires separate authorization.

Rollback/fail-open certification: `PASS`.

## 10. Redaction and privacy

- Redaction version: `stage0-redaction-v1`
- Synthetic bearer/API-key-like sentinel strings are removed.
- Raw response bodies are excluded from artifact projections.
- No actual credentials, private data, unrestricted provider bodies, or live
  headers were used.

Redaction/privacy certification: `PASS`.

## 11. Determinism and artifacts

The repeated Stage 0 run produced byte-identical reports and artifact inputs.

- Aggregate certification hash:
  `77203b2513e78635d618fbf64b1252976285c75ce7a518b04084acfd78c22d8a`
- Final certification determinism hash:
  `e8a8353bd96ca3aaf1160c9264ffc735921b3f0b7c2a34dadf58138c9f43ca0c`
- Machine-readable report:
  [`PHASE_1C_STAGE_0_CERTIFICATION.json`](PHASE_1C_STAGE_0_CERTIFICATION.json)

## 12. Unresolved Stage 1 inputs

The following remain explicitly unresolved and are marked
`REQUIRES_STAGE_1_AUTHORIZATION` in the manifest:

- provider, model, API surface, endpoint, account/region;
- credential environment-variable name;
- model settings and provider cache-control settings;
- live timeout and request/token/spend ceilings;
- exact provider pricing profile.

Stage 0 did not guess, validate, or select any of these values.

## 13. Validation performed

- `cargo fmt --all -- --check`: passed
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: passed
- focused Stage 0 tests: 6/6 passed
- existing Phase 1B.9 tests: exercised repeatedly by the Stage 0 runner and
  full workspace validation
- existing controlled-benchmark tests: included in full workspace validation
- manifest/report JSON parsing and required-content checks: passed
- deterministic hash/report validation: passed
- network-denial and credential-read-denial boundary tests: passed
- budget, abort, leakage, redaction, and rollback tests: passed
- `git diff --check`: passed

No network-capable command, provider endpoint, credential environment value,
live schema smoke, or replay was invoked.

## 14. Next authorization boundary

The only next task is a separate **Stage 1 authorization review**. It must
explicitly name the provider, model, API surface, endpoint, account/region,
credential boundary, model settings, cache isolation, task cohort, evaluator,
replicates, request/token/time/spend ceilings, pricing profile, artifact
retention/redaction, abort owner, and exact scope/expiration.

This Stage 0 result does not authorize Stage 1 or Stage 2. The proposed Stage
0 commit is:

`feat: certify Phase 1C Stage 0 offline replay`

Per the attached authorization boundary, the Stage 0 files are to be staged
for review, but no commit or push is performed without a later direct user
authorization.
