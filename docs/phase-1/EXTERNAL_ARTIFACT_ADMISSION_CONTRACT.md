# External Artifact Provenance and Admission Contract v1

## Purpose

This document describes a small deterministic contract for recording and
validating whether an external benchmark, corpus, trajectory, result archive,
source repository, or evaluator artifact is admissible for a specific
Prefixity research operation.

The contract is research-data governance infrastructure. It validates
recorded evidence and conservative Prefixity admission policy. It does not
provide legal advice, determine copyright ownership, or infer legal rights.

The implementation is isolated in
crates/prefixity-controlled-benchmark. It is offline-only and does not fetch
source URLs, query GitHub or Hugging Face, download licenses, execute
third-party code, contact providers, or inspect remote repositories.

## Why the contract exists

Prefixity has repeatedly encountered the same boundary:

1. an artifact is identifiable and technically accessible;
2. the framework/code license is easier to identify than dataset or reuse
   rights;
3. the artifact may contain third-party repositories, issue text, patches,
   source excerpts, or trajectories;
4. the evaluation target may not have a stable deterministic join;
5. gold labels or post-intervention behavior may leak into planner evidence;
6. raw-data retention and redistribution may be more restricted than metadata;
7. a permitted operation for one research use may not permit another.

The manifest makes those distinctions explicit. Unknown remains unknown.
Public or ungated access is never promoted automatically into permission for
raw reading, transformation, redistribution, or vendoring.

## Versioned manifest

The serde-compatible top-level type is
ExternalArtifactAdmissionManifestV1.

- schema ID: prefixity.external-artifact-admission.v1;
- schema version: 1;
- artifact identity: bounded artifact ID, artifact kind, canonical source,
  source owner, immutable revision kind, immutable revision, optional parent
  project, and provenance references;
- requested use: the operation for which admission is being evaluated;
- evidence sections: license/reuse, permissions, third-party material, join,
  gold independence, content sufficiency, retention, and execution.

Identity is required to include an immutable revision for reproducible
research. The validator does not dereference or verify that revision; it
validates that the manifest records one in a bounded field.

Artifact kinds include:

- BENCHMARK_DATASET;
- TRAJECTORY_DATASET;
- RESULT_ARCHIVE;
- SOURCE_REPOSITORY;
- EVALUATOR_FRAMEWORK;
- OTHER.

## Evidence versus derived decision

The manifest records evidence. The caller cannot set a trusted final
admission field and bypass validation.

derive_admission evaluates the recorded manifest and returns a structured
AdmissionDecisionReport containing:

- artifact identity and requested use;
- the derived decision;
- deterministically ordered reasons;
- blocking evidence fields;
- deterministically ordered warnings.

An invalid schema, unsupported version, unknown field, oversized field, or
incompatible evidence combination produces INVALID_MANIFEST in the derived
report. JSON parsing and explicit validation APIs also return typed errors.

Reasons are sorted by typed reason code, field, and message. This makes reports
stable across repeated evaluations without using a hash or source fetch.

## License and reuse evidence

License/reuse evidence is not one license string. The manifest records
separately:

- framework/code license evidence;
- dataset/artifact reuse evidence;
- underlying third-party material evidence.

Each evidence record uses one of:

- EXPLICIT;
- DECLARED_BUT_UNVERIFIED;
- ABSENT;
- UNKNOWN;
- NOT_APPLICABLE.

Explicit or declared evidence must carry a bounded source locator. A
repository's framework license therefore cannot silently become a dataset
license, and a benchmark paper cannot silently become raw-data reuse
permission.

The report may warn that dataset reuse evidence is not explicit while still
admitting metadata or a separately recorded operation basis. The decision is
based on the requested use and operation evidence, not on a legal conclusion.

## Requested-use profiles

Admission is operation-specific:

### METADATA_RESEARCH

Requires recorded metadata-inspection permission and a retention boundary that
does not track raw artifact or trajectory content in Prefixity. It may produce
ADMISSIBLE_METADATA_ONLY even when raw reuse is unknown or not established.

### EXTERNAL_FRONT_HALF_EVALUATION

Requires:

