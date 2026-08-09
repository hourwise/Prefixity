# Phase 1B.3 Raw Artifact Access and Upstream Schema Verification

Status: complete; assessment `PASS WITH RECORDED LIMITATIONS`.

This gate inspected only `NJU-LINK/CodeTraceBench` at revision
`aa213b84ffb6690fc37ca15766d6ca174ec36d4d`, split `verified`, and only the 24
trajectory identities already accepted by Phase 1A. It did not modify the
importer, the Phase 1B planner, the decision contract, prompts, traces,
evaluation rules or provider configuration. No live provider/model calls,
replay, mutation or Phase 1C work occurred.

## Access result

Exact raw artifacts were successfully accessed. The exact pinned metadata,
README, verified manifest and all 24 selected `bench_artifacts/full/*.tar.zst`
archives were fetched read-only into the system temporary directory
`prefixity-phase1b3-raw-access`, outside the repository. Extracted files and
archives remain local-only and are not tracked. No ignore rule was needed.

The accepted Phase 1A fixture identity was checked before raw inspection:

| Item | Verified value |
| --- | --- |
| Corpus | `NJU-LINK/CodeTraceBench` |
| Revision | `aa213b84ffb6690fc37ca15766d6ca174ec36d4d` |
| Split | `verified` |
| Accepted fixture | `fixtures/phase-1a/codetracebench-mini-swe-v1` |
| Accepted trajectories | 24 |
| Accepted request traces | 719 |
| Accepted source events | 1,498 |
| Raw archive source locator | `https://huggingface.co/datasets/NJU-LINK/CodeTraceBench/resolve/aa213b84ffb6690fc37ca15766d6ca174ec36d4d/<artifact_path>` |

The exact-revision source hashes obtained locally are:

| Source | Bytes | Locally computed SHA-256 |
| --- | ---: | --- |
| Revision metadata API response | 2,370,836 | `31249bc6ea92ad6591ee214fcd7c2919f4e5e582efc065e606f0573cb148e50c` |
| `README.md` | 7,551 | `040b8eeecea41bf7d1af086aec32f5257e8c1ed67c91ee2993e71b352fb7b762` |
| `bench_manifest.verified.jsonl` | 5,115,695 | `bd02d3dafa145146567e87f7d55b158ef0dd30043d4a4c2c03e631c2056ff92e` |
| Exact-revision root tree API response | 2,869 | `2daaa6b6e2142d22378fbcb4c60d20e7d06cf101b39aeaf043e63dfd471969ca` |

The previously recorded README and manifest hashes in
`corpus-provenance.json` were malformed/truncated (49 and 62 characters). They
were corrected to the exact 64-character values above. This was a provenance
correction only; the importer and planner were not changed.

## Artifact identity and integrity

The exact metadata response reports dataset ID
`NJU-LINK/CodeTraceBench`, revision SHA
`aa213b84ffb6690fc37ca15766d6ca174ec36d4d`, and `cardData.license = mit`.
The exact manifest contains all 24 accepted `traj_id` values and no missing or
extra selected records. Each selected record has a unique archive path and a
unique source path.

Each downloaded archive was checked against its exact manifest-derived locator,
hashed locally, opened as a zstd-compressed tar archive, and inspected without
printing member contents. Every archive contained exactly one `.traj.json`
trajectory object. The raw trajectory member SHA-256 matched the existing
Phase 1A `source_file_sha256` ledger for all 24 trajectories. The raw member
path also matched the manifest `source_relpath` directory; two upstream layout
forms were observed: `agent-logs/mini.traj.json` and a source-specific
`<trajectory-name>.traj.json`.

The complete sanitized archive inventory is below. Archive hashes are local
hashes of the downloaded `.tar.zst`; trajectory-member hashes are local hashes
of the extracted `.traj.json`. No trajectory text is included.

