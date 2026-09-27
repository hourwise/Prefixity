# CP05 workflow: route catalog across registry states

The pinned local `marketplace-edge` workspace exports the ten active gateway routes in sorted order. Each row records its owner, scopes, timeout, health endpoint, retry and fallback behavior, rollout ring, data class, and handling note. This inventory is used to reconcile gateway configuration.

At `workspace-r17`, native occurrence `e-catalog-initial` carries source revision `route-catalog-export@R17` and world state `route-registry@R17`. Operation `rebuild-route-index-064` rebuilds only the secondary index, advances `route_index_generation` from 71 to 72, and creates `workspace-r18`. It does not modify catalog object `catalog-object-6f3a`. The ordinary recovery export then emits the same catalog bytes as new native occurrence `e-catalog-after-reindex`, with distinct source and world-state revision identities.

The exact state snapshots and body files are pinned beside this workflow. The snapshots name the artifact hash, operation, and changed field. Equal catalog bytes therefore do not establish equal world state. Both event IDs and both revision identities are required for the final comparison. This is a natural re-export after a real local state transition; no repeated message was added to satisfy a size target.