- permission to read and parse the local raw artifact;
- permission for the bounded local transformation;
- recorded provenance;
- a deterministic exact join;
- BLIND_TO_GOLD with a separate evidence basis;
- SUFFICIENT_FOR_FRONT_HALF content;
- raw and third-party content excluded from tracked Prefixity data;
- no uncleared provider/model inference, third-party code, or container
  execution requirement.

Success produces ADMISSIBLE_LOCAL_STUDY. It does not authorize a provider
call, replay, prompt mutation, adapter implementation, or production planner
change.

### LIMITED_PILOT

Uses the same permission, provenance, join, gold, retention, and execution
controls but permits LIMITED content. Success produces
ADMISSIBLE_LIMITED_PILOT and carries a content limitation warning. It cannot
support a full front-half claim.

### RAW_REDISTRIBUTION

Requires explicit recorded permission to redistribute and vendor the raw
artifact, explicit artifact and underlying-material reuse evidence, no
unknown/present third-party material in the manifest, and explicit raw
tracking permission. Success produces ADMISSIBLE_RAW_REDISTRIBUTION.

### REFERENCE_ONLY

Records the artifact as a reference without admitting raw local use.

## Permission basis

Each operation is represented independently with:

- PERMITTED_EXPLICIT;
- PERMITTED_BY_RECORDED_BASIS;
- NOT_PERMITTED;
- UNKNOWN.

Tracked operations include metadata inspection, raw download, raw read/parse,
local transformation, retention of bounded metadata and aggregates, source
excerpts, raw redistribution, and raw vendoring.

Permission for metadata inspection does not imply permission for raw parsing.
Permission for local study does not imply permission for redistribution.
Raw redistribution requires PERMITTED_EXPLICIT, not merely a recorded basis.

## Third-party provenance

The manifest records presence or uncertainty for:

- source code;
- issue or pull-request text;
- patches;
- tests or test patches;
- tool output containing source excerpts;
- private or user data;
- otherwise unknown third-party material.

Each field is EXPLICIT_PRESENT, EXPLICIT_ABSENT, or UNKNOWN. Unknown/private
material blocks the applicable conservative operation. The contract does not
attempt to decide whether a present item could legally be redistributed.

## Stable join

The stable-join section records:

- exact join classification:
  EXACT_ONE_TO_ONE, EXACT_ONE_TO_MANY, EXACT_MANY_TO_ONE, AMBIGUOUS, NONE,
  or UNKNOWN;
- typed key kind;
- left and right identifier descriptions;
- deterministic exact-match boolean;
- ambiguity marker;
- optional expected and bounded observed counts.

The front-half profile admits only deterministic exact classifications. No
fuzzy, semantic, patch-based, or inferred join is implemented.

## Gold/evaluation conditioning

Gold independence is explicit:

- BLIND_TO_GOLD;
- GOLD_CONDITIONED;
- UNKNOWN;
- NOT_APPLICABLE.

BLIND_TO_GOLD requires a separate evidence basis. The absence of a field
named gold is not enough. GOLD_CONDITIONED, UNKNOWN, and NOT_APPLICABLE block
primary external front-half evaluation.

## Content sufficiency

Trajectory/result content records the presence or uncertainty of chronology,
tool calls, tool results, file reads/views, search or symbol activity,
observations, edits/actions, and stable task identity.

The derived sufficiency level is one of:

- SUFFICIENT_FOR_FRONT_HALF;
- LIMITED;
- INSUFFICIENT;
- UNKNOWN.

A final patch or score is not treated as a trajectory. Full front-half
admission requires chronology, stable task identity, and observed
action/result material.

## Raw-data retention

GitRetentionPolicy records whether Prefixity may track:

- raw artifacts;
- full trajectories;
- source file bodies;
- source excerpts;
- problem statements;
- patches or test patches;
- opaque task IDs;
- hashes;
- source URLs and revisions;
- license/provenance metadata;
- structural metadata;
- aggregate metrics.

For a normal bounded external study, raw artifact and third-party content
must be DO_NOT_TRACK. Identifiers, hashes, source/revision metadata,
provenance metadata, structural metadata, and aggregate metrics may be TRACK
when separately supported.

The contract never adds the current ContextBench or Tracebench material to
fixtures. The validator receives only the manifest supplied to it.

## Execution requirements

The manifest records whether static parsing, archive decompression,
third-party code execution, container execution, network access, or
provider/model inference is NOT_REQUIRED, REQUIRED, or UNKNOWN.

