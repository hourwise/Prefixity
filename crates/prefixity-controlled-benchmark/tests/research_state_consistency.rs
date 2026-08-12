use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

const CURRENT_STATE_BEGIN: &str = "<!-- PREFIXITY_CURRENT_STATE_BEGIN -->";
const CURRENT_STATE_END: &str = "<!-- PREFIXITY_CURRENT_STATE_END -->";
const MAX_CURRENT_STATE_BYTES: usize = 4 * 1024;
const EXPECTED_CHECKPOINT_ID: &str = "phase-1c-research-state-v1";
const EXPECTED_EXTERNAL_FRONT_HALF: &str = "EXTERNAL_TRAJECTORY_PERMISSION_PENDING";
const EXPECTED_ARTIFACT_ADMISSION_SCHEMA: &str = "prefixity.external-artifact-admission.v1";

const REQUIRED_FIELDS: [&str; 10] = [
    "checkpoint_id",
    "workspace_crates",
    "workspace_members",
    "phase_1c_stage_0",
    "phase_1c_stage_1",
    "phase_1c_live_replay",
    "external_front_half",
    "controlled_policy_name",
    "controlled_policy_scope",
    "artifact_admission_schema",
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct CurrentState {
    checkpoint_id: String,
    workspace_crates: usize,
    workspace_members: Vec<String>,
    phase_1c_stage_0: String,
    phase_1c_stage_1: String,
    phase_1c_live_replay: String,
    external_front_half: String,
    controlled_policy_name: String,
    controlled_policy_scope: String,
    artifact_admission_schema: String,
}

#[derive(Debug, Clone)]
struct RepositoryInputs {
    cargo_manifest: String,
    readme: String,
    source_of_truth: String,
    docs_index: String,
    artifact_admission_contract: String,
}

fn read_current_repository() -> RepositoryInputs {
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |relative: &str| {
        fs::read_to_string(repository_root.join(relative))
            .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"))
    };

    RepositoryInputs {
        cargo_manifest: read("Cargo.toml"),
        readme: read("README.md"),
        source_of_truth: read("docs/SOURCE_OF_TRUTH.md"),
        docs_index: read("docs/INDEX.md"),
        artifact_admission_contract: read("docs/phase-1/EXTERNAL_ARTIFACT_ADMISSION_CONTRACT.md"),
    }
}

fn parse_workspace_members(cargo_manifest: &str) -> Result<Vec<String>, String> {
    if cargo_manifest.len() > 64 * 1024 {
        return Err("Cargo.toml exceeds the bounded consistency-check input size".to_owned());
    }

    let mut in_workspace = false;
    let mut in_members = false;
    let mut members = Vec::new();
    let mut seen = BTreeSet::new();

    for (line_index, raw_line) in cargo_manifest.lines().enumerate() {
        let line = raw_line.trim();

        if line == "[workspace]" {
            in_workspace = true;
            continue;
        }

        if line.starts_with('[') {
            if in_members {
                return Err(format!(
                    "Cargo.toml workspace members list is not closed before line {}",
                    line_index + 1
                ));
            }
            in_workspace = false;
            continue;
        }

        if !in_workspace {
            continue;
        }

        if !in_members && line == "members = [" {
            in_members = true;
            continue;
        }

        if !in_members {
            continue;
        }

        if line == "]" {
            in_members = false;
            break;
        }

        let entry = line.strip_suffix(',').ok_or_else(|| {
            format!(
                "Cargo.toml workspace member on line {} must end with a comma",
                line_index + 1
            )
        })?;
        let member = entry
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'));
        let Some(member) = member else {
            return Err(format!(
                "Cargo.toml workspace member on line {} must be a quoted path",
                line_index + 1
            ));
        };
        if member.is_empty() || !seen.insert(member.to_owned()) {
            return Err(format!(
                "Cargo.toml workspace member on line {} is empty or duplicated",
                line_index + 1
            ));
        }
        members.push(member.to_owned());
    }

    if in_members {
        return Err("Cargo.toml workspace members list is not closed".to_owned());
    }
    if members.is_empty() {
        return Err("Cargo.toml does not expose a non-empty workspace members list".to_owned());
    }

    Ok(members)
}

fn parse_csv_field(field: &str, name: &str) -> Result<Vec<String>, String> {
    let mut values = Vec::new();
    let mut seen = BTreeSet::new();

    for value in field.split(',') {
        if value.is_empty() || !seen.insert(value.to_owned()) {
            return Err(format!(
                "current-state field `{name}` contains an empty or duplicate value"
            ));
        }
        values.push(value.to_owned());
    }

    Ok(values)
}

fn parse_current_state(document: &str) -> Result<CurrentState, String> {
    if document.matches(CURRENT_STATE_BEGIN).count() != 1 {
        return Err(
            "SOURCE_OF_TRUTH must contain exactly one current-state begin marker".to_owned(),
        );
    }
    if document.matches(CURRENT_STATE_END).count() != 1 {
        return Err("SOURCE_OF_TRUTH must contain exactly one current-state end marker".to_owned());
    }

    let begin = document
        .find(CURRENT_STATE_BEGIN)
        .expect("validated current-state begin marker");
    let body_start = begin + CURRENT_STATE_BEGIN.len();
    let body_end = document[body_start..]
        .find(CURRENT_STATE_END)
        .map(|offset| body_start + offset)
        .ok_or_else(|| "SOURCE_OF_TRUTH current-state marker is not closed".to_owned())?;
    let body = document[body_start..body_end].trim();
    if body.len() > MAX_CURRENT_STATE_BYTES {
        return Err(
            "SOURCE_OF_TRUTH current-state section exceeds its bounded input size".to_owned(),
        );
    }

    let mut fields = BTreeMap::new();
    for (line_index, raw_line) in body.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            return Err(format!(
                "SOURCE_OF_TRUTH current-state field on line {} is empty",
                line_index + 1
            ));
        }

        let (name, value) = line.split_once('=').ok_or_else(|| {
            format!(
                "SOURCE_OF_TRUTH current-state line {} must use key = value syntax",
                line_index + 1
            )
        })?;
        let name = name.trim();
        let value = value.trim();
        if !REQUIRED_FIELDS.contains(&name) {
            return Err(format!(
                "SOURCE_OF_TRUTH current-state field `{name}` is unknown"
            ));
        }
        if value.is_empty() || value.chars().any(char::is_whitespace) {
            return Err(format!(
                "SOURCE_OF_TRUTH current-state field `{name}` has an invalid value"
            ));
        }
        if fields.insert(name.to_owned(), value.to_owned()).is_some() {
            return Err(format!(
                "SOURCE_OF_TRUTH current-state field `{name}` is duplicated"
            ));
        }
    }

    for required_field in REQUIRED_FIELDS {
        if !fields.contains_key(required_field) {
            return Err(format!(
                "SOURCE_OF_TRUTH current-state field `{required_field}` is missing"
            ));
        }
    }

    let field = |name: &str| {
        fields
            .get(name)
            .cloned()
            .expect("required current-state field was checked")
    };
    let workspace_crates = field("workspace_crates").parse::<usize>().map_err(|_| {
        "SOURCE_OF_TRUTH current-state field `workspace_crates` must be an integer".to_owned()
    })?;

    Ok(CurrentState {
        checkpoint_id: field("checkpoint_id"),
        workspace_crates,
        workspace_members: parse_csv_field(&field("workspace_members"), "workspace_members")?,
        phase_1c_stage_0: field("phase_1c_stage_0"),
        phase_1c_stage_1: field("phase_1c_stage_1"),
        phase_1c_live_replay: field("phase_1c_live_replay"),
        external_front_half: field("external_front_half"),
        controlled_policy_name: field("controlled_policy_name"),
        controlled_policy_scope: field("controlled_policy_scope"),
        artifact_admission_schema: field("artifact_admission_schema"),
    })
}

