//! The four Phase 0B experiment scenarios.

use crate::error::LiveError;

/// A Phase 0B experiment scenario. Only these four are defined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Scenario {
    /// One harmless request; verifies the usage schema matches our normalizer.
    SchemaSmoke,
    /// Sequential requests sharing a large prefix (2 for OpenAI/Anthropic,
    /// 3 for DeepSeek, which primes the cache with A and B).
    StablePrefix,
    /// A block near the beginning changes (turn B for OpenAI/Anthropic,
    /// turn C for DeepSeek so A and B first establish the common prefix).
    EarlyDivergence,
    /// The prefix is split into a stable core (~90%) and a late mutable
    /// suffix (~10%) as separate structural and wire blocks. The suffix
    /// changes late (2 requests for OpenAI/Anthropic; 4 for DeepSeek, which
    /// primes with A/B, diverges the suffix at C, and probes common-core
    /// persistence with a second distinct suffix at D).
    LateDivergence,
}

impl Scenario {
    /// Parse a scenario name. Accepts the full names and the single letters
    /// A–D.
    pub fn parse(input: &str) -> Result<Scenario, LiveError> {
        match input {
            "schema-smoke" | "A" | "a" => Ok(Scenario::SchemaSmoke),
            "stable-prefix" | "B" | "b" => Ok(Scenario::StablePrefix),
            "early-divergence" | "C" | "c" => Ok(Scenario::EarlyDivergence),
            "late-divergence" | "D" | "d" => Ok(Scenario::LateDivergence),
            other => Err(LiveError::argument(format!(
                "unknown scenario '{other}'. expected one of: schema-smoke, stable-prefix, early-divergence, late-divergence"
            ))),
        }
    }

    /// Canonical kebab-case name.
    pub fn as_str(&self) -> &'static str {
        match self {
            Scenario::SchemaSmoke => "schema-smoke",
            Scenario::StablePrefix => "stable-prefix",
            Scenario::EarlyDivergence => "early-divergence",
            Scenario::LateDivergence => "late-divergence",
        }
    }

    /// Single-letter label (A–D).
    pub fn label(&self) -> &'static str {
        match self {
            Scenario::SchemaSmoke => "A",
            Scenario::StablePrefix => "B",
            Scenario::EarlyDivergence => "C",
            Scenario::LateDivergence => "D",
        }
    }
}