| Trajectory / archive identity | Archive bytes | Archive SHA-256 | Raw member SHA-256 | Messages / assistant / stages |
| --- | ---: | --- | --- | ---: |
| `miniswe-Anthropic__Claude-Sonnet-4-20250514-Thinking-build-pmars-23413a0e` | 106,870 | `66396b2345f4a503ff83604a6cc418aaa742e5d86e5c7aea9b29ae552740f4bd` | `3a61a64c6d639a2d70162e8bf100f83e3d639480a72938be39a9d61ee17004c9` | 90 / 44 / 6 |
| `miniswe-Anthropic__Claude-Sonnet-4-20250514-Thinking-git-workflow-hack-15e07e43` | 102,947 | `d3ad080c760e7ea34656c37b89e750146da6bcf03b0dde29da720e85647cb34e` | `377c5573e29a0d4120ddba4f487c644c291643d74c31de2e61514d50dc517928` | 48 / 23 / 5 |
| `miniswe-Anthropic__Claude-Sonnet-4-20250514-Thinking-install-windows-xp-edd35096` | 76,046 | `4fb0e7716b53c126046aaf053ca017387c7c12a956f54441490cefd77b1ad9b0` | `49ed4a32f2df16d30bda8d1e4e2f4082a765f63ed74dca91d68bef0e40c8b073` | 44 / 21 / 4 |
| `miniswe-Anthropic__Claude-Sonnet-4-20250514-Thinking-neuron-to-jaxley-conversion-87200f19` | 135,231 | `e1d5e7f14306d1728c26fc3047e8c275994cc7bbf01e534493685f1fb9b4d968` | `9ff64fea7aeccd84615c05810c0997cb89aef5a1d906677c5275a7ace52c7297` | 54 / 26 / 8 |
| `miniswe-Anthropic__Claude-Sonnet-4-20250514-Thinking-path-tracing-79ade2b0` | 134,846 | `866272c576d44e9cd7b6b93ff0a24ceb7029d1c16e6c46abee134fdae3c36906` | `d5765ab5ab59b6178869d202aa3a963f9449eca574da4af1aa02cea1dbea9d1e` | 96 / 47 / 7 |
| `miniswe-Anthropic__Claude-Sonnet-4-20250514-Thinking-reshard-c4-data-f4ff7301` | 130,558 | `62743ea54ed55447daa1f3bf2d45c4a8120648f9980d729a7d725f77661233d7` | `09013b81a7efb46242d775dbeb5fbb8aa5542b26733ab0be28a7322028dbdc02` | 88 / 43 / 8 |
| `miniswe-Anthropic__Claude-Sonnet-4-20250514-Thinking-train-bpe-tokenizer-2df9c860` | 91,414 | `f591f9c3d3350483d9ef241148fe211f2c269608b47b3fc2a9325d40a8d24647` | `2c25f9e9562a931dbd9f6895b74ab268cf75b0a7282f763e2eb8a8d68298e573` | 58 / 28 / 4 |
| `miniswe-Anthropic__Claude-Sonnet-4-20250514-Thinking-vul-flask-4946dda9` | 113,063 | `f364168f0e25a33bb6e62053036a6ae03141de029a3cb3d184f43652942ba018` | `11a35d9969f4c1c3e6b920a265ed624328693228525d01f2424ece748c3862e8` | 74 / 36 / 6 |
| `miniswe-DeepSeek__DeepSeek-V3.2-cartpole-rl-training-74a92a16` | 462,413 | `d29ea7bdfd8b4c59325ca6809dc4c25cb79b9359724f4d1ccc52671292cc3c89` | `d4a9fb98c0eb0bb98a28800ed60b17c524f990d4aedbd90768016cc75ca6003f` | 50 / 24 / 5 |
| `miniswe-DeepSeek__DeepSeek-V3.2-merge-diff-arc-agi-task-cb5aafdd` | 85,595 | `e981b645b2f6e8adc65546310e1c37002cea719553604222d3bb20fc09cf2016` | `4203f8c8f538b161a8d882c1fb362981721703334876f7184d6a3b6855124cf5` | 44 / 21 / 6 |
| `miniswe-DeepSeek__DeepSeek-V3.2-query-optimize-3b3e2297` | 88,077 | `fa8cc915f2a0414428f798bb852e5ae6e7466db15cbd659cb3c90ba5f0087e3a` | `df5a2c50d9407ffddf6f90cc0758ea0fb7c82a7a05f193e93188c82dc10b02e9` | 50 / 24 / 4 |
| `miniswe-DeepSeek__DeepSeek-V3.2-schemelike-metacircular-eval-e73a741e` | 168,256 | `fe4856d5867430886585bac59d68986576dce6cd39b33ac33a45be3a74d5ac19` | `4d9140b8cbaa4818d18ddf938cd8fe82722c72917ca7049710714c664719ddc9` | 100 / 49 / 7 |
| `miniswe-OpenAI__GPT-5-clap-rs__clap-2758-8f87bc0d` | 50,151 | `4ece3ac6de8971b7bd623cbd397b9bc0711ceffd3c49a5da0ac8f95e4ff603f8` | `f42e3d16cdcd2daa209505424b0725d9b70cef1493e6e89b33142662f4c4d670` | 85 / 41 / 4 |
| `miniswe-OpenAI__GPT-5-clap-rs__clap-2895-928269b9` | 32,839 | `16547e6cc19a8d872e92b22450e4bbef9b8f0a311c46f2cd2335f2f81e723803` | `431740f65493febe0591125963c570337cb589175c1a704e8ed1b2b021a14c73` | 51 / 24 / 3 |
| `miniswe-OpenAI__GPT-5-django__django-14999-5d6ca542` | 25,526 | `5ced92e5cd95e0cb722d44b3007be637d06d85f520c89f852f2da7a940647bbd` | `9c33732b1d0e85c417f011e46ac7fc69eec6807fe369356c411c242c8cf48026` | 43 / 20 / 4 |
| `miniswe-OpenAI__GPT-5-facebook__zstd-1733-786102c1` | 38,679 | `13651b937e7917a9468bed1f220441c47fa7ec32938804573c275aac4e803e5e` | `c1fc50fa6919cfcf19b95c833dd218ca55c0aaf90fa871bb2594a370e21b9f99` | 91 / 44 / 6 |
| `miniswe-OpenAI__GPT-5-instance_ansible__ansible-7e1a347695c7987ae56ef1b6919156d9254010ad-v390e508d27db7a51eece36bb6d9698b63a5b638a-f7c2004c` | 26,870 | `2bf3170f0763173b01aa419e8675693282b2f915dc69d971faea7b45adb3a2c1` | `a8865959bebbfbd06b3b7dfe8f936e6c59e484c9625e6d3b10d99cf6cf0d9049` | 47 / 22 / 3 |
| `miniswe-OpenAI__GPT-5-instance_element-hq__element-web-aeabf3b18896ac1eb7ae9757e66ce886120f8309-vnan-dc9947f9` | 44,775 | `b49aef0364ba5265dd6376936a75017d78257094758e63b5243a6413febfaf8f` | `ec984f21519e4af95ab8bf5aa08fccda20b32d4552849c1414c485aa10bb3b7c` | 63 / 30 / 6 |
| `miniswe-OpenAI__GPT-5-instance_internetarchive__openlibrary-bb152d23c004f3d68986877143bb0f83531fe401-ve8c8d62a2b60610a3c4631f5f23ed866bada9818-78872a1e` | 39,255 | `05e890afd544fd98c25ecb08b1b0301b97ce3c7322edb630e104e0f976e9b583` | `6941c8950144a2893da61d3ca269bd129a6cc6ca8128ba8c1be79f2ad8a2b75b` | 63 / 30 / 4 |
| `miniswe-OpenAI__GPT-5-keras-team__keras-19484-2bd0f0db` | 22,429 | `1baff14e1c81b7e2cfeeee24044ad7ec07caeb6d346b369afdd77835975cda21` | `7819bf05d35a61ee9a0595ef5ef7e19535e7501677d45c911365c2e60e73d417` | 43 / 20 / 5 |
| `miniswe-OpenAI__GPT-5-ponylang__ponyc-2205-2a662253` | 44,248 | `ee465407f87dc92146aa5aaa3754c76a0776e426481dc76d55602057d122c3e7` | `ce3cdc8147c0ed1cdea39e9404427ee470b5bff71045c1be2bd5224f51232e51` | 67 / 32 / 5 |
| `miniswe-OpenAI__GPT-5-prettier__prettier-12930-d76c2178` | 24,167 | `e99bb2eac826a6834d7dcb055bb2cec56bb66367808cca237a403a2c369375ee` | `28d37b2f902e3cfe48a70f574370fcbbc08ee92bdf31ce359031745fc6919b68` | 45 / 21 / 4 |
| `miniswe-OpenAI__GPT-5-sphinx-doc__sphinx-8638-0bbaf886` | 39,176 | `4571754ca64a79069f3b73a56ea6fab96d70208bbf5bac5879a617947f54fb53` | `f158d0d0b3d14b56ace1a047e2d0d6d40c4ca09a25e4be26ada6bb5ec550d3ec` | 43 / 20 / 4 |
| `miniswe-OpenAI__GPT-5-sveltejs__svelte-10608-2fe6d098` | 55,971 | `0a5c49b544577a7231574987bee15d11b851dc266c7a51b3c874089d587df411` | `0b60eca33ceca46d2a9dc996531a71424055737ab47c4e48536f591db0103cd7` | 61 / 29 / 6 |

