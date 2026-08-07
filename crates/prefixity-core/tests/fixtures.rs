//! Fixture-scenario integration tests.
//!
//! Each test maps to one of the eight documented synthetic scenarios and
//! checks the numbers the harness should deterministically produce.

mod common;

use prefixity_core::analysis::analyze_trace;
use prefixity_core::compare::{compare_traces, DiffKind};
use prefixity_core::cost::default_synthetic_profile;
use prefixity_core::model::TRACE_FORMAT_VERSION;
use prefixity_core::validation::validate_trace;

const ALL_FIXTURES: &[&str] = &[
    "01-stable-prefix.json",
    "01-stable-prefix-turn2.json",
    "02-early-timestamp-break.json",
    "02-early-timestamp-break-turn2.json",
    "03-tool-order-break.json",
    "03-tool-order-break-turn2.json",
    "04-large-tool-output.json",
    "05-cache-write-not-economic.json",
    "05-cache-write-not-economic-turn2.json",
    "06-context-reduction-wins.json",
    "07-already-optimal.json",
    "08-unsafe-pruning-example.json",
];

#[test]
fn all_fixtures_load_and_validate() {
    for name in ALL_FIXTURES {
        let trace = common::load_fixture(name);
        validate_trace(&trace, None).unwrap_or_else(|e| panic!("{name} failed validation: {e}"));
        assert_eq!(trace.format_version, TRACE_FORMAT_VERSION, "{name}");
    }
}

#[test]
fn scenario_01_large_stable_prefix_is_reused() {
    let a = common::load_fixture("01-stable-prefix.json");
    let b = common::load_fixture("01-stable-prefix-turn2.json");
    let comparison = compare_traces(&a, &b, None).unwrap();

    assert!(!comparison.identical);
    assert_eq!(comparison.reusable_prefix_tokens, 9500);
    assert_eq!(comparison.tokens_after_divergence, 200);

    let divergence = comparison.first_divergence.as_ref().unwrap();
    assert_eq!(divergence.position, 4);
    assert_eq!(divergence.current_block_id, "user-request");
    assert_eq!(divergence.kind, DiffKind::Changed);

    // Provider-reported reuse matches the theoretical estimate here.
    let usage = b.usage.as_ref().unwrap();
    assert_eq!(usage.cache_read_tokens, Some(9500));
}

#[test]
fn scenario_02_early_timestamp_break_destroys_reuse() {
    let a = common::load_fixture("02-early-timestamp-break.json");
    let b = common::load_fixture("02-early-timestamp-break-turn2.json");
    let comparison = compare_traces(&a, &b, None).unwrap();

    let divergence = comparison.first_divergence.as_ref().unwrap();
    assert_eq!(divergence.position, 0);
    assert_eq!(divergence.current_block_id, "timestamp");
    assert_eq!(comparison.reusable_prefix_tokens, 0);
    assert_eq!(comparison.tokens_after_divergence, 9660);

    // Provider also reported zero cache reads on turn 2.
    let usage = b.usage.as_ref().unwrap();
    assert_eq!(usage.cache_read_tokens, Some(0));
}

#[test]
fn scenario_03_tool_order_break_is_reordering() {
    let a = common::load_fixture("03-tool-order-break.json");
    let b = common::load_fixture("03-tool-order-break-turn2.json");
    let comparison = compare_traces(&a, &b, None).unwrap();

    let divergence = comparison.first_divergence.as_ref().unwrap();
    assert_eq!(divergence.position, 2);
    assert_eq!(divergence.kind, DiffKind::Reordered);
    assert_eq!(comparison.reusable_prefix_tokens, 2000);
    assert!(comparison
        .explanation
        .contains("moved from position 3 to 2"));

    // Provider-reported cache reads match the reusable prefix (2000).
    let usage = b.usage.as_ref().unwrap();
    assert_eq!(usage.cache_read_tokens, Some(2000));
}

#[test]
fn scenario_04_large_tool_output_dominates_fresh_context() {
    let trace = common::load_fixture("04-large-tool-output.json");
    let analysis = analyze_trace(&trace, None).unwrap();

    assert_eq!(analysis.total_estimated_tokens, 45850);
    // file_content scores 0.48 (< 0.50 threshold), so only the first four
    // blocks count as theoretically stable.
    assert_eq!(analysis.theoretical_stable_tokens, 9500);
    assert_eq!(analysis.theoretical_volatile_tokens, 36350);

    let top = &analysis.top_fresh_blocks[0];
    assert_eq!(top.id, "tool-result-large");
    assert_eq!(top.tokens, 30000);

    // Reconciliation present because the fixture carries provider usage.
    // The provider reused 15500 tokens (including the file read); Prefixity's
    // conservative theoretical estimate is 9500. Both views are preserved and
    // kept distinct, per source-of-truth principle 7.
    let reconciliation = analysis.reconciliation.as_ref().unwrap();
    assert_eq!(reconciliation.reported_cache_read_tokens, Some(15500));
    assert_eq!(reconciliation.reported_fresh_tokens, Some(30350));
    assert_eq!(reconciliation.theoretical_reusable_prefix_tokens, 9500);
    assert!(reconciliation.note.contains("outranks"));
}

