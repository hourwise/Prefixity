//! Policy-simulation integration tests.
//!
//! Focus areas: source traces are never mutated; required blocks are never
//! removed; the already-optimal fixture produces no false recommendation;
//! the compression policy name is reserved; malformed input is rejected.

mod common;

use prefixity_core::cost::default_synthetic_profile;
use prefixity_core::policy::{
    policy_from_name, simulate_policy, BaselinePolicy, CombinedPolicy, DeferVolatilePolicy,
    PruneStaleToolOutputPolicy, StablePrefixPolicy,
};

fn all_policy_names() -> &'static [&'static str] {
    &[
        "baseline",
        "stable-prefix",
        "defer-volatile",
        "prune-stale-tool-output",
        "combined",
    ]
}

#[test]
fn simulation_never_mutates_the_source_trace() {
    for name in [
        "01-stable-prefix.json",
        "06-context-reduction-wins.json",
        "08-unsafe-pruning-example.json",
    ] {
        let trace = common::load_fixture(name);
        let original = trace.clone();
        for policy_name in all_policy_names() {
            let policy = policy_from_name(policy_name).unwrap();
            let _ = simulate_policy(&trace, policy.as_ref(), &default_synthetic_profile()).unwrap();
            assert_eq!(
                trace, original,
                "policy {policy_name} mutated the source trace for {name}"
            );
        }
    }
}

#[test]
fn baseline_reproduces_recorded_structure() {
    let trace = common::load_fixture("06-context-reduction-wins.json");
    let result = simulate_policy(&trace, &BaselinePolicy, &default_synthetic_profile()).unwrap();
    assert_eq!(result.token_difference, 0);
    assert!(result.removed_blocks.is_empty());
    assert!(result.relocated_blocks.is_empty());
    assert_eq!(result.simulated_tokens, result.original_tokens);
}

#[test]
fn stable_prefix_reorders_without_removing() {
    let trace = common::load_fixture("06-context-reduction-wins.json");
    let result =
        simulate_policy(&trace, &StablePrefixPolicy, &default_synthetic_profile()).unwrap();
    assert!(result.removed_blocks.is_empty());
    assert_eq!(result.token_difference, 0);
    assert!(!result.relocated_blocks.is_empty());
    // repo-map (position 4) moves before git-status (position 3).
    assert!(result.relocated_blocks.iter().any(|r| r.id == "repo-map"));
}

#[test]
fn defer_volatile_only_removes_optional_volatile_blocks() {
    let trace = common::load_fixture("06-context-reduction-wins.json");
    let result =
        simulate_policy(&trace, &DeferVolatilePolicy, &default_synthetic_profile()).unwrap();
    let removed_ids: Vec<&str> = result
        .removed_blocks
        .iter()
        .map(|b| b.id.as_str())
        .collect();
    assert!(removed_ids.contains(&"old-session-summary"));
    assert!(removed_ids.contains(&"stale-tool-output"));
    // Non-optional volatile blocks (git-status, user-request) are retained.
    assert!(result.retained_blocks.iter().any(|id| id == "git-status"));
    assert!(result.retained_blocks.iter().any(|id| id == "user-request"));
}

#[test]
fn prune_stale_tool_output_only_removes_stale_tool_results() {
    let trace = common::load_fixture("06-context-reduction-wins.json");
    let result = simulate_policy(
        &trace,
        &PruneStaleToolOutputPolicy,
        &default_synthetic_profile(),
    )
    .unwrap();
    assert_eq!(result.removed_blocks.len(), 1);
    assert_eq!(result.removed_blocks[0].id, "stale-tool-output");
    // old-session-summary is stale but not a tool output: retained.
    assert!(result
        .retained_blocks
        .iter()
        .any(|id| id == "old-session-summary"));
}

#[test]
fn combined_removes_and_reorders() {
    let trace = common::load_fixture("06-context-reduction-wins.json");
    let result = simulate_policy(&trace, &CombinedPolicy, &default_synthetic_profile()).unwrap();
    assert_eq!(result.token_difference, -20000);
    assert_eq!(result.removed_blocks.len(), 2);
    assert_eq!(result.stable_prefix_candidate_difference, 1800);
}

#[test]
fn stable_prefix_relocations_are_labelled_experimental() {
    let trace = common::load_fixture("06-context-reduction-wins.json");
    let result =
        simulate_policy(&trace, &StablePrefixPolicy, &default_synthetic_profile()).unwrap();
    assert!(!result.relocated_blocks.is_empty());
    assert!(result.relocated_blocks.iter().all(|r| matches!(
        r.safety,
        prefixity_core::policy::RelocationSafety::Experimental(_)
    )));
}

#[test]
fn unsafe_cross_zone_reorder_is_deferred() {
    let trace = common::load_fixture("16-global-reorder-would-be-unsafe.json");
    let result =
        simulate_policy(&trace, &StablePrefixPolicy, &default_synthetic_profile()).unwrap();
    // The numerically attractive but semantically unsafe reorder is not applied.
    assert!(result.relocated_blocks.is_empty());
    assert!(!result.unsafe_transformations_deferred.is_empty());
    assert!(result
        .unsafe_transformations_deferred
        .iter()
        .any(|d| d.contains("cross-zone")));
    assert!(result
        .warnings
        .iter()
        .any(|w| w.contains("No safe relocation")));
}

#[test]
fn required_blocks_are_never_removed_by_any_policy() {
    let trace = common::load_fixture("08-unsafe-pruning-example.json");
    for policy_name in all_policy_names() {
        let policy = policy_from_name(policy_name).unwrap();
        let result =
            simulate_policy(&trace, policy.as_ref(), &default_synthetic_profile()).unwrap();
        assert!(
            result
                .retained_blocks
                .iter()
                .any(|id| id == "critical-config"),
            "policy {policy_name} removed a required block"
        );
    }
}

#[test]
fn compression_policy_is_reserved_not_implemented() {
    let err = policy_from_name("compression").unwrap_err();
    assert!(matches!(
        err,
        prefixity_core::PrefixityError::Reserved { .. }
    ));
    assert!(err.to_string().contains("reserved"));
}

#[test]
fn unknown_policy_name_is_rejected() {
    let err = policy_from_name("definitely-not-a-policy").unwrap_err();
    assert!(matches!(
        err,
        prefixity_core::PrefixityError::PolicyNotFound { .. }
    ));
}

#[test]
fn malformed_trace_is_rejected_by_analysis_and_simulation() {
    let mut trace = common::load_fixture("01-stable-prefix.json");
    trace.format_version = 99;
    let analysis_err = prefixity_core::analysis::analyze_trace(&trace, None).unwrap_err();
    assert!(matches!(
        analysis_err,
        prefixity_core::PrefixityError::UnsupportedFormatVersion { .. }
    ));

    let simulate_err =
        simulate_policy(&trace, &BaselinePolicy, &default_synthetic_profile()).unwrap_err();
    assert!(matches!(
        simulate_err,
        prefixity_core::PrefixityError::UnsupportedFormatVersion { .. }
    ));

    let compare_err = prefixity_core::compare::compare_traces(
        &trace,
        &common::load_fixture("01-stable-prefix-turn2.json"),
        None,
    )
    .unwrap_err();
    assert!(matches!(
        compare_err,
        prefixity_core::PrefixityError::UnsupportedFormatVersion { .. }
    ));
}