Cross-check totals: archive bytes `2,139,402`; raw messages `1,498`; raw
assistant messages `719`; manifest `step_count` total `719`. All 24 raw-member
hash comparisons passed.

## Inspection coverage and method

The deterministic detailed sample was fixed before interpretation using the
Phase 1B.2 rule: the lexicographically first selected trajectory in each
`solved x short/medium/long` cell, six trajectories total. All 24 accepted
archives were then inspected structurally after the first sample exposed a
provider-specific response shape. No trajectory was selected because it had a
desired field.

Inspection read JSON keys, types, array lengths, role/type vocabularies,
identifier/reference field names, timestamps/order, manifest metadata, archive
member paths, hashes and line-range locators. It did not print or retain raw
prompt, reasoning, assistant, user, command, observation, source-code or tool
output text. Raw `content` fields were treated as opaque strings.

## Structural schema inventory

Coverage notation is `present / inspected trajectories` for trajectory-level
fields and `present / inspected objects` for message/response-level fields.

### Raw trajectory object

| Field path | Type and coverage | Structural finding | Evidence classification |
| --- | --- | --- | --- |
| `$` | object, 24/24 | Raw trajectory root | `CAPTURED_EXPLICIT` |
| `$.info` | object, 24/24 | Run metadata container | `CAPTURED_EXPLICIT` |
| `$.messages` | array, 24/24; 43–100 entries per trajectory | Ordered raw message/event container; 1,498 entries total | `CAPTURED_EXPLICIT` |
| `$.trajectory_format` | string, 24/24; protocol value `mini-swe-agent-1` | Format identifier | `CAPTURED_EXPLICIT` |
| `$.instance_id` | string, 12/24 | Optional run/instance identity; not a stage or step ID | `CAPTURED_EXPLICIT` |
| `$.info.config` | object, 24/24 | Agent, environment and model configuration metadata | `CAPTURED_EXPLICIT` |
| `$.info.exit_status` | string, 24/24 | Run outcome/status metadata | `CAPTURED_EXPLICIT`; not requiredness |
| `$.info.mini_version` | string, 24/24 | Agent version metadata | `CAPTURED_EXPLICIT` |
| `$.info.model_stats` | object, 24/24 | Contains `api_calls` and `instance_cost` fields | `CAPTURED_EXPLICIT`; cost semantics not normalized |
| `$.info.submission` | string, 24/24 | Submission/outcome metadata; content not retained in evidence | `CAPTURED_EXPLICIT` field presence only |
| `$.info.docker_config` | object, 8/24 | Optional environment metadata | `CAPTURED_EXPLICIT` field presence only |
| `$.info.patch_context_data` | object, 3/24 | Optional evaluation/run metadata | `EVALUATION_ONLY` / provenance metadata; not planner input |

