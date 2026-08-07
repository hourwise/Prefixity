//! Fixture-scenario integration tests.
//!
//! Each test maps to one of the documented synthetic scenarios (01-16) and
//! checks the numbers the harness should deterministically produce.

mod common;

use prefixity_core::analysis::analyze_trace;
use prefixity_core::compare::{compare_traces, DiffKind};
use prefixity_core::cost::default_synthetic_profile;
use prefixity_core::model::TRACE_FORMAT_VERSION;
use prefixity_core::structure::structural_fingerprint;
use prefixity_core::usage::normalize_usage;
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
    "09-anthropic-usage-semantics.json",
    "10-deepseek-usage-semantics.json",
    "11-openai-usage-semantics.json",
    "12-same-content-different-role.json",
    "13-same-content-different-zone.json",
    "14-first-request-no-observed-reuse.json",
    "15-history-proves-prefix-reuse.json",
    "15-history-proves-prefix-reuse-turn2.json",
    "16-global-reorder-would-be-unsafe.json",
    "17-deepseek-live-schema-smoke.json",
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
fn scenario_17_deepseek_live_schema_smoke_normalizes() {
    // Sanitized fixture derived from the first real Phase 0B DeepSeek live
    // schema-smoke (2026-08-07, deepseek-v4-flash). Proves the observed live
    // usage shape normalizes to the recorded values. It is NOT synthetic
    // provider documentation.
    let trace = common::load_fixture("17-deepseek-live-schema-smoke.json");
    let usage = trace.usage.as_ref().expect("fixture must carry usage");
    assert_eq!(usage.provider_schema, "deepseek-chat-completions-v1");
    let normalized = normalize_usage(usage);
    assert_eq!(
        normalized.normalization_source,
        "deepseek-chat-completions-v1"
    );
    // Observed live values: hit 0 + miss 1215 = total 1215; cache read 0.
    assert_eq!(normalized.total_input_tokens, Some(1215));
    assert_eq!(normalized.fresh_input_tokens, Some(1215));
    assert_eq!(normalized.cache_read_tokens, Some(0));
    assert_eq!(normalized.output_tokens, Some(1));

    // The fixture must contain only safe accounting values: no credentials,
    // no credential headers/values, no provider request id.
    let raw = std::fs::read_to_string(common::fixture_path("17-deepseek-live-schema-smoke.json"))
        .unwrap();
    let lower = raw.to_lowercase();
    for forbidden in ["api_key", "bearer", "sk-", "48883131"] {
        assert!(
            !lower.contains(forbidden),
            "fixture must not contain '{forbidden}'"
        );
    }
}

#[test]
fn scenario_01_large_stable_prefix_is_reused() {
    let a = common::load_fixture("01-stable-prefix.json");
    let b = common::load_fixture("01-stable-prefix-turn2.json");
    let comparison = compare_traces(&a, &b, None).unwrap();

    assert!(!comparison.identical);
    assert_eq!(comparison.observed_reusable_prefix_tokens, 9500);
    assert_eq!(comparison.tokens_after_divergence, 200);

    let divergence = comparison.first_divergence.as_ref().unwrap();
    assert_eq!(divergence.position, 4);
    assert_eq!(divergence.current_block_id, "user-request");
    assert_eq!(divergence.kind, DiffKind::Changed);

    // Provider-reported reuse (normalized) also reports 9500 cache reads.
    let usage = b.usage.as_ref().map(normalize_usage).unwrap();
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
    assert_eq!(comparison.observed_reusable_prefix_tokens, 0);
    assert_eq!(comparison.tokens_after_divergence, 9660);

    // Provider also reported zero cache reads on turn 2.
    let usage = b.usage.as_ref().map(normalize_usage).unwrap();
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
    assert_eq!(comparison.observed_reusable_prefix_tokens, 2000);
    assert!(comparison
        .explanation
        .contains("moved from position 3 to 2"));

    // Provider-reported cache reads match the observed reuse (2000).
    let usage = b.usage.as_ref().map(normalize_usage).unwrap();
    assert_eq!(usage.cache_read_tokens, Some(2000));
}

