use prefixity_controlled_benchmark::{
    apply_claim2_decision, compare_paired_arms, evaluate_claim2_arm, load_claim2_case,
    project_claim2_trace, render_claim2_request, select_claim2_decision, validate_claim2_case,
    Claim2ArmState, Claim2EvaluationKey, Claim2ProjectionMode, Claim2PromptPart, Claim2PromptRole,
    Claim2SlotStatus, RelationType, ResearchInterventionClass, SourceKind,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn case_root(case_id: &str) -> PathBuf {
    repo_root()
        .join("fixtures/claim2")
        .join(case_id.to_ascii_lowercase())
}

fn load(case_id: &str) -> prefixity_controlled_benchmark::LoadedClaim2Case {
    load_claim2_case(&case_root(case_id).join("case.json"))
        .unwrap_or_else(|error| panic!("{case_id} should load: {error}"))
}

fn evaluation_key(case_id: &str) -> Claim2EvaluationKey {
    serde_json::from_slice(&fs::read(case_root(case_id).join("evaluation/key.json")).unwrap())
        .unwrap()
}

fn sha256(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

fn all_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else {
                files.push(path);
            }
        }
    }
    files.sort_by_key(|path| {
        path.strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/")
    });
    files
}

fn verify_sha256_sums(root: &Path) {
    let sums_path = root.join("SHA256SUMS");
    let sums_text = fs::read_to_string(&sums_path).unwrap();
    let expected = sums_text
        .lines()
        .map(|line| {
            let (hash, relative) = line.split_once("  ").expect("two spaces separate checksum");
            (relative.to_string(), hash.to_string())
        })
        .collect::<BTreeMap<_, _>>();
    let actual_files = all_files(root)
        .into_iter()
        .filter(|path| path != &sums_path)
        .collect::<Vec<_>>();
    let actual_names = actual_files
        .iter()
        .map(|path| {
            path.strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(actual_names, expected.keys().cloned().collect());
    for path in actual_files {
        let relative = path
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        assert_eq!(
            sha256(&fs::read(path).unwrap()),
            expected[&relative],
            "{relative}"
        );
    }
}

fn source_tree_digest(root: &Path, exclude_sums: bool) -> String {
    let mut digest = Sha256::new();
    for path in all_files(root)
        .into_iter()
        .filter(|path| !exclude_sums || path.file_name().is_some_and(|name| name != "SHA256SUMS"))
    {
        let relative = path
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        digest.update(relative.as_bytes());
        digest.update([0]);
        digest.update(fs::read(path).unwrap());
        digest.update([0]);
    }
    format!("{:x}", digest.finalize())
}

fn run_synthetic_arm(
    case: &prefixity_controlled_benchmark::LoadedClaim2Case,
    key: &Claim2EvaluationKey,
    mode: Claim2ProjectionMode,
    answer_override: Option<Value>,
) -> (
    Claim2ArmState,
    Vec<prefixity_controlled_benchmark::Claim2RenderedRequest>,
    prefixity_controlled_benchmark::Claim2Evaluation,
) {
    let mut arm = Claim2ArmState::new(case, mode);
    let mut requests = Vec::new();
    for slot in 0..2 {
        let request = render_claim2_request(case, &mut arm).unwrap();
        assert!(!request.dispatchable);
        assert_eq!(request.request_slot, slot as u8 + 1);
        requests.push(request);
        assert!(arm
            .record_output(
                case,
                json!({"action_id": key.expected_action_ids[slot]}).to_string()
            )
            .unwrap()
            .is_none());
    }
    let request = render_claim2_request(case, &mut arm).unwrap();
    assert!(!request.dispatchable);
    assert_eq!(request.request_slot, 3);
    requests.push(request);
    let verify_repeat_evaluation = answer_override.is_none();
    let answer = answer_override.unwrap_or_else(|| key.expected_final_answer.clone());
    let raw = json!({"answer": answer}).to_string();
    let evaluation = arm
        .record_output(case, raw)
        .unwrap()
        .expect("the final synthetic response completes the evaluator");
    if verify_repeat_evaluation {
        assert_eq!(evaluate_claim2_arm(case, &arm).unwrap(), evaluation);
    }
    (arm, requests, evaluation)
}

fn event<'a>(
    case: &'a prefixity_controlled_benchmark::LoadedClaim2Case,
    id: &str,
) -> &'a prefixity_controlled_benchmark::Event {
    case.manifest()
        .planner_input
        .events
        .iter()
        .find(|event| event.event_id == id)
        .unwrap_or_else(|| panic!("missing event {id}"))
}