`info.config` includes explicit agent format/template metadata, a model object,
and an environment object. A credential-shaped `info.config.model.api_key`
field is present in 13/24 inspected trajectories, and
`info.config.environment.env` is present in 24/24. Values were never read or
printed. These raw fields make the privacy boundary operationally important;
future evidence must preserve names/types or hashes only and must never copy
configuration values.

### Messages and response envelopes

| Field path | Type and coverage | Structural finding | Evidence classification |
| --- | --- | --- | --- |
| `$.messages[]` | object, 1,498/1,498 | One raw message object per array entry | `CAPTURED_EXPLICIT` |
| `$.messages[].role` | string, 1,498/1,498; `system`, `user`, `assistant` | Explicit protocol role; no `tool` role observed | `CAPTURED_EXPLICIT` |
| `$.messages[].content` | string, 1,498/1,498 | Opaque content; no structural protocol assignment from text | `CAPTURED_EXPLICIT` field presence only |
| `$.messages[].timestamp` | number/float, 1,498/1,498 | Explicit event timestamp; unique and non-decreasing within all 24 trajectories | `CAPTURED_EXPLICIT` |
| `$.messages[].extra` | object, 719/1,498 | Present only on assistant messages | `CAPTURED_EXPLICIT` |
| `$.messages[].extra.response` | object, 719/719 assistant messages | Embedded provider response envelope | `CAPTURED_EXPLICIT` |
| `...response.id` | string, 719/719; globally unique across inspected responses | Provider response identity; not a raw message ID, action ID or step ID | `CAPTURED_EXPLICIT` |
| `...response.model` | string, 719/719 | Provider/model response identity | `CAPTURED_EXPLICIT` |
| `...response.created` | integer, 719/719 | Provider response timestamp/metadata | `CAPTURED_EXPLICIT` |
| `...response.object` | string, 719/719 | Provider response object-kind metadata | `CAPTURED_EXPLICIT` |
| `...response.choices` | array, 719/719; exactly one choice each | Provider response choice container | `CAPTURED_EXPLICIT` |
| `...choices[].index` | integer, 719/719 | Choice index | `CAPTURED_EXPLICIT` |
| `...choices[].finish_reason` | string, 719/719 | Provider response outcome metadata | `CAPTURED_EXPLICIT` |
| `...choices[].message.role` | string, 719/719; `assistant` | Explicit response message role | `CAPTURED_EXPLICIT` |
| `...choices[].message.content` | string, 719/719 | Opaque provider response content | `CAPTURED_EXPLICIT` field presence only |
| `...choices[].message.reasoning_content` | string 268, null 70 | Explicit provider response channel where present; no text retained | `CAPTURED_EXPLICIT` field presence/type |
| `...choices[].message.tool_calls` | null, 379/379 occurrences | Schema key exists as a null placeholder; no actual tool-call array was captured | `ABSENT` for usable raw calls |
| `...choices[].message.function_call` | null, 309/309 occurrences | Schema key exists as a null placeholder; no actual function-call object was captured | `ABSENT` for usable raw calls |
| `...choices[].message.annotations` | empty array, 333/333 occurrences | No message annotations with usable event identity | `ABSENT` |
| `...choices[].message.provider_specific_fields` | object, 309/309; observed key shape `refusal` | Provider-specific metadata container; no action/result relation | `CAPTURED_EXPLICIT` field presence only |

The response IDs are explicit and unique, but they identify provider response
objects. They must not be relabelled as message/event IDs, action IDs or
evaluation step IDs.

### Provider usage payload

The raw response usage object is present for all 719 assistant responses. The
following are structural field counts, not a claim that fields have identical
semantics across providers:

| Field path | Coverage / type | Classification |
| --- | --- | --- |
| `...response.usage` | 719/719 objects | `CAPTURED_EXPLICIT` provider response payload |
| `...usage.prompt_tokens` | 719 integers | `CAPTURED_EXPLICIT` |
| `...usage.completion_tokens` | 719 integers | `CAPTURED_EXPLICIT` |
| `...usage.total_tokens` | 719 integers | `CAPTURED_EXPLICIT` |
| `...usage.input_tokens` | 268 integers | `CAPTURED_EXPLICIT`, provider-specific |
| `...usage.output_tokens` | 268 integers | `CAPTURED_EXPLICIT`, provider-specific |
| `...usage.claude_cache_creation_1_h_tokens` | 292 integers | `CAPTURED_EXPLICIT`, provider-specific cache metadata |
| `...usage.claude_cache_creation_5_m_tokens` | 292 integers | `CAPTURED_EXPLICIT`, provider-specific cache metadata |
| `...usage.prompt_tokens_details.cached_tokens` | 601 integers | `CAPTURED_EXPLICIT`, provider-specific detail |
| `...usage.prompt_tokens_details.audio_tokens` | 601 integers | `CAPTURED_EXPLICIT`, provider-specific detail |
| `...usage.prompt_tokens_details.image_tokens` | 268 integers and 309 nulls | `CAPTURED_EXPLICIT` field shape |
| `...usage.prompt_tokens_details.text_tokens` | 268 integers and 309 nulls | `CAPTURED_EXPLICIT` field shape |
| `...usage.completion_tokens_details.reasoning_tokens` | 603 integers | `CAPTURED_EXPLICIT`, provider-specific detail |
| `...usage.completion_tokens_details.audio_tokens` | 601 integers | `CAPTURED_EXPLICIT`, provider-specific detail |
| `...usage.completion_tokens_details.accepted_prediction_tokens` | 333 integers | `CAPTURED_EXPLICIT`, provider-specific detail |
| `...usage.completion_tokens_details.rejected_prediction_tokens` | 333 integers | `CAPTURED_EXPLICIT`, provider-specific detail |

`info.model_stats.api_calls` and `info.model_stats.instance_cost` are also
explicit fields, but the raw schema does not establish that `instance_cost`
is a provider-reported price with a stable currency/price contract. Future
usage preservation must retain the provider-specific raw usage schema and must
not combine token fields by name alone.

## Evidence matrix

| Desired Prefixity field/fact | Exact raw source | Classification | Safe interpretation |
| --- | --- | --- | --- |
| Corpus/revision/artifact identity | Exact metadata, manifest, archive locator, accepted selection | `CAPTURED_EXPLICIT` | Preserve as provenance and verify before use |
| Trajectory/task/model identity | Manifest `traj_id`, task fields, model; archive path | `CAPTURED_EXPLICIT` | Preserve provenance; do not expose raw content |
| Raw trajectory format | `trajectory_format` | `CAPTURED_EXPLICIT` | Preserve format identifier |
| Stage ID | Manifest `stages[].stage_id` | `EVALUATION_ONLY` | Keep outside planner input |
| Stage boundaries | Manifest `stages[].start_step_id` / `end_step_id` | `EVALUATION_ONLY` | Evaluation structure only |
| Step ID | Manifest `step_id` within annotation records | `EVALUATION_ONLY` | No raw message field; keep labels external |
| Raw message/event ID | No `messages[].id` or equivalent | `ABSENT` | Do not promote generated `message-####` to source identity |
| Provider response ID | `messages[].extra.response.id` | `CAPTURED_EXPLICIT` | Preserve as response identity, not message/action identity |
| Message order | `messages[]` array order | `CAPTURED_EXPLICIT` | Array order is explicit chronology/order |
| Message timestamp | `messages[].timestamp` | `CAPTURED_EXPLICIT` | Preserve timestamp; age does not imply staleness |
| Generated source event index/path | Message array index and `messages[n]` structural path | `DERIVED_STRUCTURAL` | Deterministic provenance projection |
| Message role | `messages[].role` | `CAPTURED_EXPLICIT` | Use for protocol role only |
| Assistant reasoning channel | Nested `reasoning_content` field | `CAPTURED_EXPLICIT` | Preserve typed field presence; do not retain text |
| Tool-call ID | `tool_calls` key is null wherever present | `ABSENT` | No actual raw tool-call object/ID captured |
| Function-call ID | `function_call` key is null wherever present | `ABSENT` | No actual raw function-call object/ID captured |
| Tool/action name | No non-null structured call object; content is opaque string | `ABSENT` | Do not parse natural-language content as protocol type |
| Observation/result ID | No `tool`, result, observation or result-ID field | `ABSENT` | No raw result identity |
| Action-to-observation reference | No raw call/result/reference field | `ABSENT` | Adjacency is not a link |
| Evaluation source locator | `incorrect_stages[].steps[].action_ref` / `observation_ref` | `EVALUATION_ONLY` | Explicit label-to-source locator, not planner evidence |
| Semantic zone | Explicit role mapped by a fixed adapter rule | `DERIVED_STRUCTURAL` | `system -> system`; `user/assistant -> messages`; no tools zone from raw structure |
| Protocol dependency | No dependency/parent/consumed-output edge | `ABSENT` | Keep protocol protection distinct from dependency graph |
| Semantic/load-bearing dependency | No explicit relation | `ABSENT` for source evidence; any text/topology guess is `INFERRED_UNSAFE` | Do not propose as planner input |
| Required | No raw requiredness field | `ABSENT` | Existing normalized `false` default is unknown, not explicit false |
| Optional | No raw optionality field | `ABSENT` | Do not infer from role, age, output, labels or success |
| Stale/invalidation/supersession | No invalidation, replacement, lifetime or supersession field | `ABSENT` | Timestamp/order alone is not staleness |
| Provider/model usage | Response `usage`, response model, model config | `CAPTURED_EXPLICIT` | Preserve provider-specific raw usage; do not call estimates usage |
| Exact evaluation join | Explicit evaluation refs for only part of labels; structural line-to-message mapping | `DERIVED_STRUCTURAL` plus `EVALUATION_ONLY` | Exact bounded join exists; complete all-label join does not |