fn bounded_section<'a>(document: &'a str, heading: &str) -> Result<&'a str, String> {
    let start = document
        .find(heading)
        .ok_or_else(|| format!("document is missing bounded section `{heading}`"))?;
    let section = &document[start..];
    let end = section
        .find("\n## ")
        .map(|offset| offset + 1)
        .unwrap_or(section.len());
    Ok(&section[..end])
}

fn expect_marker_value(name: &str, actual: &str, expected: &str) -> Result<(), String> {
    if actual == expected {
        return Ok(());
    }
    Err(format!(
        "SOURCE_OF_TRUTH declares {name} = {actual}, but the required state is {expected}."
    ))
}

fn validate_current_state(inputs: &RepositoryInputs) -> Result<(), String> {
    let cargo_members = parse_workspace_members(&inputs.cargo_manifest)?;
    let state = parse_current_state(&inputs.source_of_truth)?;

    if state.workspace_crates != cargo_members.len() {
        return Err(format!(
            "SOURCE_OF_TRUTH declares workspace_crates = {}, but Cargo.toml contains {} workspace members",
            state.workspace_crates,
            cargo_members.len()
        ));
    }
    if state.workspace_members != cargo_members {
        return Err(format!(
            "SOURCE_OF_TRUTH workspace_members = {:?}, but Cargo.toml declares {:?}",
            state.workspace_members, cargo_members
        ));
    }

    let layout = bounded_section(&inputs.readme, "## Repository layout")?;
    for member in &cargo_members {
        let display_name = member.strip_prefix("crates/").unwrap_or(member);
        if !layout.contains(display_name) {
            return Err(format!(
                "README repository layout is missing workspace member `{display_name}`"
            ));
        }
    }

    let status = bounded_section(&inputs.readme, "## Status")?;
    if !status.contains("Stage 1 is currently blocked") {
        return Err(
            "README status must state that Phase 1C Stage 1 is currently blocked".to_owned(),
        );
    }

    expect_marker_value(
        "checkpoint_id",
        &state.checkpoint_id,
        EXPECTED_CHECKPOINT_ID,
    )?;
    expect_marker_value("phase_1c_stage_0", &state.phase_1c_stage_0, "CERTIFIED")?;
    expect_marker_value("phase_1c_stage_1", &state.phase_1c_stage_1, "BLOCKED")?;
    expect_marker_value(
        "phase_1c_live_replay",
        &state.phase_1c_live_replay,
        "NOT_STARTED",
    )?;
    expect_marker_value(
        "external_front_half",
        &state.external_front_half,
        EXPECTED_EXTERNAL_FRONT_HALF,
    )?;
    expect_marker_value(
        "controlled_policy_name",
        &state.controlled_policy_name,
        "controlled-evidence-policy-v1",
    )?;
    expect_marker_value(
        "controlled_policy_scope",
        &state.controlled_policy_scope,
        "CONTROLLED_ONLY",
    )?;
    expect_marker_value(
        "artifact_admission_schema",
        &state.artifact_admission_schema,
        EXPECTED_ARTIFACT_ADMISSION_SCHEMA,
    )?;

    if !inputs
        .docs_index
        .contains("phase-1/EXTERNAL_ARTIFACT_ADMISSION_CONTRACT.md")
    {
        return Err(
            "docs/INDEX.md does not reference the external artifact admission contract".to_owned(),
        );
    }
    if !inputs
        .artifact_admission_contract
        .contains(EXPECTED_ARTIFACT_ADMISSION_SCHEMA)
    {
        return Err(format!(
            "external artifact admission contract is missing schema `{EXPECTED_ARTIFACT_ADMISSION_SCHEMA}`"
        ));
    }

    Ok(())
}

