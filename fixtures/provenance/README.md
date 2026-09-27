# Immutable historical source provenance

`phase1b9-748e4673.rs.txt` is the exact Git blob from commit
`748e4673e8454d2ac3e27cefabee9259992038aa`, path
`crates/prefixity-controlled-benchmark/src/phase1b9.rs`.
SHA-256: `2f1dae56606815034530b9a7114170eea08b9269eed57b18a4fb5d9747830a33`.

The unchanged h001 source manifest binds that historical revision. Its offline
artifact validation reads this pinned source snapshot rather than confusing
the current policy implementation with the historical revision. The snapshot
is evidence, not a compiled policy fork. Claim-2 uses the current shared
Phase 1B.9 implementation and verifies historical report parity.

No consumed manifest, fingerprint, or gate identity was updated. Live gate
source bindings still check current runtime source files and fail closed after
source changes; this archive does not restore permission to execute an old gate.
