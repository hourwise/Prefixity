# P0-L13 — Candidate Materialization and Safety Certificate

P0-L13 is Prefixity’s controlled, provider-neutral materialization boundary.
It converts a source `ConformanceRequest`, an approved P0-L11 candidate, and a
matching P0-L12 evaluation into an inert experimental request only when the
request can be reconstructed and proven to differ from the source solely by
the candidate’s authorized artifact reorder.

```text
P0-L11
proposes a structurally safe candidate
        ↓
P0-L12
gates what current evidence justifies
        ↓
P0-L13
materializes an inert experimental request and proves exactly what changed
        ↓
future live experiment
executes control versus treatment
```

The materializer does not execute the request, contact a provider, project it
into llama.cpp JSON, replace the source, or attach runtime observations. The
source remains the control and the materialized candidate is the treatment for
a future experiment. “Treatment” is neutral experiment language; it does not
mean optimized or proven.

## Authorization gate

Materialization fails closed unless all of these identities and boundaries
hold:

- the source fingerprint still matches the P0-L11 plan and planned diff;
- the candidate is the exact safe candidate recorded in that plan;
- the candidate layout references a permutation of the source segments;
- each reference preserves its role, logical identity, content fingerprint,
  and, where supplied, the complete P0-L2 metadata fingerprint;
- the P0-L12 evaluation identifies the same candidate, source, request diff,
  and safe structural result;
- the transformation is one supported adjacent swap or region-local move;
- P0-L7’s actual source-to-materialized diff agrees with the planned diff and
  contains only artifact order change;
- P0-L10 re-analysis reproduces the candidate structural result; and
- all conservation invariants pass.

Empirical cache evidence is not required to materialize a safe experiment
candidate. Performance and causal claims remain disallowed, and cache impact
remains `unknown`.

## Certificate boundary

`MaterializationSafetyCertificate` is a deterministic internal proof artifact,
not a cryptographic attestation or third-party security certification. It
records explicit results for source identity, candidate identity, authorized
transformation, model-visible content, artifact multiset, tool surface,
envelope, trust, provenance, order-only change, RequestDiff agreement, and
P0-L10 re-analysis.

The certificate proves transformation fidelity, not performance benefit.

For example:

```text
SOURCE

artifact order:
A B C D

CANDIDATE

artifact order:
A C B D

SAFETY CHECK

source fingerprint             pass
candidate fingerprint          pass
artifact membership            pass
artifact contents              pass
system instruction             unchanged
current user                   unchanged
tools                          unchanged
request envelope               unchanged
trust/provenance               unchanged
actual diff                    reorder only
planned diff                   reorder only
P0-L10 structural result       matched

CERTIFICATE

experiment materialization:
PASS

PERFORMANCE STATUS

unverified

CACHE IMPACT

unknown

EXECUTION

none
```

Artifact conservation is count-aware. An omitted occurrence, an unexpected
duplicate, or a same-identity artifact with a different content fingerprint
fails certification. When P0-L2 metadata is available, origin, content source,
revision, trust, lifecycle, version/hash, and provenance are covered by the
metadata identity fingerprint; metadata is never silently rewritten.

Tools remain in the source order with their names, descriptions, schemas, and
optional fields unchanged. The model, reasoning setting, and response format
remain unchanged. System instructions and current-user content are fixed.
No JSON canonicalization, whitespace normalization, tool selection, deletion,
duplication, summarization, compaction, or message rewriting occurs.

## Failure example

```text
REJECTED

artifact src/auth.rs retained logical identity
but content fingerprint changed

candidate must be replanned
```

The typed failure is bounded (`artifact_content_mismatch`); the materializer
does not return the source request as a successful fallback and does not return
a partial candidate. Other bounded failures cover stale source/evaluation,
unsafe candidates, unsupported transformations, missing or duplicated
artifacts, tool or envelope changes, trust/provenance mismatch, planned/actual
diff mismatch, and structural re-analysis mismatch.

`CandidateExperimentPair` records only the future control/treatment design:
source and candidate request fingerprints, candidate fingerprint, certificate
fingerprint, case IDs, and caller-supplied provenance. It contains no runtime
result, telemetry, cache observation, or performance claim. Its identity is
deterministic for identical inputs and provenance; it has no automatic
timestamp.

A materialized candidate is an experiment input, not an automatically applied
optimisation.