fn assert_error_contains(inputs: &RepositoryInputs, expected: &str) {
    let error = validate_current_state(inputs).expect_err("synthetic drift should be rejected");
    assert!(
        error.contains(expected),
        "expected error containing {expected:?}, got {error:?}"
    );
}

fn replace_once(value: &mut String, old: &str, new: &str) {
    let replaced = value.replacen(old, new, 1);
    assert_ne!(
        replaced.as_str(),
        value.as_str(),
        "test fixture replacement did not match"
    );
    *value = replaced;
}

#[test]
fn current_repository_state_passes() {
    validate_current_state(&read_current_repository()).expect("current state should be consistent");
}

#[test]
fn missing_workspace_member_is_detected() {
    let mut inputs = read_current_repository();
    replace_once(
        &mut inputs.source_of_truth,
        ",crates/prefixity-controlled-benchmark",
        "",
    );
    assert_error_contains(&inputs, "workspace_members");
}

#[test]
fn extra_workspace_member_is_detected_from_cargo() {
    let mut inputs = read_current_repository();
    replace_once(
        &mut inputs.cargo_manifest,
        "    \"crates/prefixity-controlled-benchmark\",\n",
        "    \"crates/prefixity-controlled-benchmark\",\n    \"crates/fabricated\",\n",
    );
    assert_error_contains(&inputs, "workspace_crates");
}