fn rendered_text(request: &prefixity_controlled_benchmark::Claim2RenderedRequest) -> String {
    request
        .messages
        .iter()
        .map(|message| message.content.as_str())
        .collect::<String>()
}

fn occurrences(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

fn source_message_index(
    request: &prefixity_controlled_benchmark::Claim2RenderedRequest,
    event_id: &str,
) -> usize {
    request
        .messages
        .iter()
        .flat_map(|message| {
            message
                .source_receipt_event_ids
                .iter()
                .chain(&message.source_event_ids)
        })
        .position(|source_id| source_id == event_id)
        .unwrap_or_else(|| panic!("missing rendered source event {event_id}"))
}

fn assert_cumulative_template_order(
    case_id: &str,
    case: &prefixity_controlled_benchmark::LoadedClaim2Case,
) {
    let templates = &case.manifest().request_templates;
    assert_eq!(templates.len(), 3);
    assert_eq!(
        &templates[1].messages[..templates[0].messages.len()],
        templates[0].messages.as_slice(),
        "{case_id}: request 2 must preserve request 1 as an unchanged prefix"
    );
    assert_eq!(
        &templates[2].messages[..templates[1].messages.len()],
        templates[1].messages.as_slice(),
        "{case_id}: request 3 must preserve request 2 as an unchanged prefix"
    );

    let slot2_tail = &templates[1].messages[templates[0].messages.len()..];
    assert_eq!(slot2_tail.len(), 4);
    assert_eq!(slot2_tail[0].role, Claim2PromptRole::Assistant);
    assert_eq!(
        slot2_tail[0].parts,
        vec![Claim2PromptPart::PriorAssistantOutput { request_slot: 1 }]
    );
    assert_eq!(slot2_tail[1].role, Claim2PromptRole::User);
    assert_eq!(
        slot2_tail[1].parts,
        vec![Claim2PromptPart::PriorEnvironmentReceipt { request_slot: 1 }]
    );
    assert_eq!(slot2_tail[2].role, Claim2PromptRole::User);
    assert_eq!(
        slot2_tail[2].parts,
        vec![Claim2PromptPart::EventBody {
            event_id: "e-inventory-original".to_string()
        }]
    );
    assert_eq!(slot2_tail[3].role, Claim2PromptRole::User);
    assert_eq!(
        slot2_tail[3].parts,
        vec![Claim2PromptPart::Asset {
            asset_id: "menu2".to_string()
        }]
    );

    let slot3_tail = &templates[2].messages[templates[1].messages.len()..];
    assert_eq!(slot3_tail[0].role, Claim2PromptRole::Assistant);
    assert_eq!(
        slot3_tail[0].parts,
        vec![Claim2PromptPart::PriorAssistantOutput { request_slot: 2 }]
    );
    assert_eq!(slot3_tail[1].role, Claim2PromptRole::User);
    assert_eq!(
        slot3_tail[1].parts,
        vec![Claim2PromptPart::PriorEnvironmentReceipt { request_slot: 2 }]
    );
    if case_id == "CP02" {
        assert_eq!(slot3_tail.len(), 4);
        assert_eq!(slot3_tail[2].role, Claim2PromptRole::User);
        assert_eq!(
            slot3_tail[2].parts,
            [
                "e-read-service",
                "e-read-index",
                "e-read-store",
                "e-read-cache"
            ]
            .into_iter()
            .map(|event_id| Claim2PromptPart::EventBody {
                event_id: event_id.to_string()
            })
            .collect::<Vec<_>>()
        );
        assert_eq!(slot3_tail[3].role, Claim2PromptRole::User);
    } else {
        assert_eq!(slot3_tail.len(), 3);
        assert_eq!(slot3_tail[2].role, Claim2PromptRole::User);
    }
    let final_message = slot3_tail.last().unwrap();
    assert_eq!(
        final_message.parts,
        vec![
            Claim2PromptPart::EventBody {
                event_id: "e-inventory-copy".to_string()
            },
            Claim2PromptPart::Asset {
                asset_id: "final".to_string()
            }
        ]
    );
}

#[test]
fn cp02_cp03_are_admitted_by_the_exact_frozen_policy_and_prune_only_the_reemission() {
    for case_id in ["CP02", "CP03"] {
        let case = load(case_id);
        validate_claim2_case(&case).unwrap();
        assert_cumulative_template_order(case_id, &case);
        let selection = select_claim2_decision(&case).unwrap();
        assert_eq!(selection.decision.class, ResearchInterventionClass::Prune);
        assert_eq!(selection.decision.rule, "EXACT_DUPLICATE_PRUNE");
        assert_eq!(
            selection.decision.target_event_id.as_deref(),
            Some("e-inventory-copy")
        );
        assert_eq!(selection.candidates.exact_duplicate_prune.len(), 1);
        assert!(selection.candidates.explicit_supersession_defer.is_empty());
        assert!(selection.candidates.same_zone_protocol_relocate.is_empty());
        let target = event(&case, "e-inventory-copy");
        let original = event(&case, "e-inventory-original");
        assert_eq!(
            target.event_type,
            prefixity_controlled_benchmark::EventType::Message
        );
        assert_eq!(target.content_hash, original.content_hash);
        assert_eq!(target.world_state_revision, original.world_state_revision);
        assert_eq!(
            target.actor_role,
            prefixity_controlled_benchmark::ActorRole::User
        );
        let same_state_edges = case
            .manifest()
            .planner_input
            .relations
            .iter()
            .filter(|relation| relation.relation_type == RelationType::SameStateRevision)
            .collect::<Vec<_>>();
        assert_eq!(same_state_edges.len(), 1);
        assert_eq!(same_state_edges[0].from_id, "e-inventory-original");
        assert_eq!(same_state_edges[0].to_id, "e-inventory-copy");
        assert!(case
            .manifest()
            .planner_input
            .events
            .iter()
            .all(|candidate| {
                !candidate.reference_event_ids.iter().any(|reference| {
                    reference == "e-inventory-copy"
                        || reference == target.context_block_id.as_deref().unwrap()
                })
            }));
        assert!(case
            .manifest()
            .planner_input
            .relations
            .iter()
            .all(|relation| {
                !matches!(
                    relation.relation_type,
                    RelationType::DependsOn | RelationType::ProtocolPrecedes
                ) || (relation.from_id != "e-inventory-copy"
                    && relation.to_id != "e-inventory-copy")
            }));
        assert_eq!(
            fs::read(case_root(case_id).join("bodies/inventory.txt")).unwrap(),
            fs::read(case_root(case_id).join("bodies/inventory-reemitted.txt")).unwrap()
        );

        let baseline = project_claim2_trace(&case, Claim2ProjectionMode::Baseline).unwrap();
        let noop = project_claim2_trace(&case, Claim2ProjectionMode::NoOp).unwrap();
        let intervention = project_claim2_trace(&case, Claim2ProjectionMode::Intervention).unwrap();
        assert_eq!(baseline, noop);
        let baseline_ids = baseline
            .events
            .iter()
            .map(|event| event.event_id.as_str())
            .collect::<BTreeSet<_>>();
        let intervention_ids = intervention
            .events
            .iter()
            .map(|event| event.event_id.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            baseline_ids
                .difference(&intervention_ids)
                .copied()
                .collect::<BTreeSet<_>>(),
            BTreeSet::from(["e-inventory-copy"])
        );
        assert_eq!(intervention_ids.difference(&baseline_ids).count(), 0);
        let baseline_relations = baseline
            .relations
            .iter()
            .map(|relation| relation.relation_id.as_str())
            .collect::<BTreeSet<_>>();
        let intervention_relations = intervention
            .relations
            .iter()
            .map(|relation| relation.relation_id.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            baseline_relations
                .difference(&intervention_relations)
                .copied()
                .collect::<BTreeSet<_>>(),
            BTreeSet::from(["same-state-inventory"])
        );
        assert_eq!(
            intervention_relations
                .difference(&baseline_relations)
                .count(),
            0
        );
        for relation_id in [
            "produce-inventory",
            "produce-verification",
            "produce-callsite",
        ] {
            if case_id == "CP02" && relation_id == "produce-verification" {
                continue;
            }
            if case_id == "CP03" && relation_id == "produce-callsite" {
                continue;
            }
            assert!(baseline_relations.contains(relation_id));
            assert!(intervention_relations.contains(relation_id));
        }
        let key = evaluation_key(case_id);
        for event_id in key.required_event_ids.iter().chain(&key.critical_event_ids) {
            assert!(
                baseline_ids.contains(event_id.as_str()),
                "{case_id}: required {event_id}"
            );
            assert!(
                intervention_ids.contains(event_id.as_str()),
                "{case_id}: removed required {event_id}"
            );
        }
        let applied = apply_claim2_decision(&case, Claim2ProjectionMode::Intervention).unwrap();
        assert_eq!(applied.actual_selection, selection);
        assert_eq!(applied.planner_input, intervention);
    }
}

#[test]
fn cp02_cp03_run_arm_local_synthetic_trajectories_with_exact_noop_and_pruned_context() {
    for case_id in ["CP02", "CP03"] {
        let case = load(case_id);
        let key = evaluation_key(case_id);
        let (mut baseline, baseline_requests, baseline_eval) =
            run_synthetic_arm(&case, &key, Claim2ProjectionMode::Baseline, None);
        let (mut noop, noop_requests, noop_eval) =
            run_synthetic_arm(&case, &key, Claim2ProjectionMode::NoOp, None);
        let (mut intervention, intervention_requests, intervention_eval) =
            run_synthetic_arm(&case, &key, Claim2ProjectionMode::Intervention, None);
        for evaluation in [&baseline_eval, &noop_eval, &intervention_eval] {
            assert_eq!(
                evaluation.status,
                Claim2SlotStatus::Pass,
                "{case_id}: {evaluation:?}"
            );
            assert!(evaluation.action_checks_passed);
            assert!(evaluation.intermediate_state_checks_passed);
            assert!(evaluation.receipt_identity_checks_passed);
            assert!(evaluation.final_answer_passed);
            assert!(evaluation.required_context_preserved);
            assert!(evaluation.required_relations_preserved);
            assert!(evaluation.critical_events_preserved);
            assert!(evaluation.structurally_complete);
        }
        assert_eq!(baseline_requests.len(), 3);
        assert_eq!(noop_requests.len(), 3);
        assert_eq!(intervention_requests.len(), 3);
        assert_eq!(
            &baseline_requests[1].messages[..baseline_requests[0].messages.len()],
            baseline_requests[0].messages.as_slice(),
            "{case_id}: rendered request 2 retains request 1 as an unchanged prefix"
        );
        assert_eq!(
            &baseline_requests[2].messages[..baseline_requests[1].messages.len()],
            baseline_requests[1].messages.as_slice(),
            "{case_id}: rendered request 3 retains request 2 as an unchanged prefix"
        );
        assert_eq!(
            &intervention_requests[2].messages[..intervention_requests[1].messages.len()],
            intervention_requests[1].messages.as_slice(),
            "{case_id}: intervention request 3 retains request 2 as an unchanged prefix"
        );
        assert!(
            source_message_index(&baseline_requests[1], "e-result-inventory")
                < source_message_index(&baseline_requests[1], "e-inventory-original")
        );
        let verify_result = if case_id == "CP02" {
            "e-result-callsite"
        } else {
            "e-result-verify"
        };
        assert!(
            source_message_index(&baseline_requests[2], verify_result)
                < source_message_index(&baseline_requests[2], "e-inventory-copy")
        );
        if case_id == "CP02" {
            let reads = [
                "e-read-service",
                "e-read-index",
                "e-read-store",
                "e-read-cache",
            ];
            let read_positions = reads
                .iter()
                .map(|event_id| source_message_index(&baseline_requests[2], event_id))
                .collect::<Vec<_>>();
            assert!(read_positions.windows(2).all(|pair| pair[0] < pair[1]));
            assert!(
                read_positions.last().unwrap()
                    < &source_message_index(&baseline_requests[2], "e-inventory-copy")
            );
        }
        for request_slot in 0..2 {
            assert_eq!(
                baseline_requests[request_slot].messages,
                noop_requests[request_slot].messages
            );
            assert_eq!(
                baseline_requests[request_slot].request_json,
                noop_requests[request_slot].request_json
            );
            assert_eq!(
                baseline_requests[request_slot].request_json,
                intervention_requests[request_slot].request_json
            );
        }
        assert_eq!(baseline_requests[2].messages, noop_requests[2].messages);
        assert_eq!(
            baseline_requests[2].request_json,
            noop_requests[2].request_json
        );
        let inventory =
            fs::read_to_string(case_root(case_id).join("bodies/inventory.txt")).unwrap();
        assert_eq!(
            occurrences(&rendered_text(&baseline_requests[1]), &inventory),
            1
        );
        assert_eq!(
            occurrences(&rendered_text(&baseline_requests[2]), &inventory),
            2
        );
        assert_eq!(
            occurrences(&rendered_text(&intervention_requests[2]), &inventory),
            1
        );
        assert_eq!(
            baseline_requests[2].metrics.omitted_attachment_utf8_bytes,
            0
        );
        assert_eq!(noop_requests[2].metrics.omitted_attachment_utf8_bytes, 0);
        assert_eq!(
            intervention_requests[2]
                .metrics
                .omitted_attachment_utf8_bytes,
            inventory.len()
        );
        assert_eq!(
            baseline_requests[2].metrics.attachment_utf8_bytes
                - intervention_requests[2].metrics.attachment_utf8_bytes,
            inventory.len()
        );
        assert_ne!(
            baseline_requests[2].request_json,
            intervention_requests[2].request_json
        );
        assert_eq!(
            compare_paired_arms(&mut baseline, &mut noop, &mut intervention)
                .unwrap()
                .status,
            Claim2SlotStatus::Pass
        );
        assert_eq!(
            baseline
                .slots()
                .iter()
                .map(|slot| slot.status.clone())
                .collect::<Vec<_>>(),
            vec![Claim2SlotStatus::Pass; 3]
        );
        assert_eq!(
            intervention
                .slots()
                .iter()
                .map(|slot| slot.status.clone())
                .collect::<Vec<_>>(),
            vec![Claim2SlotStatus::Pass; 3]
        );
    }
}

#[test]
fn cp02_cp03_wrong_actions_and_final_facts_fail_without_retries() {
    let cp02 = load("CP02");
    let mut bad_action = Claim2ArmState::new(&cp02, Claim2ProjectionMode::Baseline);
    let first = render_claim2_request(&cp02, &mut bad_action).unwrap();
    assert!(!first.dispatchable);
    let failed = bad_action
        .record_output(&cp02, json!({"action_id":"skip_inventory"}).to_string())
        .unwrap()
        .expect("wrong valid-menu action fails the frozen intermediate oracle");
    assert_eq!(failed.status, Claim2SlotStatus::Fail);
    assert_eq!(bad_action.slots()[0].status, Claim2SlotStatus::Fail);
    assert_eq!(
        bad_action.slots()[1].status,
        Claim2SlotStatus::NotExecutedAfterFailure
    );
    assert_eq!(
        bad_action.slots()[2].status,
        Claim2SlotStatus::NotExecutedAfterFailure
    );
    assert!(render_claim2_request(&cp02, &mut bad_action).is_err());

    let cp03 = load("CP03");
    let key = evaluation_key("CP03");
    let mut wrong_answer = key.expected_final_answer.clone();
    wrong_answer["cache_engine_version"] = json!("5.2.0");
    let (wrong_answer_arm, _, evaluation) = run_synthetic_arm(
        &cp03,
        &key,
        Claim2ProjectionMode::Baseline,
        Some(wrong_answer),
    );
    assert_eq!(evaluation.status, Claim2SlotStatus::Fail);
    assert!(!evaluation.final_answer_passed);
    assert!(evaluation.structurally_complete);
    assert!(evaluation
        .failure_reasons
        .iter()
        .any(|reason| reason == "final_answer_mismatch"));
    assert!(!evaluation
        .failure_reasons
        .iter()
        .any(|reason| reason == "incomplete_arm"));
    let reevaluated = evaluate_claim2_arm(&cp03, &wrong_answer_arm).unwrap();
    assert_eq!(reevaluated.status, Claim2SlotStatus::Fail);
    assert!(!reevaluated.final_answer_passed);
    assert!(reevaluated.structurally_complete);
    assert!(!reevaluated
        .failure_reasons
        .iter()
        .any(|reason| reason == "incomplete_arm"));
}

#[test]
fn cp02_inventory_is_generated_from_the_pinned_repository_and_call_chain_sources() {
    let root = case_root("CP02");
    let snapshot = root.join("snapshot");
    verify_sha256_sums(&snapshot);
    let inventory = fs::read_to_string(root.join("bodies/inventory.txt")).unwrap();
    let source_files = all_files(&snapshot.join("src"))
        .into_iter()
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        .collect::<Vec<_>>();
    assert_eq!(source_files.len(), 21);
    let mut digest = Sha256::new();
    for path in &source_files {
        let relative = path
            .strip_prefix(&snapshot)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        digest.update(relative.as_bytes());
        digest.update([0]);
        digest.update(fs::read(path).unwrap());
        digest.update([0]);
    }
    let source_hash = format!("{:x}", digest.finalize());
    assert!(inventory.contains(&format!("source_files={}", source_files.len())));
    assert!(inventory.contains(&format!("source_sha256={source_hash}")));
    let service = fs::read_to_string(snapshot.join("src/service.rs")).unwrap();
    let index = fs::read_to_string(snapshot.join("src/index.rs")).unwrap();
    let store = fs::read_to_string(snapshot.join("src/store.rs")).unwrap();
    let cache = fs::read_to_string(snapshot.join("src/cache.rs")).unwrap();
    for (path, source, declaration) in [
        ("src/service.rs", service.as_str(), "pub fn find_record"),
        ("src/index.rs", index.as_str(), "pub(crate) fn lookup"),
        ("src/store.rs", store.as_str(), "pub(crate) fn read_segment"),
        ("src/cache.rs", cache.as_str(), "pub(crate) fn get_or_load"),
    ] {
        let line = source
            .lines()
            .position(|line| line.contains(declaration))
            .unwrap()
            + 1;
        let symbol = match path {
            "src/service.rs" => "Library::find_record",
            "src/index.rs" => "IndexSnapshot::lookup",
            "src/store.rs" => "SegmentStore::read_segment",
            _ => "IndexCache::get_or_load",
        };
        assert!(inventory.contains(&format!("{path}:{line}: declaration: {symbol}")));
    }
    assert!(service.contains("self.snapshot.lookup(&self.store, query)?"));
    assert!(index.contains("store.read_segment(&segment_path)?"));
    assert!(store.contains("self.cache.get_or_load(owned_path.clone(), ||"));
    assert!(fs::read(root.join("bodies/direct-service.rs")).unwrap() == service.as_bytes());
    assert!(fs::read(root.join("bodies/direct-index.rs")).unwrap() == index.as_bytes());
    assert!(fs::read(root.join("bodies/direct-store.rs")).unwrap() == store.as_bytes());
    assert!(fs::read(root.join("bodies/direct-cache.rs")).unwrap() == cache.as_bytes());
    let key = evaluation_key("CP02");
    assert_eq!(key.expected_final_answer["entry_path"], "src/service.rs");
    assert_eq!(key.expected_final_answer["index_path"], "src/index.rs");
    assert_eq!(key.expected_final_answer["store_path"], "src/store.rs");
    assert_eq!(key.expected_final_answer["cache_path"], "src/cache.rs");
}

fn package_version(manifest: &str) -> String {
    let mut in_package = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
        } else if in_package && trimmed.starts_with("version = ") {
            return trimmed
                .trim_start_matches("version = ")
                .trim_matches('"')
                .to_string();
        }
    }
    panic!("manifest has no package version")
}