## Deterministic structural derivations

The following are deterministic structural projections and are not captured
upstream facts:

1. The raw `messages[]` array index becomes a source-event index and
   `messages[n]` structural path. The accepted Phase 1A ledger records the
   derived source event ID/index and the raw member hash.
2. Array order and the explicit timestamp sequence provide chronology. They do
   not provide requiredness, optionality, dependency or staleness.
3. A fixed role-only mapping can produce coarse Prefixity zones:
   `system` role to `system` zone, and `user`/`assistant` roles to the existing
   `messages` zone. It must not create a `tools` zone from content markers.
4. For evaluation references, the explicit `path`, `line_start` and
   `line_end` locator can be structurally matched to the JSON source member,
   then to exactly one `messages[]` object. This is not a positional guess:
   it uses the explicit source locator and JSON object spans.
5. The generated `message-####` IDs and request/turn IDs in Phase 1A remain
   derived structural IDs. They are not upstream message or step IDs.

No semantic interpretation of prompt, reasoning, assistant, user, command or
observation text was used in these derivations.

## Stage, step and message identity result

The raw trajectory object has no explicit `stage_id`, `step_id`, parent-stage
field or raw message ID. The exact manifest has explicit evaluation `stages`
and annotated `step_id` values, but they are outside the raw trajectory message
objects. Across the accepted selection, manifest `step_count` equals the 719
raw assistant-message count. This is a measured count reconciliation, not
proof that assistant ordinal equals evaluation `step_id`; matching counts are
not used as an identity join.

The raw response envelope has 719 explicit, globally unique response IDs. They
are provider response IDs and do not correspond to the manifest stage/step IDs.

## Tool/action/observation linkage result

No usable raw tool-call/action structure was captured in the 24 inspected
trajectories:

- `tool_calls` appears only as a null field in 379 provider response message
  objects;
- `function_call` appears only as a null field in 309 provider response message
  objects;
- raw `messages[].content` is always a string and must remain opaque;
- no raw `tool` role, result object, observation ID, result ID, call ID,
  originating-call reference or parent action field was found; and
- response IDs identify provider responses, not tool calls.

The evaluation sidecar does contain explicit `action_ref` and `observation_ref`
objects, but these are label metadata with source paths and line ranges, not
raw action/result IDs. They establish a limited evaluation-to-source locator,
not a planner-safe action-to-observation relationship. Sequential adjacency,
role alternation, content markers and chronology are rejected as substitutes.

## Evaluation-step join result

The selected manifest contains 60 labelled step records. Their explicit
reference coverage is:

| Measure | Count |
| --- | ---: |
| Labelled step records | 60 |
| Records with neither `action_ref` nor `observation_ref` | 28 |
| Records with action and observation refs | 31 |
| Records with action ref only | 1 |
| Non-null action refs | 32 |
| Non-null observation refs | 31 |
| Non-null raw-message references | 63 |
| References whose path suffix matches the exact raw member | 63/63 |
| References with valid in-file line ranges | 63/63 |
| References structurally contained by exactly one raw `messages[]` object | 63/63 |

Therefore an exact, non-positional join can be established for the 32
reference-bearing evaluation step records:

```text
evaluation-only step_id
  -> evaluation-only action_ref/observation_ref path + line range
  -> exact pinned raw .traj.json member
  -> one raw messages[] object index (derived structural locator)
  -> accepted Phase 1A source_event_index/source_event_id
  -> normalized block occurrences where that source event is retained
```

This is a partial join. The 28 records without explicit refs, including the
28 labelled OpenAI records in this slice, cannot be mapped to raw messages
without unsafe positional reconstruction. Evaluation labels and source refs
remain evaluation-only and must not enter planner inputs.

## Dependency result

No explicit dependency evidence exists beyond ordinary protocol roles and
chronology. The raw object contains no dependency array, parent ID, consumed
output identifier, graph edge, call-result reference or prerequisite field.
The evaluation source locators are not dependency edges. Requiredness from
system role, textual references, chronology, later use, task success or
annotation labels would be `INFERRED_UNSAFE` and is rejected.

## Required, optional and stale result

