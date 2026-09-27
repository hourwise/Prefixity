# quarry-ledger

`quarry-ledger` is a small local library and command-line tool for looking up
records in immutable segment files. This checked-in snapshot is the complete
repository state named `quarry-ledger-r1`. The lookup path is intentionally
split across the public library, index, segment store and cache modules so a
repository investigation can identify real call sites rather than infer
relationships from names alone.

Run `python tools/symbol_inventory.py --root . --revision quarry-ledger-r1`
from this directory to produce the deterministic broad inventory used by the
case. The command reads only `.rs` files under `src/` and hashes those exact
source files. It does not depend on network access, Git metadata, generated
files, or environment state.