#[test]
fn authorized_stage_1_state_is_rejected() {
    let mut inputs = read_current_repository();
    replace_once(
        &mut inputs.source_of_truth,
        "phase_1c_stage_1 = BLOCKED",
        "phase_1c_stage_1 = AUTHORIZED",
    );
    assert_error_contains(&inputs, "phase_1c_stage_1 = AUTHORIZED");
}

#[test]
fn unimplemented_stage_0_state_is_rejected() {
    let mut inputs = read_current_repository();
    replace_once(
        &mut inputs.source_of_truth,
        "phase_1c_stage_0 = CERTIFIED",
        "phase_1c_stage_0 = UNIMPLEMENTED",
    );
    assert_error_contains(&inputs, "phase_1c_stage_0 = UNIMPLEMENTED");
}

#[test]
fn completed_stage_1_or_live_replay_state_is_rejected() {
    let mut inputs = read_current_repository();
    replace_once(
        &mut inputs.source_of_truth,
        "phase_1c_stage_1 = BLOCKED",
        "phase_1c_stage_1 = COMPLETED",
    );
    assert_error_contains(&inputs, "phase_1c_stage_1 = COMPLETED");

    let mut inputs = read_current_repository();
    replace_once(
        &mut inputs.source_of_truth,
        "phase_1c_live_replay = NOT_STARTED",
        "phase_1c_live_replay = SUCCESS",
    );
    assert_error_contains(&inputs, "phase_1c_live_replay = SUCCESS");
}

#[test]
fn controlled_policy_scope_promotion_is_rejected() {
    let mut inputs = read_current_repository();
    replace_once(
        &mut inputs.source_of_truth,
        "controlled_policy_scope = CONTROLLED_ONLY",
        "controlled_policy_scope = PRODUCTION",
    );
    assert_error_contains(&inputs, "controlled_policy_scope = PRODUCTION");
}

#[test]
fn external_front_half_blocker_must_remain_pending() {
    let mut inputs = read_current_repository();
    replace_once(
        &mut inputs.source_of_truth,
        "external_front_half = EXTERNAL_TRAJECTORY_PERMISSION_PENDING",
        "external_front_half = ADMITTED",
    );
    assert_error_contains(&inputs, "external_front_half = ADMITTED");
}

#[test]
fn artifact_admission_schema_marker_must_match_contract() {
    let mut inputs = read_current_repository();
    replace_once(
        &mut inputs.source_of_truth,
        "artifact_admission_schema = prefixity.external-artifact-admission.v1",
        "artifact_admission_schema = prefixity.external-artifact-admission.v0",
    );
    assert_error_contains(&inputs, "artifact_admission_schema");
}

#[test]
fn malformed_current_state_marker_fails_cleanly() {
    let mut inputs = read_current_repository();
    replace_once(
        &mut inputs.source_of_truth,
        "phase_1c_stage_1 = BLOCKED",
        "phase_1c_stage_1 BLOCKED",
    );
    assert_error_contains(&inputs, "must use key = value syntax");
}

#[test]
fn duplicate_or_unknown_marker_fields_fail_cleanly() {
    let mut inputs = read_current_repository();
    replace_once(
        &mut inputs.source_of_truth,
        "phase_1c_stage_1 = BLOCKED\n",
        "phase_1c_stage_1 = BLOCKED\nphase_1c_stage_1 = BLOCKED\n",
    );
    assert_error_contains(&inputs, "phase_1c_stage_1` is duplicated");

    let mut inputs = read_current_repository();
    replace_once(
        &mut inputs.source_of_truth,
        "phase_1c_stage_1 = BLOCKED\n",
        "unknown_field = VALUE\nphase_1c_stage_1 = BLOCKED\n",
    );
    assert_error_contains(&inputs, "unknown");
}

#[test]
fn historical_documents_are_not_considered_current_state() {
    let inputs = read_current_repository();
    let historical_document =
        "Earlier design text: Phase 1C Stage 1 was AUTHORIZED in a superseded plan.";
    assert!(historical_document.contains("AUTHORIZED"));
    validate_current_state(&inputs)
        .expect("historical wording outside the explicit current-state inputs must not matter");
}
