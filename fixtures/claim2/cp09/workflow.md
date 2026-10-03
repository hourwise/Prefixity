# CP09 synthetic software-release promotion workflow, version 1

This case is a wholly synthetic, offline release-board fixture. Release,
build, image, environment, operator, and rollback identifiers are invented;
the builder creates no credentials and contacts no production system,
provider, model, or network.

## Complete governing record

The release office opens one immutable deployment-cohort manifest for every
promotion review. The manifest's complete-record contract is fixed before
materialization: release identity and revision; source/build/image digests;
named environment targets and cohort membership; required gate IDs; gate
owners; rollout order; and the exact rollback artifact ID for every production
target. A manifest missing any of these fields is incomplete and cannot be
reviewed. Cohort membership is the finite set listed in the manifest; no
outside service or deployment unit is implied. The manifest is the governing
object for this release decision.

## Ordinary release-board sequence

At intake, the release coordinator attaches the governing manifest beside the
frozen release-state snapshot. A read-only provenance check resolves the
source revision, build attestation, and image digests. The next board step
checks each required rollout gate against the same frozen snapshot and pinned
provenance receipt. Both actions are finite, read-only checks; neither changes
the release, deployment state, or rollback record.

When the promotion request is assembled, the standard change-board packet
recipe carries forward the unique check receipts, preserves the frozen state,
and attaches the complete governing manifest again so the approval decision
travels with the exact release cohort and rollback authority it approves.
This recipe is used for every release-board promotion review, irrespective of
document size or any benchmark arm. The final native manifest Message is
resolved from the same content-addressed revision as the intake attachment.

## Frozen state and decision

`source/deployment-cohort-manifest-v1.json` and
`source/frozen-release-state-v1.json` define the complete, closed synthetic
world. Their world-state revision is `release-promotion@REL-SYN-2026-10-03-04-v1`.
The manifest remains immutable through all three requests. The state snapshot
pins the source commit, builder attestation, image digest, required gate
results, production canary metrics, cohort membership, and rollback artifact
identity and digest.

The first deterministic result checks release/build/image provenance against
the frozen state. The second independently checks every manifest-required
gate, canary bounds, production cohort, and rollback artifact. The structured
board answer is `PROMOTE`, `HOLD`, or `ROLLBACK`, with the exact release and
build/image identities, governing gate IDs, production cohort identity,
required rollback artifact, and reason code. Source records and receipts are
read-only evidence, not mutations or approvals.

## Provenance boundary

All facts and identifiers are synthetic. The case supports deterministic
offline evaluator checks only. It provides no claim about real deployment
safety, software supply-chain security, operational release quality, or
tokenization performance. No byte-to-token estimate was used to compose the
records or define their completeness.
