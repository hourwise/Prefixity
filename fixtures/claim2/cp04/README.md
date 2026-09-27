# CP04 local fixture record

This positive case is a self-authored, pinned local branch/backtrack diagnosis
fixture. It is offline materialization data; no model output is included.
case.json contains the public trace, deterministic action transitions,
request templates, and pinned public-asset hashes. The exact evaluation key
is isolated at evaluation/key.json; its SHA-256 is
d89525072cd7717fa75b1aa9fcae5c97a72673eab9a03429252d7bb6600369bd. The key
file is not a prompt asset and is not embedded in case.json.

## Scenario and identity

Source identity: cp04-r41. Initial task identity:
CP04-task-v1. Starting environment state: CP04_START_R41.
The complete source/config snapshot is the source_original native context
message and is pinned at 3342 UTF-8 bytes with SHA-256
de35a0d6e94711674f8ac039debbc318324077bc397bf4cf665770e0cbaaf5fe. The later
exporter occurrence e-recovery-copy has the same bytes, content hash, source
revision, and explicit world-state revision. The recovery packet is
RECOVERY-CP04-02.

Normal workflow provenance: after branch restoration is verified, the
standard recovery-packet exporter reattaches the unchanged R41 source/config
revision beside the recovery Result for request 3. The original occurrence
remains in the carried transcript. The repeated body is a native Message
asset from its creation, while each diagnostic action's actual output remains
a separately pinned Result receipt. The candidate configuration is stored as
a separate environment snapshot; it is not part of the original R41 attachment.

## Frozen trajectory and dependencies

There are exactly three request templates: initial inspection, deterministic
verification/diagnosis, and final answer. Request 2 and request 3 preserve the
original system/task messages and carry this arm's own raw assistant outputs
and each environment receipt in separate messages. The duplicate body first
appears in request 3. Each of the two request slots has two finite actions;
every menu action maps to one pinned action event, result event, receipt, and
resulting state. The evaluation action/state/final-answer/required-context
labels remain only in evaluation/key.json.

The offline planning scenario assumes 2048 raw UTF-8 bytes per assistant output.
This is an estimate, not an enforced cap or a conservative tokenizer bound;
the accepted output ceiling remains 1024 tokens.
Request 1 is a complete fixed body. Request 2 and request 3 prechecks are
non-dispatchable skeletons: each leaves its earlier raw output and environment
receipt unbound and reports the separate byte envelopes and receipt choices.
No future assistant output body or token count is claimed.

The duplicate occurrence has no consumer and no DependsOn or ProtocolPrecedes
edge. Its only relation endpoint is the exact SameStateRevision pair with
the earlier source occurrence. Action/result producer links remain pinned and
the shared fixture loader validates their closure.

Admission uses the live shared controlled-evidence-policy-v1 implementation.
The expected result is one EXACT_DUPLICATE_PRUNE candidate selected as
e-recovery-copy; fixture tests check the actual returned class, rule,
candidate inventory, and target before relying on that expectation. Candidate
size is not used to choose a target. Token counts are absent:
token_proof_inputs is null and the exact outcome is
EXACT_TOKENIZATION_REQUIRED.

## Pinned asset inventory

| Asset ID | Relative path | UTF-8 bytes | SHA-256 |
| --- | --- | ---: | --- |
| task_message | bodies/task_message.txt | 409 | 1af736cc7d41b13fd5d74ca9c126120a5d503b4456889768204052a7c524d1ba |
| source_original | bodies/source_export.txt | 3342 | de35a0d6e94711674f8ac039debbc318324077bc397bf4cf665770e0cbaaf5fe |
| source_repeat | bodies/cp04_repeat.txt | 3342 | de35a0d6e94711674f8ac039debbc318324077bc397bf4cf665770e0cbaaf5fe |
| system | prompt/system.txt | 397 | 72e04083be3efb4610490ee5369742f359a58ebf13ddc208c1468d9ae4b247c6 |
| request_1_menu | prompt/request_1_menu.txt | 275 | 31e2a53a7b3cccb7d4ad50201df9c731c259737f47c1ecb1413624ee9c4c06e7 |
| request_2_menu | prompt/request_2_menu.txt | 276 | 74af8c0c2cddf8535eea3e088f057e83d2b2c69d79468a484cfe0ec60fd7b169 |
| request_3_final | prompt/request_3_final.txt | 356 | 9d589cab3489674f1097dfad5cc1ba356a41bcb6cc6238dbc66cb697beb56648 |
| receipt1 | environment/receipt1.txt | 425 | 6ab3eb54f41ecf9ff1ba4a38b147a16c91f6425b073bd98357c7a98252546b69 |
| receipt1_alt | environment/receipt1_alt.txt | 237 | 1dc884db1cf2b63aeb0ab72355dd2034eaf6402e56efb07402bd92ffef75d043 |
| receipt2 | environment/receipt2.txt | 475 | e68f339375c2fe0c36583226e3e35db64e63407574ac7ec7d44d89b0d34ec874 |
| receipt2_alt | environment/receipt2_alt.txt | 228 | 9ba0e5477f95a05b63db4c4d7446cb555062fb879750ceb95061d0ae986fec75 |
| candidate_branch_state | source/quick-route-v42.toml | 201 | 3b27519814cbf449c3c2b38f3707e27acfa466e20d74541c7accf3df8839ffba |

## Source/config component inventory

| Local component | UTF-8 bytes | SHA-256 |
| --- | ---: | --- |
| source/branch-policy.toml | 208 | 2ca24b1d603f7b93a9390d4e42520c473e7900601452b3ec42c841bcfcfd1ccf |
| source/branch_rules.py | 867 | 53ec7ee623abe2134401e0b88a7093d7fd11c5c8017a5ee1ca400479e8c18139 |
| source/quick-route-v42.toml | 201 | 3b27519814cbf449c3c2b38f3707e27acfa466e20d74541c7accf3df8839ffba |
| source/recovery-protocol.md | 384 | 1576a4b064e2ae533bc500265d96d40ce2af43e1b95ff090a137d02f8c903c90 |
| source/recovery.py | 425 | d440408bbcdaac6afdfde9c91d19f7750fd2bd9979178c44a1622b710b7d136a |
| source/runtime.toml | 205 | fce74edd6df1f9597e3a25ee1c85598082284083de53bc9d17d386b2b53f0c5d |
| source/test_branch_invariants.py | 504 | 70f98194c1a5b024f840469de9fa159bef1a1e2fe5f618d1c4a222d42b655165 |
| source/test_recovery.py | 416 | a5c4c21fda8bba15ca8e1ba2cb14828790156f94b45664cea7b9490a5d9e6efa |

Manifest SHA-256: 7c3f089131b8b0338a3de601349ed2ab4e04b8a52741c6a5b76d1f7cee32c097.