fn cargo_dependency_lines(manifest: &str) -> Vec<(String, String)> {
    let mut in_dependencies = false;
    let mut dependencies = Vec::new();
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_dependencies = trimmed == "[dependencies]";
        } else if in_dependencies && trimmed.contains("= {") {
            let (name, declaration) = trimmed.split_once("= {").unwrap();
            let version = declaration
                .split("version = \"")
                .nth(1)
                .and_then(|tail| tail.split('"').next())
                .expect("path dependency pins a version");
            dependencies.push((name.trim().to_string(), version.to_string()));
        }
    }
    dependencies
}

fn cargo_caret_accepts(requirement: &str, resolved: &str) -> bool {
    let parse = |version: &str| {
        let version = version.trim_start_matches('^');
        let mut parts = version.split('.').map(|part| part.parse::<u64>().unwrap());
        (
            parts.next().unwrap(),
            parts.next().unwrap_or(0),
            parts.next().unwrap_or(0),
        )
    };
    let requested = parse(requirement);
    let found = parse(resolved);
    if found < requested || found.0 != requested.0 {
        return false;
    }
    if requested.0 == 0 {
        if requested.1 == 0 {
            found.1 == requested.1 && found.2 == requested.2
        } else {
            found.1 == requested.1
        }
    } else {
        true
    }
}