#[test]
fn scenario_04_large_tool_output_dominates_fresh_context() {
    let trace = common::load_fixture("04-large-tool-output.json");
    let analysis = analyze_trace(&trace, None).unwrap();

    assert_eq!(analysis.total_estimated_tokens, 45850);
    // file_content scores 0.48 (< 0.50 threshold), so only the first four
    // blocks count as stable-prefix candidates.
    assert_eq!(analysis.stable_prefix_candidate_tokens, 9500);
    assert_eq!(analysis.volatile_tokens, 36350);

    let top = &analysis.top_fresh_blocks[0];
    assert_eq!(top.id, "tool-result-large");
    assert_eq!(top.tokens, 30000);

    // Reconciliation: provider reused 15500 (including the file read); the
    // conservative candidate figure is 9500. Both views are preserved and
    // kept distinct, and a single trace cannot prove reuse.
    let reconciliation = analysis.reconciliation.as_ref().unwrap();
    assert_eq!(reconciliation.reported_cache_read_tokens, Some(15500));
    assert_eq!(reconciliation.reported_fresh_input_tokens, Some(30350));
    assert_eq!(reconciliation.leading_stable_prefix_candidate_tokens, 9500);
    assert!(reconciliation.note.contains("cannot prove reuse"));
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

    // stable-prefix removes nothing; it only improves heuristic candidates.
    assert_eq!(stable.token_difference, 0);
    assert_eq!(stable.stable_prefix_candidate_difference, 1800);

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
    // Reported: total input 9700, cache read 9500 -> fresh 200 (synthetic schema).
    assert_eq!(cost.total_input_tokens, 9700);
    assert_eq!(cost.cache_read_tokens, 9500);
    assert_eq!(cost.fresh_input_tokens, 200);
    assert!(cost.fresh_input_derivation.contains("synthetic"));
}

#[test]
fn scenario_09_anthropic_usage_semantics_are_normalized_correctly() {
    let trace = common::load_fixture("09-anthropic-usage-semantics.json");
    let analysis = analyze_trace(&trace, None).unwrap();
    let normalized = analysis.normalized_usage.as_ref().unwrap();

    // input_tokens is the UNCACHED remainder; total is the sum of the three
    // input categories.
    assert_eq!(normalized.total_input_tokens, Some(5000));
    assert_eq!(normalized.fresh_input_tokens, Some(500));
    assert_eq!(normalized.cache_read_tokens, Some(4000));
    assert_eq!(normalized.cache_write_tokens, Some(500));
    assert_eq!(normalized.output_tokens, Some(120));
    assert_eq!(normalized.normalization_source, "anthropic-messages-v1");
}

#[test]
fn scenario_10_deepseek_usage_semantics_are_normalized_correctly() {
    let trace = common::load_fixture("10-deepseek-usage-semantics.json");
    let analysis = analyze_trace(&trace, None).unwrap();
    let normalized = analysis.normalized_usage.as_ref().unwrap();

    assert_eq!(normalized.total_input_tokens, Some(5000));
    assert_eq!(normalized.fresh_input_tokens, Some(1000));
    assert_eq!(normalized.cache_read_tokens, Some(4000));
    assert_eq!(normalized.cache_write_tokens, None);
    assert_eq!(normalized.output_tokens, Some(120));
    assert_eq!(
        normalized.normalization_source,
        "deepseek-chat-completions-v1"
    );
}

#[test]
fn scenario_11_openai_usage_semantics_are_normalized_correctly() {
    let trace = common::load_fixture("11-openai-usage-semantics.json");
    let analysis = analyze_trace(&trace, None).unwrap();
    let normalized = analysis.normalized_usage.as_ref().unwrap();

    assert_eq!(normalized.total_input_tokens, Some(5000));
    assert_eq!(normalized.fresh_input_tokens, Some(1000));
    assert_eq!(normalized.cache_read_tokens, Some(4000));
    assert_eq!(normalized.cache_write_tokens, None);
    assert_eq!(normalized.output_tokens, Some(120));
    assert_eq!(
        normalized.normalization_source,
        "openai-chat-completions-v1"
    );
}

