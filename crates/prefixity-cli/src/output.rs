//! Output rendering for the `prefixity` CLI.
//!
//! Two modes:
//!
//! * human mode — readable text (all untrusted strings are sanitized);
//! * `--json` mode — a single stable JSON document per invocation, built
//!   from the serializable result structs so output is deterministic.
//!
//! The JSON builders are separate from the printers so tests can assert
//! byte-for-byte determinism without spawning the binary.

use prefixity_core::analysis::TraceAnalysis;
use prefixity_core::compare::Comparison;
use prefixity_core::error::PrefixityError;
use prefixity_core::policy::SimulationResult;
use prefixity_core::terminal::sanitize_for_terminal;
use prefixity_core::validation::ValidationReport;
use std::path::Path;

// ---------------------------------------------------------------------------
// JSON builders (deterministic)
// ---------------------------------------------------------------------------

/// JSON document for a successful validation.
pub fn validation_json(report: &ValidationReport) -> serde_json::Value {
    serde_json::json!({
        "ok": true,
        "valid": true,
        "warnings": report.warnings,
    })
}

/// JSON document for an analysis result.
pub fn analysis_json(analysis: &TraceAnalysis) -> serde_json::Value {
    serde_json::json!({ "ok": true, "analysis": analysis })
}

/// JSON document for a comparison result.
pub fn comparison_json(comparison: &Comparison) -> serde_json::Value {
    serde_json::json!({ "ok": true, "comparison": comparison })
}

/// JSON document for a simulation result.
pub fn simulation_json(result: &SimulationResult) -> serde_json::Value {
    serde_json::json!({ "ok": true, "simulation": result })
}

/// JSON document for an error.
pub fn error_json(error: &PrefixityError) -> serde_json::Value {
    serde_json::json!({ "ok": false, "error": error.to_string() })
}

// ---------------------------------------------------------------------------
// Printing
// ---------------------------------------------------------------------------

/// Print a JSON document with stable pretty-printing.
pub fn print_json(value: &serde_json::Value) {
    let text =
        serde_json::to_string_pretty(value).expect("JSON serialization of results cannot fail");
    println!("{text}");
}

/// Print an error in the mode implied by `--json`.
pub fn print_error(cli: &crate::cli::Cli, error: &PrefixityError) {
    if cli.json {
        print_json(&error_json(error));
    } else {
        eprintln!("error: {error}");
    }
}

/// Print the validation result.
pub fn print_validation(cli: &crate::cli::Cli, path: &Path, report: &ValidationReport) {
    if cli.json {
        print_json(&validation_json(report));
        return;
    }
    println!(
        "valid: true ({})",
        sanitize_for_terminal(&path.display().to_string())
    );
    if report.warnings.is_empty() {
        println!("warnings: (none)");
    } else {
        println!("warnings:");
        for warning in &report.warnings {
            println!("  - {}", sanitize_for_terminal(warning));
        }
    }
}