#[test]
fn cp03_inventory_and_verification_reconcile_the_actual_pinned_workspace() {
    let root = case_root("CP03");
    let snapshot = root.join("snapshot");
    verify_sha256_sums(&snapshot);
    let actual_tree_hash = source_tree_digest(&snapshot, true);
    let inventory = fs::read_to_string(root.join("bodies/inventory.txt")).unwrap();
    let receipt = fs::read_to_string(root.join("bodies/receipt2.txt")).unwrap();
    assert!(inventory.contains(&format!("snapshot_tree_sha256={actual_tree_hash}")));
    assert!(receipt.contains(&format!("project-tree SHA-256 {actual_tree_hash}")));
    assert!(inventory.contains("reconciliation=compatible"));
    assert!(inventory.contains("lock_package_count=7"));

    let member_paths = [
        "crates/wire-schema",
        "crates/segment-format",
        "crates/snapshot-index",
        "crates/cache-engine",
        "crates/query-router",
        "crates/recovery-journal",
        "crates/audit-cli",
    ];
    let mut packages = BTreeMap::new();
    let mut actual_edges = Vec::new();
    for member in member_paths {
        let manifest = fs::read_to_string(snapshot.join(member).join("Cargo.toml")).unwrap();
        let package_name = manifest
            .lines()
            .find_map(|line| line.trim().strip_prefix("name = \""))
            .and_then(|tail| tail.split('"').next())
            .unwrap()
            .to_string();
        packages.insert(package_name.clone(), package_version(&manifest));
        for (dependency, requirement) in cargo_dependency_lines(&manifest) {
            actual_edges.push((package_name.clone(), dependency, requirement));
        }
    }
    actual_edges.sort();
    let mut answer_edges = evaluation_key("CP03").expected_final_answer["compatible_edges"]
        .as_array()
        .unwrap()
        .iter()
        .map(|edge| {
            let consumer = edge["consumer"].as_str().unwrap().to_string();
            let dependency = edge["dependency"].as_str().unwrap().to_string();
            let requirement = edge["requirement"].as_str().unwrap().to_string();
            let resolved = edge["resolved_version"].as_str().unwrap().to_string();
            assert_eq!(packages[&dependency], resolved);
            assert!(cargo_caret_accepts(&requirement, &resolved));
            assert_eq!(edge["compatible"], true);
            (consumer, dependency, requirement)
        })
        .collect::<Vec<_>>();
    answer_edges.sort();
    assert_eq!(actual_edges, answer_edges);
    for (package, version) in [
        ("wire-schema", "2.4.1"),
        ("cache-engine", "5.3.0"),
        ("audit-cli", "0.7.2"),
    ] {
        assert_eq!(packages[package], version);
        assert_eq!(
            evaluation_key("CP03").expected_final_answer[match package {
                "wire-schema" => "wire_schema_version",
                "cache-engine" => "cache_engine_version",
                _ => "audit_cli_version",
            }],
            version
        );
        assert!(inventory.contains(&format!("{package} {version} locked={version}")));
    }
    let workspace = fs::read_to_string(snapshot.join("Cargo.toml")).unwrap();
    assert!(workspace.contains("rust-version = \"1.78\""));
    assert!(workspace.contains("resolver = \"2\""));
    let toolchain = fs::read_to_string(snapshot.join("rust-toolchain.toml")).unwrap();
    assert!(toolchain.contains("channel = \"1.82.0\""));
    let ci = fs::read_to_string(snapshot.join(".ci/rust-toolchain-matrix.txt")).unwrap();
    let ci_versions = ci
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(ci_versions, vec!["1.78.0", "1.82.0"]);
    let key = evaluation_key("CP03");
    assert_eq!(key.expected_final_answer["snapshot_unchanged"], true);
    assert_eq!(key.expected_final_answer["reconciliation"], "compatible");
    assert!(receipt.contains("All 12 local dependency edges"));
}