#[test]
fn scenario_05_cache_economics_depend_on_profile_data() {
    let a = common::load_fixture("05-cache-write-not-economic.json");
    let b = common::load_fixture("05-cache-write-not-economic-turn2.json");

    // With an expensive-write profile, caching is a net loss.
    let expensive = common::load_profile("synthetic-cache-write-expensive.json");
    let comparison = compare_traces(&a, &b, Some(&expensive)).unwrap();
    let economics = comparison.cache_economics.as_ref().unwrap();
    assert_eq!(economics.reusable_tokens, 3200);
    assert!(!economics.cache_worthwhile);

    // With the default synthetic profile, the same pair IS worthwhile,
    // proving provider economics are represented as data, not hard-coded.
    let default = default_synthetic_profile();
    let comparison_default = compare_traces(&a, &b, Some(&default)).unwrap();
    let economics_default = comparison_default.cache_economics.as_ref().unwrap();
    assert!(economics_default.cache_worthwhile);
}

#[test]
fn scenario_06_context_reduction_wins_over_cache_placement() {
    let trace = common::load_fixture("06-context-reduction-wins.json");
    let profile = default_synthetic_profile();

    let stable = prefixity_core::policy::simulate_policy(
        &trace,
        prefixity_core::policy::policy_from_name("stable-prefix")
            .unwrap()
            .as_ref(),
        &profile,
    )
    .unwrap();
    let defer = prefixity_core::policy::simulate_policy(
        &trace,
        prefixity_core::policy::policy_from_name("defer-volatile")
            .unwrap()
            .as_ref(),
        &profile,
    )
    .unwrap();

    // stable-prefix removes nothing; it only improves theoretical reuse.
    assert_eq!(stable.token_difference, 0);
    assert_eq!(stable.reusable_prefix_difference, 1800);

    // defer-volatile removes 20000 optional/stale tokens.
    assert_eq!(defer.token_difference, -20000);
    assert_eq!(defer.removed_blocks.len(), 2);

    // Removing context is a far larger saving than improving cache placement.
    assert!(defer.cost_difference < stable.cost_difference);
}

#[test]
fn scenario_07_already_optimal_no_false_recommendation() {
    let trace = common::load_fixture("07-already-optimal.json");
    let analysis = analyze_trace(&trace, None).unwrap();
    assert!(analysis.recommendation.contains("no structural change"));

    let profile = default_synthetic_profile();
    for name in prefixity_core::policy::available_policies() {
        let policy = prefixity_core::policy::policy_from_name(name).unwrap();
        let result =
            prefixity_core::policy::simulate_policy(&trace, policy.as_ref(), &profile).unwrap();
        assert_eq!(
            result.token_difference, 0,
            "policy {name} changed tokens on an optimal trace"
        );
        assert!(
            result.removed_blocks.is_empty(),
            "policy {name} removed blocks on an optimal trace"
        );
        assert!(
            result.relocated_blocks.is_empty(),
            "policy {name} relocated blocks on an optimal trace"
        );
    }
}

#[test]
fn scenario_08_required_block_is_never_pruned() {
    let trace = common::load_fixture("08-unsafe-pruning-example.json");
    let profile = default_synthetic_profile();

    for name in prefixity_core::policy::available_policies() {
        let policy = prefixity_core::policy::policy_from_name(name).unwrap();
        let result =
            prefixity_core::policy::simulate_policy(&trace, policy.as_ref(), &profile).unwrap();
        assert!(
            result
                .retained_blocks
                .iter()
                .any(|id| id == "critical-config"),
            "policy {name} removed the required block"
        );
        assert!(
            result
                .removed_blocks
                .iter()
                .all(|b| b.id != "critical-config"),
            "policy {name} reported removal of the required block"
        );
    }

    // Removal policies surface a warning that the required block was retained.
    let defer = prefixity_core::policy::simulate_policy(
        &trace,
        prefixity_core::policy::policy_from_name("defer-volatile")
            .unwrap()
            .as_ref(),
        &profile,
    )
    .unwrap();
    assert!(defer.warnings.iter().any(|w| w.contains("critical-config")));
}

#[test]
fn scenario_01_cost_uses_reported_usage_when_present() {
    let trace = common::load_fixture("01-stable-prefix-turn2.json");
    let profile = default_synthetic_profile();
    let analysis = analyze_trace(&trace, Some(&profile)).unwrap();
    let cost = analysis.cost.as_ref().unwrap();
    // Reported: input 9700, cache read 9500 -> fresh 200.
    assert_eq!(cost.input_tokens, 9700);
    assert_eq!(cost.cache_read_tokens, 9500);
    assert_eq!(cost.fresh_tokens, 200);
}