#[test]
fn scenario_12_same_content_different_role_differs_structurally() {
    let trace = common::load_fixture("12-same-content-different-role.json");
    let first = &trace.blocks[0];
    let second = &trace.blocks[1];
    // Same text -> same content hash.
    assert_eq!(first.content_hash, second.content_hash);
    // Different role -> different structural fingerprint.
    assert_ne!(
        structural_fingerprint(first),
        structural_fingerprint(second)
    );
}

#[test]
fn scenario_13_same_content_different_zone_differs_structurally() {
    let trace = common::load_fixture("13-same-content-different-zone.json");
    let first = &trace.blocks[0];
    let second = &trace.blocks[1];
    assert_eq!(first.content_hash, second.content_hash);
    assert_ne!(
        structural_fingerprint(first),
        structural_fingerprint(second)
    );
}

#[test]
fn scenario_14_first_request_must_not_claim_reuse() {
    let trace = common::load_fixture("14-first-request-no-observed-reuse.json");
    let analysis = analyze_trace(&trace, Some(&default_synthetic_profile())).unwrap();

    // No provider usage captured, so nothing is normalized and no
    // reconciliation can be produced.
    assert!(analysis.normalized_usage.is_none());
    assert!(analysis.reconciliation.is_none());

    // Cost must NOT bill stable-prefix candidates at cache-read prices.
    let cost = analysis.cost.as_ref().unwrap();
    assert_eq!(cost.cache_read_tokens, 0);
    assert_eq!(cost.cache_write_tokens, 0);
    assert_eq!(cost.fresh_input_tokens, analysis.total_estimated_tokens);
    assert!(cost.fresh_input_derivation.contains("no provider usage"));

    // The recommendation explicitly states a single trace cannot prove reuse.
    assert!(analysis.recommendation.contains("cannot prove"));
}

#[test]
fn scenario_15_history_proves_observed_reuse_separate_from_provider_reported() {
    let a = common::load_fixture("15-history-proves-prefix-reuse.json");
    let b = common::load_fixture("15-history-proves-prefix-reuse-turn2.json");
    let comparison = compare_traces(&a, &b, None).unwrap();

    // Observed structural reuse from the trace pair.
    assert_eq!(comparison.observed_reusable_prefix_tokens, 5200);
    // Provider-reported cache reads (normalized) — a distinct concept, and
    // deliberately different (5000) to prove they are not interchangeable.
    assert_eq!(comparison.provider_reported_cache_read_tokens, Some(5000));
    assert!(comparison.reuse_reconciliation_note.is_some());
    assert!(comparison
        .reuse_reconciliation_note
        .as_ref()
        .unwrap()
        .contains("outrank"));

    // A single-trace analysis of turn 2 cannot claim the 5200 as reuse.
    let single = analyze_trace(&b, None).unwrap();
    assert_eq!(single.leading_stable_prefix_candidate_tokens, 5200);
    assert!(single.recommendation.contains("cannot prove"));
}

#[test]
fn scenario_16_global_reorder_would_be_unsafe_is_deferred() {
    let trace = common::load_fixture("16-global-reorder-would-be-unsafe.json");
    let profile = default_synthetic_profile();
    let stable = prefixity_core::policy::simulate_policy(
        &trace,
        prefixity_core::policy::policy_from_name("stable-prefix")
            .unwrap()
            .as_ref(),
        &profile,
    )
    .unwrap();

    // The numerically attractive but semantically unsafe reorder is NOT
    // applied: no relocations, and the unsafe transformations are deferred.
    assert!(
        stable.relocated_blocks.is_empty(),
        "unsafe reorder must not be applied"
    );
    assert!(!stable.unsafe_transformations_deferred.is_empty());
    assert!(stable
        .unsafe_transformations_deferred
        .iter()
        .any(|d| d.contains("cross-zone")));
    assert!(stable
        .warnings
        .iter()
        .any(|w| w.contains("No safe relocation")));
}