#[test]
fn cp02_cp03_hide_evaluation_keys_and_keep_receipts_as_results() {
    for case_id in ["CP02", "CP03"] {
        let root = case_root(case_id);
        let case = load(case_id);
        let manifest_bytes = fs::read(root.join("case.json")).unwrap();
        let manifest_text = String::from_utf8(manifest_bytes).unwrap();
        assert!(!manifest_text.contains("expected_final_answer"));
        assert!(!manifest_text.contains("expected_action_ids"));
        let key_path = root.join("evaluation/key.json");
        let key_bytes = fs::read(&key_path).unwrap();
        let key_hash = sha256(&key_bytes);
        assert!(!case
            .manifest()
            .assets
            .iter()
            .any(|asset| asset.relative_path == "evaluation/key.json" || asset.sha256 == key_hash));
        let key_value: Value = serde_json::from_slice(&key_bytes).unwrap();
        let encoded_answer = serde_json::to_string(&key_value["expected_final_answer"]).unwrap();
        let key_event_results = case
            .manifest()
            .planner_input
            .events
            .iter()
            .filter(|event| event.event_type == prefixity_controlled_benchmark::EventType::Result)
            .collect::<Vec<_>>();
        assert_eq!(key_event_results.len(), 4);
        assert!(key_event_results.iter().all(|event| {
            event.actor_role == prefixity_controlled_benchmark::ActorRole::Tool
                && event.result.is_some()
        }));
        assert!(case
            .manifest()
            .planner_input
            .provenance
            .iter()
            .all(|provenance| provenance.source_kind == SourceKind::SelfAuthored));
        for mode in [
            Claim2ProjectionMode::Baseline,
            Claim2ProjectionMode::NoOp,
            Claim2ProjectionMode::Intervention,
        ] {
            let arm = Claim2ArmState::new(&case, mode);
            let static_request = arm.preview_request(&case, 3).unwrap();
            let request_text = String::from_utf8(static_request.request_json).unwrap();
            assert!(!request_text.contains(&encoded_answer));
        }
    }
}
