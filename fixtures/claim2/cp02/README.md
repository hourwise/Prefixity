# CP02: repository investigation

This frozen case asks the model to trace the `quarry-ledger` lookup path in a
self-authored, pinned Rust repository. The repository under `snapshot/` is the
complete local source snapshot for revision `quarry-ledger-r1`; its files are
functional source, documentation, and a deterministic inventory command.
`snapshot/tools/symbol_inventory.py` walks the source files and emits every
declared item plus relevant qualified call sites in stable path and line
order. The public inventory attachment in `bodies/inventory.txt` is the
output of that command for query `symbol-map-v1`, so its rows describe the
actual snapshot rather than a hand-written symbol list.

Request 1 presents the task and broad-inventory menu. After its receipt,
request 2 retains request 1's messages unchanged, appends the original native
inventory, then presents the narrowed `Library::find_record` call-chain menu.
After the second receipt, request 3 retains all prior messages unchanged,
appends four direct source reads, then the identical inventory reattachment
and final question. Both inventory occurrences name repository revision
`quarry-ledger-r1` and the same query identity; their `SameStateRevision`
relation records that normal reattachment. Result receipts remain separate
from native Message bodies.

The hidden answer key is in `evaluation/key.json`. It records the exact path
and symbol chain independently of the model-visible prompt. The integration
test derives those path/symbol facts from the pinned source files, checks the
inventory rows against the source, runs the deterministic policy and shared
arm evaluator, and measures the three request skeletons. No model outputs are
stored in this fixture.

The per-file hashes are recorded in `snapshot/SHA256SUMS`.
All text files are UTF-8 with LF endings. The task, menus, receipts, direct
reads, generated attachment and all structural event identities are pinned
by `case.json`.