For external front-half admission, required or unknown provider/model
inference, third-party code execution, or container execution blocks the
decision. Network access is reported as a warning because this contract
performs no network operation and applies only to supplied local evidence.

These are research controls for the requested operation, not global claims
about every possible use of the artifact.

## Derived classifications

The v1 decision enum includes:

- ADMISSIBLE_LOCAL_STUDY;
- ADMISSIBLE_LIMITED_PILOT;
- ADMISSIBLE_METADATA_ONLY;
- ADMISSIBLE_RAW_REDISTRIBUTION;
- REFERENCE_ONLY;
- BLOCKED_PERMISSION;
- BLOCKED_PROVENANCE;
- BLOCKED_JOIN;
- BLOCKED_GOLD_INDEPENDENCE;
- BLOCKED_CONTENT_SUFFICIENCY;
- BLOCKED_EXECUTION_REQUIREMENT;
- INVALID_MANIFEST.

Blocking reasons are selected from typed evidence failures. If multiple
failures exist, all are retained in the report while the primary decision is
chosen by a fixed conservative precedence. This is a research-admission
classification, not a legal determination.

## Fail-closed behavior

For optimization, Prefixity fails open to retention. For artifact admission,
uncertainty fails closed with respect to the requested risky operation:

- public or ungated access does not imply raw permission;
- unknown dataset reuse evidence may still permit metadata-only research but
  does not by itself permit raw local evaluation;
- unknown permission blocks the operation that needs it;
- unknown or absent deterministic join blocks front-half evaluation;
- unknown or gold-conditioned independence blocks primary external evaluation;
- insufficient chronology/content blocks front-half evaluation;
- unknown third-party or raw-retention status blocks broad redistribution.

The report explains the blocking field rather than making a legal claim.

## Synthetic fixtures and validation

The test suite uses fictional, non-networked evidence only. It covers:

1. explicit local-study permission and exact join;
2. public/ungated access with unknown raw permission;
3. code-license evidence without dataset permission;
4. gold-conditioned and unknown gold independence;
5. missing deterministic join;
6. final-patch-only limited content;
7. limited pilot;
8. redistribution with non-explicit permission;
9. provider/model execution requirement;
10. malformed, oversized, unknown-field, and unsupported-version manifests.

Tests cover JSON round-trip, stable reason ordering, different decisions for
different requested uses, no-panic malformed input, and all required
fail-closed branches.

## Relationship to current ContextBench/Tracebench work

This generic contract does not encode the current ContextBench or Tracebench
decisions as executable fixtures. The historical evidence remains in:

- docs/phase-1/CONTEXTBENCH_FRONT_HALF_EXTERNAL_EVIDENCE.md;
- docs/phase-1/CONTEXTBENCH_EXTERNAL_TRAJECTORY_ADMISSION.md;
- docs/phase-1/PHASE_1C_EXTERNAL_EVIDENCE_FRONT_HALF_GATE.md.

The current research state remains unchanged:

- ContextBench is EuniAI/ContextBench at the pinned revision
  1436c28a8eb95496da4ea69ad458b9f8a8eb7d61;
- its bounded result remains
  NO-GO — BENCHMARK ADMISSION/TRAJECTORY INSUFFICIENT;
- Tracebench remains the strongest technical candidate at observed revision
  7da2e4f45b330be8b6e8f1cff835247723cb3341;
- its result remains
  NO-GO — NO PERMISSION-CLEARED EXTERNAL TRAJECTORY ARTIFACT FOUND;
- Stage 1 remains blocked pending external reuse clarification;
- no response from Tracebench maintainers is assumed or recorded.

This implementation does not admit Tracebench, download raw trajectories,
inspect raw archives, implement a ContextBench adapter, contact maintainers,
call a provider/model, access credentials, or change the production planner.

## Explicit non-goals

This v1 contract does not:

- give legal advice or determine legal rights;
- fetch or verify a remote license or revision;
- download, inspect, transform, or redistribute external data;
- perform fuzzy or semantic joins;
- infer gold independence from missing fields;
- execute third-party code, containers, or benchmark repositories;
- call providers or models;
- modify prefixity-core, InterventionPlan, or
  controlled-evidence-policy-v1;
- couple artifact admission to production intervention planning;
- add a CLI command or JSON Schema dependency.