`required`, `optional` and `stale` are absent from the raw trajectory schema.
No explicit invalidation, supersession, replacement, lifetime, TTL or version
transition was found. The explicit timestamps are usable chronology metadata
only. Existing normalized boolean defaults must remain unknown rather than
being interpreted as captured false values.

## Provider-usage result

Provider usage evidence exists and is materially richer than the Phase 1A
derivative. Every assistant response has an explicit provider response model
and usage object. The raw usage includes provider-specific prompt,
completion, total, input/output, cached-token and cache-creation fields, plus
provider-specific detail objects. This is actual captured response telemetry,
not the Phase 1A surrogate character/token estimate.

It does not establish a universal tokenizer, current pricing, cache-read
semantics across providers, economic savings or intervention quality. A future
adapter must preserve the raw usage schema and provider identity, then apply
versioned provider-specific normalization without conflating fields by name.

## Semantic-zone and protocol result

The raw protocol structure explicitly exposes only `system`, `user` and
`assistant` roles in the top-level message array. The provider response has an
explicit `reasoning_content` field for some responses, but the existing
Prefixity zone model has no separate reasoning zone. A future adapter may
preserve that field as typed provider-response metadata or map it to a reviewed
non-destructive conversation/other representation; it must not use its text.

The proposed design-only mapping is:

| Raw field/type | Prefixity semantic zone | Classification | Constraint |
| --- | --- | --- | --- |
| `messages[].role = system` | `system` | `DERIVED_STRUCTURAL` | Role-only mapping; protocol protection only |
| `messages[].role = user` | `messages` | `DERIVED_STRUCTURAL` | Chronology/request context; not optionality |
| `messages[].role = assistant` | `messages` | `DERIVED_STRUCTURAL` | Conversation context; not tool identity |
| `reasoning_content` field presence | `other` or typed response metadata pending contract | `CAPTURED_EXPLICIT` raw field; zone mapping not yet accepted | Do not retain or interpret text |
| `tool_calls` / `function_call` non-null object | `tools` only if a future exact artifact supplies one | `ABSENT` in this selection | Current values are null; no tools zone claim |
| `usage`, response IDs, timestamps | No context block zone | `CAPTURED_EXPLICIT` metadata | Keep outside planner context blocks |

## Licence classification

At the exact pinned revision, metadata declares `mit` and the exact primary
README states that the dataset is released under the MIT License and links a
root `LICENSE` file. The exact-revision root tree contains no `LICENSE` entry.
No licence text was copied, reconstructed or substituted from another
revision, repository or corpus. Raw archive metadata did not contain a
separate licence file in the inspected members.

Classification: `METADATA_AND_README_ONLY`.

This is a provenance/redistribution limitation, not a legal conclusion. The
raw artifacts remain local-only and untracked.

## Proposed importer implications

No importer change is made by this task. A separately authorized future
evidence-adapter revision should:

| Future handling | Fields/evidence |
| --- | --- |
| Preserve directly | Exact corpus/revision/archive/member locators and hashes; raw role; raw timestamp; `trajectory_format`; provider response ID; provider/model identity; raw provider usage with its provider schema; explicit response field presence such as `reasoning_content` without content |
| Derive structurally | Message index/`messages[n]`; source-event ID/index; chronology; role-only coarse zones; evaluation ref line-range to raw message index, kept in the evaluation sidecar |
| Keep evaluation-only | `stages[].stage_id`, stage boundaries, `step_id`, solved/outcome labels, incorrect/unuseful labels, `action_ref`/`observation_ref` locators and any annotation metadata |
| Ignore or hash-only | All raw content, prompts, reasoning text, assistant/user text, commands, observations, source-code payloads, environment values, API-key-shaped fields and raw archive extraction contents |
| Leave unknown | Raw message ID, action/tool-call ID, observation/result ID, call-result relation, semantic dependency, required, optional, stale, invalidation, task-quality and safe removability |

The adapter must preserve the distinction between provider response IDs and
message IDs, and between evaluation source locators and planner-safe evidence.
It must not create tool links from adjacency or content parsing.

## Decision-gate answers

1. **Were exact raw artifacts for the pinned revision successfully accessed?**
   Yes. All 24 accepted archive locators resolved at the exact revision and
   were inspected locally.
2. **Were their identities verified strongly enough to use as evidence?** Yes.
   Exact metadata revision, exact manifest membership, accepted selection
   membership, archive locator, archive hash, raw member hash and Phase 1A
   source-file hash all reconcile for 24/24 trajectories.
3. **Does the raw schema contain materially more useful structure than the
   accepted Phase 1A derivative representation?** Yes. It contains explicit
   timestamps, provider response IDs, raw provider usage, provider response
   metadata, typed reasoning-channel presence and evaluation source locators.
   The derivative did not preserve these as raw fields.
4. **Which fields are `CAPTURED_EXPLICIT`?** Exact provenance/manifest identity,
   raw format, roles, timestamps, provider response IDs, response model,
   response status/choice metadata, provider usage fields, model/run metadata,
   and explicit evaluation source-locator fields (the latter evaluation-only).