/// Print the analysis result.
pub fn print_analysis(cli: &crate::cli::Cli, analysis: &TraceAnalysis) {
    if cli.json {
        print_json(&analysis_json(analysis));
        return;
    }

    let t = &analysis.trace;
    println!(
        "Trace: {}   Session: {}",
        sanitize_for_terminal(&t.request_id),
        t.session_id
            .as_deref()
            .map(sanitize_for_terminal)
            .unwrap_or_else(|| "(none)".to_string())
    );
    println!(
        "Provider: {}   Model: {}   Format version: {}   Blocks: {}   Estimated tokens: {}",
        sanitize_for_terminal(&t.provider),
        sanitize_for_terminal(&t.model),
        analysis.format_version,
        analysis.block_count,
        comma(analysis.total_estimated_tokens)
    );
    println!(
        "Total bytes: {}   Heuristic token estimates used: {}",
        comma(analysis.total_bytes),
        if analysis.used_heuristic_token_estimates {
            "yes"
        } else {
            "no"
        }
    );
    println!();

    if let Some(usage) = &analysis.usage {
        println!("Provider-reported usage:");
        println!("  input          {}", opt_comma(usage.input_tokens));
        println!("  cache read     {}", opt_comma(usage.cache_read_tokens));
        println!("  cache write    {}", opt_comma(usage.cache_write_tokens));
        println!("  output         {}", opt_comma(usage.output_tokens));
        if let Some(rec) = &analysis.reconciliation {
            println!("  fresh (derived) {}", opt_comma(rec.reported_fresh_tokens));
            println!();
            println!("Reconciliation (theoretical vs reported):");
            println!(
                "  theoretical reusable prefix: {}",
                comma(rec.theoretical_reusable_prefix_tokens)
            );
            println!("  note: {}", sanitize_for_terminal(&rec.note));
        }
        println!();
    }

    println!("Theoretical context accounting (estimates):");
    println!(
        "  stable tokens    {} ({:.1}%)",
        comma(analysis.theoretical_stable_tokens),
        fraction(
            analysis.theoretical_stable_tokens,
            analysis.total_estimated_tokens
        )
    );
    println!(
        "  volatile tokens  {} ({:.1}%)",
        comma(analysis.theoretical_volatile_tokens),
        fraction(
            analysis.theoretical_volatile_tokens,
            analysis.total_estimated_tokens
        )
    );
    println!(
        "  theoretical reusable prefix: {}",
        comma(analysis.theoretical_reusable_prefix_tokens)
    );
    println!();

    println!("prefixity scores (experimental, explainable):");
    for score in &analysis.prefixity_scores {
        let summary = score.reasons.first().map(String::as_str).unwrap_or("");
        println!(
            "  {:<24} {:.2}   [{}]",
            sanitize_for_terminal(&score.block_id),
            score.score,
            sanitize_for_terminal(summary)
        );
    }
    println!();

    if analysis.top_fresh_blocks.is_empty() {
        println!("Top fresh-context contributors: (none)");
    } else {
        println!("Top fresh-context contributors:");
        for (i, block) in analysis.top_fresh_blocks.iter().enumerate() {
            println!(
                "  {}. {:<24} {} tok   [{}]",
                i + 1,
                sanitize_for_terminal(&block.id),
                comma(block.tokens),
                sanitize_for_terminal(&block.explanation)
            );
        }
    }
    println!();

    if let Some(cost) = &analysis.cost {
        println!(
            "Cost (profile: {} [{}]):",
            sanitize_for_terminal(&cost.profile_name),
            if cost.synthetic {
                "SYNTHETIC"
            } else {
                "external"
            }
        );
        println!(
            "  fresh input  {} tok  -> {:.6}",
            comma(cost.fresh_tokens),
            cost.fresh_input_cost
        );
        println!(
            "  cache read   {} tok  -> {:.6}",
            comma(cost.cache_read_tokens),
            cost.cache_read_cost
        );
        println!(
            "  cache write  {} tok  -> {:.6}",
            comma(cost.cache_write_tokens),
            cost.cache_write_cost
        );
        println!(
            "  output       {} tok  -> {:.6}",
            comma(cost.output_tokens),
            cost.output_cost
        );
        println!("  total                = {:.6}", cost.total_cost);
        println!();
    } else {
        println!("Cost: not computed (supply --provider-profile to estimate cost).");
        println!();
    }

    println!("Recommendation:");
    println!("  {}", sanitize_for_terminal(&analysis.recommendation));

    if !analysis.warnings.is_empty() {
        println!();
        println!("Warnings:");
        for warning in &analysis.warnings {
            println!("  - {}", sanitize_for_terminal(warning));
        }
    }
}

/// Print the comparison result.
pub fn print_comparison(cli: &crate::cli::Cli, comparison: &Comparison) {
    if cli.json {
        print_json(&comparison_json(comparison));
        return;
    }

    println!("{}", sanitize_for_terminal(&comparison.explanation));
    println!();
    println!("Summary:");
    println!(
        "  blocks: {} -> {}",
        comparison.blocks_a, comparison.blocks_b
    );
    println!("  identical: {}", comparison.identical);
    println!(
        "  unchanged positions: {}",
        comparison.unchanged_block_count
    );
    println!(
        "  theoretical cache-write tokens: {}",
        comma(comparison.theoretical_cache_write_tokens)
    );

    if let Some(economics) = &comparison.cache_economics {
        println!();
        println!("Cache economics (profile-based, theoretical):");
        println!("  no-cache cost:      {:.6}", economics.cost_no_cache);
        println!("  with-cache cost:    {:.6}", economics.cost_with_cache);
        println!(
            "  cache worthwhile:   {}",
            if economics.cache_worthwhile {
                "yes"
            } else {
                "no"
            }
        );
        println!("  {}", sanitize_for_terminal(&economics.explanation));
    } else {
        println!();
        println!("Cache economics: not computed (supply --provider-profile).");
    }

    if !comparison.warnings.is_empty() {
        println!();
        println!("Warnings:");
        for warning in &comparison.warnings {
            println!("  - {}", sanitize_for_terminal(warning));
        }
    }
}

