# CP01 local fixture record

This positive case is a self-authored, pinned local coding/configuration
diagnosis fixture. It is offline materialization data; no model output is included.
case.json contains the public trace, deterministic action transitions,
request templates, and pinned public-asset hashes. The exact evaluation key
is isolated at evaluation/key.json; its SHA-256 is
cbc28532dd7cfc20e806662a751a1fe8b2f133e8b0919a37786b660a9b04ad42. The key
file is not a prompt asset and is not embedded in case.json.

## Scenario and identity

Source identity: cp01-r41. Initial task identity:
CP01-task-v1. Starting environment state: CP01_START_R41.
The complete source/config snapshot is the source_original native context
message and is pinned at 2954 UTF-8 bytes with SHA-256
a18d19cfd7af0b2932d70e928f0edc4a191597a0a5dbf3e5f7d1889b0d7a45c9. The
later exporter occurrence e-repeat has the same bytes, content hash, source
revision, and explicit world-state revision. The verification packet is
EXPORT-CP01-VERIFY-02.

Normal workflow provenance: after the regression check, the standard
verification-packet exporter reattaches unchanged revision cp01-r41 beside
its Result receipt for request 3. The original occurrence remains in the
carried transcript. The repeated body is a native Message asset from its
creation, while each diagnostic action's actual output remains a separately
pinned Result receipt.

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
e-repeat; fixture tests check the actual returned class, rule,
candidate inventory, and target before relying on that expectation. Candidate
size is not used to choose a target. Token counts are absent:
token_proof_inputs is null and the exact outcome is
EXACT_TOKENIZATION_REQUIRED.

## Pinned asset inventory

| Asset ID | Relative path | UTF-8 bytes | SHA-256 |
| --- | --- | ---: | --- |
| task_message | bodies/task_message.txt | 389 | 167d984a1969c1fe3eb08fc610984188d4fb666d43a8d2ef9956eeb2c5aa0a3e |
| source_original | bodies/source_export.txt | 2954 | a18d19cfd7af0b2932d70e928f0edc4a191597a0a5dbf3e5f7d1889b0d7a45c9 |
| source_repeat | bodies/cp01_repeat.txt | 2954 | a18d19cfd7af0b2932d70e928f0edc4a191597a0a5dbf3e5f7d1889b0d7a45c9 |
| system | prompt/system.txt | 397 | 72e04083be3efb4610490ee5369742f359a58ebf13ddc208c1468d9ae4b247c6 |
| request_1_menu | prompt/request_1_menu.txt | 222 | c1b1c036ac715791417f17cb540d093b820f780e1c8c00241f5e56b77e053a2c |
| request_2_menu | prompt/request_2_menu.txt | 227 | 216da2d5430a68a3a437d0956774006752e5dcb9595229f2748f01072cdcba26 |
| request_3_final | prompt/request_3_final.txt | 307 | 3190d74f4fbf4743834c0d1d8d66303c82121e3797f3a1749b8608b356bd972c |
| receipt1 | environment/receipt1.txt | 312 | 3eaddfcb86e49ad105f6d11f3a900affae2e5ad3d0a10c54f4078ccff86484a8 |
| receipt1_alt | environment/receipt1_alt.txt | 233 | c0d0c383b38ccf83387538b772bca99582926f007d792e010e29fc3d30db1512 |
| receipt2 | environment/receipt2.txt | 616 | aa90d37f70fafb34c9409a7e025efe4636afb5ba1d3934d777e8aa88078bd7f2 |
| receipt2_alt | environment/receipt2_alt.txt | 343 | 7402586967697e72b6a6a638d78f3e9e1efb188369ede109848daa439c64761c |

## Source/config component inventory

| Local component | UTF-8 bytes | SHA-256 |
| --- | ---: | --- |
| source/dispatcher.toml | 133 | 69abe573301082770bb426c97c766fb6f0cfa6e96505fdc62a6ad1ad653cf0ff |
| source/policy_loader.py | 360 | afc1fc3fa1227e20abf53097c865e6329459265de01a4fb0288c53573c8cb78d |
| source/retry-contract.md | 347 | 9a1920d1a839735a8496c35720aec81e483f97937b4008c00839e7aa87e077d0 |
| source/retry.py | 719 | c6822e1b133d149570ca3190e731eba8e7265a5d849426b09925b719650f182f |
| source/test_policy_loader.py | 471 | d3900d9bdaa57094e00b8fd10b569c44edbebc9af84a2b503a66baa230032b7b |
| source/test_retry.py | 624 | 8f70b66cc5185252a2a583dc02a3c19fb7ce0480db100d5346adf496398b6ca9 |

Manifest SHA-256: a36fd2bcf0e1f6cb68f3c7ff1a2ea2ca6536dfaafd38a59d3d2d6d4615dd4852.
