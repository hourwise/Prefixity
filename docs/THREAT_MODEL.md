# Threat model

Phase 0 is an **offline analysis harness**: it reads trace files and never
touches the network or live requests. The threats below are about the data
it processes and the code that processes it.

## Data-sensitivity threats

### Traces containing API keys
Trace files may accidentally capture prompts that contain secrets. Phase 0
mitigations:

- Fixtures contain **no real credentials** (see `fixtures/traces/README.md`).
- Trace blocks may omit content entirely; hashes and metadata suffice.
- The harness never logs content; human output truncates and sanitises.
- `.env` files are gitignored; keys never enter fixtures.

### Traces containing source code
Private source code may appear in `file_content` blocks. Phase 0 treats all
trace content as potentially sensitive:

- Content is never written back to disk by the harness.
- Displayed content is sanitised and truncated.
- The trace format makes content optional so recorders can log hashes only.

### Cross-project leakage
A trace recorded for project A could be analysed while the user works in
project B. Phase 0 has no storage, so nothing persists across runs. A future
phase must never index or store raw prompt content; derived state must be
disposable and rebuildable (source-of-truth principle 3).

## Malicious-input threats

### Malicious trace files
Trace files are untrusted input. The harness:

- never executes anything from a trace;
- treats all strings as data and sanitises control characters before
  terminal display (`terminal::sanitize_for_terminal`);
- validates structure before analysis and returns structured errors instead
  of panicking.

### Extremely large trace files
- The CLI rejects files above `MAX_TRACE_FILE_BYTES` (256 MiB) before
  reading them into memory.
- Validation rejects traces above `MAX_BLOCKS` (100,000 blocks), oversized
  block content, oversized IDs, and oversized metadata maps.
- `serde_json` applies a default recursion depth limit, bounding nested
  structure.

### Denial-of-service through huge JSON structures
Bounded by the limits above plus the file-size cap. All limits are
documented constants in `prefixity-core::limits`.

### Path traversal / symlinks
Phase 0 takes file paths from the command line and only reads them. It
never writes derived files, so path traversal and symlink attacks have no
write target. A future phase that adds storage must re-evaluate this.

### Terminal escape/control sequences in displayed content
Every string from a trace passes through `sanitize_for_terminal` before
human output. Control characters (other than `\n`/`\t`) are replaced, so
escape-sequence injection into the terminal is prevented.

### Untrusted provider metadata
`ProviderUsage::provider_raw` and `metadata` are preserved verbatim as JSON
for forward compatibility. They are treated as untrusted data: never
interpreted, only displayed sanitised (or emitted as JSON).

## Privacy stance

Phase 0 **avoids storing complete prompt content wherever hashes and
metadata are sufficient**. The trace format supports contentless blocks and
the fixtures exercise that path. If content is present, validation recomputes
its hash and rejects mismatches, so content that is stored is at least
self-checking.

## Future-phase notes

- Any Phase 0B live-testing harness must ensure API keys never enter
  fixtures, are never logged, and are never committed; `.env` files are
  gitignored.
- Any storage added later must be disposable and rebuildable (never
  authoritative — source-of-truth principles 1–3).
