# Phase 1B.1 Characterization Report Schema

Status: frozen for the Phase 1B.1 CodeTraceBench characterization.

## Identity

The report uses the stable schema name
`prefixity.phase1b1.characterization` and schema version `1`. The planner
contract version is reported separately and comes from the emitted
`InterventionPlan.contract_version`.

The canonical hash input is a UTF-8 JSON array sorted by relative trace path.
Each successful entry contains only the sanitized relative trace path,
request ID, and SHA-256 digest of the complete planner plan. Failure entries
contain the relative trace path and a sanitized failure classification. JSON
objects are serialized with sorted keys, compact separators, and UTF-8
characters escaped. This hash is an audit identity, not a quality or savings
metric.

## Report sections

The report contains these top-level sections:

- `schema` — schema name, version, canonicalization, and the frozen class,
  reason-code, evidence, and safety field vocabularies.
- `corpus` — corpus name, exact revision, split, fixture identity, accepted
  trajectory/request counts, source-event count, and Phase 1A established
  exclusions.
- `planner` — intervention contract version, frozen Phase 1B.0 checkpoint,
  Git base checkpoint, offline CLI mode, binary digest, and availability of
  provider/economic/quality inputs. Evaluation labels are always false here.
- `execution` — attempted traces, successful plans, individually accounted
  failures, first/second pass aggregate hashes, and the deterministic match.
- `decision_distribution` — recommendation counts, trace coverage by class,
  target-block counts, non-no-op trace count, `DO_NOTHING` trace count, and
  traces with at least two actual intervention candidates. Counts are not
  savings claims.
- `evidence_distribution` — recommendation-level counts for reason codes,
  evidence strength, expected quality risk, provider-state dependence, and
  independent provider/economic/quality/dependency evidence dimensions.
- `safety_audit` — explicit counts for the required destructive, dependency,
  relocation, compression, contradiction, `DO_NOTHING` coexistence, and
  source-integrity invariants. A clean audit has zero for every failure
  count.
- `deterministic_examples` — at most one lexicographically first sanitized
  recommendation per emitted class, with IDs, reasons, and compact evidence
  metadata only.
- `post_hoc_label_audit` — optional trajectory-level solved/unsolved overlay
  loaded after planner execution. It cannot contain planner inputs. Exact
  step-level overlap is reported as unavailable when message IDs cannot be
  reliably joined to evaluation step IDs.

## Counting rules

Recommendation counts are counts of emitted recommendation records. Trace
coverage counts a class at most once per trace. Target-block counts sum the
number of target IDs in records of that class. An actual intervention
candidate is `DEFER`, `PRUNE`, `RELOCATE_CANDIDATE`, or
`COMPRESS_CANDIDATE`; `KEEP` and `DO_NOTHING` are not intervention candidates.
Evidence counters are recommendation-level. Quality evidence is present
only when the planner's quality evidence list contains a positive fact rather
than the Phase 1B.0 absence note. Dependency states are `UNCERTAIN` when
unknown dependency evidence is present, `RELEVANT_DEPENDENCY` when recorded
dependencies are relevant to the recommendation, and
`NO_RELEVANT_DEPENDENCY` otherwise.

The six contract classes and all safety-audit fields are present even when
their counts are zero. No report field converts these observations into
token, cache, cost, latency, or task-quality claims.