5. **Which useful facts are only `DERIVED_STRUCTURAL`?** Message array index,
   `messages[n]` path, generated source-event/message IDs, chronology
   projections, coarse role-to-zone mapping and the line-locator-to-message
   index mapping for reference-bearing evaluation labels.
6. **Which desired fields remain `ABSENT`?** Raw message IDs, actual tool/action
   IDs, observation/result IDs, raw call-result references, dependency edges,
   required, optional, stale, invalidation/supersession and complete semantic
   load-bearing labels.
7. **Does explicit tool-call/action -> observation/result linkage exist?** No
   usable raw linkage exists. `tool_calls` and `function_call` are null
   placeholders; no result/reference IDs or tool role exist. Evaluation refs
   provide only a bounded, evaluation-only source locator.
8. **Does explicit dependency evidence exist beyond protocol/tool
   relationships?** No. There are no raw dependency/reference edges, and the
   selected raw data has no actual tool relationship to preserve.
9. **Are `required`, `optional` or `stale` represented explicitly?** No. All
   remain `ABSENT`; timestamps do not establish stale state.
10. **Can an exact evaluation-step join be established without unsafe
    positional inference?** A bounded exact join can: 32/60 labelled step
    records expose 63 explicit path/line refs that each map to one raw message
    object. A complete all-step join cannot: 28 records have no refs, so the
    overall answer is partial, not universal.
11. **Does the raw schema contain provider usage evidence?** Yes. Usage is
    present on all 719 assistant response envelopes, with provider-specific
    input/output/total/cache/detail fields.
12. **What is the exact licence-evidence classification?**
    `METADATA_AND_README_ONLY`.
13. **Is a narrow importer/evidence-model revision now justified?** Yes, for
    provenance-safe preservation of explicit response IDs, timestamps,
    provider usage and evaluation source locators. This does not authorize a
    planner change.
14. **Would such a revision materially exercise at least one currently
    untested Phase 1B planner evidence path?** Yes, it can exercise the
    provider-evidence/usage-presence reporting path and improve exact
    provenance/evaluation coverage. It will not enable safe `PRUNE`/`DEFER` or
    dependency-aware relocation because optional, stale and dependency facts
    remain absent.
15. **Should CodeTraceBench remain the Phase 1B corpus?** Yes, retain this
    bounded pinned corpus for a separately authorized evidence-adapter revision
    and characterization. Do not claim it is sufficient for positive safety
    coverage, and do not broaden the corpus in this task.

## Assessment and recommended next task

Assessment: `PASS WITH RECORDED LIMITATIONS`.

The exact raw schema is accessible and strongly verified. It is materially
richer than the accepted derivative for provider usage, response identity,
timestamps and bounded evaluation-source joining, so a narrow evidence-model
revision is justified. However, it still does not provide planner-safe
required/optional/stale metadata, semantic dependency edges, actual raw
tool-call/result linkage or a complete evaluation join. The result does not
justify weakening the Phase 1B planner, inferring safety metadata, or claiming
positive intervention coverage.

Recommended next task: a separately authorized narrow Phase 1B.4
evidence-adapter revision and recharacterization gate. It should preserve only
hashes/metadata and typed provider usage/response identity, preserve explicit
evaluation source locators in a sidecar, add provenance/privacy/negative-
inference tests, and rerun the frozen planner without changing its rules. Keep
`required`, `optional`, `stale`, dependencies and missing call/result links
unknown. Do not begin that task here.

## Completion record

- Raw-artifact access: complete for all 24 accepted trajectories at the exact
  pinned revision; local-only temp storage; no tracked raw files.
- Revision/artifact identity: exact dataset ID and revision matched metadata;
  selected manifest rows matched the accepted selection 24/24.
- Hash/integrity: metadata, README, manifest and tree response hashes recorded;
  all 24 archive hashes recorded; all 24 raw member hashes matched the Phase
  1A source-event ledger.
- Inspection coverage: all 24 raw trajectory objects; six deterministic
  detailed sample rule retained; 1,498 messages and 719 assistant responses
  structurally counted.
- Schema findings: roles, order, timestamps, response IDs, provider response
  envelopes, provider usage, null call placeholders, metadata objects and
  evaluation source locators recorded above.
- Evidence classifications: `CAPTURED_EXPLICIT`, `DERIVED_STRUCTURAL`,
  `EVALUATION_ONLY`, `INFERRED_UNSAFE` and `ABSENT` applied explicitly.
- Evaluation join: exact bounded reference join for 32/60 labelled step
  records; no unsafe positional completion.
- Licence: `METADATA_AND_README_ONLY`; no missing licence text reconstructed.
- Importer/planner: no importer or Phase 1B planner change made; only the
  malformed prior source hashes in `corpus-provenance.json` were corrected.
- Privacy: no raw content, prompts, reasoning text, assistant/user text, tool
  output, reconstructed conversations, credentials or archives added to the
  repository.
- Checks pending at document creation: compact audit JSON validation,
  tracked-evidence privacy scan, raw-artifact untracked check and
  `git diff --check`.