/// Print the simulation result.
pub fn print_simulation(cli: &crate::cli::Cli, result: &SimulationResult) {
    if cli.json {
        print_json(&simulation_json(result));
        return;
    }

    println!("Policy: {}", sanitize_for_terminal(&result.policy));
    println!("  {}", sanitize_for_terminal(&result.description));
    println!();

    println!("Tokens:");
    println!("  original    {}", comma(result.original_tokens));
    println!("  simulated   {}", comma(result.simulated_tokens));
    println!("  difference  {}", signed_i64(result.token_difference));
    println!();

    println!("Theoretical reusable prefix:");
    println!(
        "  original    {}",
        comma(result.original_reusable_prefix_tokens)
    );
    println!(
        "  simulated   {}",
        comma(result.simulated_reusable_prefix_tokens)
    );
    println!(
        "  difference  {}",
        signed_i64(result.reusable_prefix_difference)
    );
    println!();

    if result.removed_blocks.is_empty() {
        println!("Removed blocks: (none)");
    } else {
        println!("Removed blocks:");
        for removed in &result.removed_blocks {
            println!(
                "  [{}] {}  ({})",
                removed.position,
                sanitize_for_terminal(&removed.id),
                sanitize_for_terminal(&removed.reason)
            );
        }
    }

    if result.relocated_blocks.is_empty() {
        println!("Relocated blocks: (none)");
    } else {
        println!("Relocated blocks:");
        for relocation in &result.relocated_blocks {
            println!(
                "  {}  {} -> {}",
                sanitize_for_terminal(&relocation.id),
                relocation.from_position,
                relocation.to_position
            );
        }
    }
    println!();

    println!(
        "Cost (profile: {} [{}]):",
        sanitize_for_terminal(&result.original_cost.profile_name),
        if result.original_cost.synthetic {
            "SYNTHETIC"
        } else {
            "external"
        }
    );
    println!("  original    {:.6}", result.original_cost.total_cost);
    println!("  simulated   {:.6}", result.simulated_cost.total_cost);
    println!("  difference  {}", signed_f64(result.cost_difference));
    println!();

    println!("Assumptions:");
    for assumption in &result.assumptions {
        println!("  - {}", sanitize_for_terminal(assumption));
    }
    if !result.warnings.is_empty() {
        println!();
        println!("Warnings:");
        for warning in &result.warnings {
            println!("  - {}", sanitize_for_terminal(warning));
        }
    }
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

/// Format a count with thousands separators.
fn comma(value: u64) -> String {
    let text = value.to_string();
    let mut out = String::new();
    for (index, c) in text.chars().enumerate() {
        if index > 0 && (text.len() - index) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Format an optional count, rendering `None` as `(not reported)`.
fn opt_comma(value: Option<u64>) -> String {
    match value {
        Some(v) => comma(v),
        None => "(not reported)".to_string(),
    }
}

/// Percentage of `part` within `total`, or 0.0 when `total` is zero.
fn fraction(part: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        part as f64 / total as f64 * 100.0
    }
}

/// Format a signed i64 with an explicit sign.
fn signed_i64(value: i64) -> String {
    if value >= 0 {
        format!("+{value}")
    } else {
        format!("{value}")
    }
}

/// Format a signed cost difference.
fn signed_f64(value: f64) -> String {
    format!("{value:+.6}")
}

/// Serialize any value for JSON and return the string (used by tests).
#[cfg(test)]
fn json_string<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("serialization cannot fail")
}

#[cfg(test)]
mod tests {
    use super::*;
    use prefixity_core::analysis::analyze_trace;
    use prefixity_core::compare::compare_traces;
    use prefixity_core::cost::default_synthetic_profile;
    use prefixity_core::policy::{policy_from_name, simulate_policy};
    use std::path::Path;

    fn fixture(name: &str) -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/traces")
            .join(name)
    }

    fn load(name: &str) -> prefixity_core::model::RequestTrace {
        let path = fixture(name);
        let data = std::fs::read(&path).unwrap_or_else(|e| panic!("failed to read {path:?}: {e}"));
        serde_json::from_slice(&data).unwrap_or_else(|e| panic!("failed to parse {path:?}: {e}"))
    }

    #[test]
    fn analysis_json_is_deterministic() {
        let trace = load("01-stable-prefix.json");
        let analysis = analyze_trace(&trace, Some(&default_synthetic_profile())).unwrap();
        let first = json_string(&analysis_json(&analysis));
        let second = json_string(&analysis_json(&analysis));
        assert_eq!(first, second);
        // And the wrapped document parses back.
        let parsed: serde_json::Value = serde_json::from_str(&first).unwrap();
        assert_eq!(parsed["ok"], true);
        assert!(parsed["analysis"]["total_estimated_tokens"].is_number());
    }

    #[test]
    fn comparison_json_is_deterministic() {
        let a = load("03-tool-order-break.json");
        let b = load("03-tool-order-break-turn2.json");
        let comparison = compare_traces(&a, &b, Some(&default_synthetic_profile())).unwrap();
        let first = json_string(&comparison_json(&comparison));
        let second = json_string(&comparison_json(&comparison));
        assert_eq!(first, second);
    }

    #[test]
    fn simulation_json_is_deterministic() {
        let trace = load("06-context-reduction-wins.json");
        let policy = policy_from_name("combined").unwrap();
        let result =
            simulate_policy(&trace, policy.as_ref(), &default_synthetic_profile()).unwrap();
        let first = json_string(&simulation_json(&result));
        let second = json_string(&simulation_json(&result));
        assert_eq!(first, second);
    }

    #[test]
    fn validation_json_is_deterministic() {
        let trace = load("01-stable-prefix.json");
        let report = prefixity_core::validation::validate_trace(&trace, None).unwrap();
        let first = json_string(&validation_json(&report));
        let second = json_string(&validation_json(&report));
        assert_eq!(first, second);
    }
}
