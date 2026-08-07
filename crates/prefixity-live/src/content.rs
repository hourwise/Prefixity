//! Deterministic synthetic prompt content for Phase 0B.
//!
//! Content is generated locally from a fixed seed and a fixed algorithm. It
//! contains no private source code, no user data, no web content, no
//! copyrighted long-form text, and no secrets. The same seed always produces
//! the same text (deterministic).

use crate::scenario::Scenario;

/// Token estimate heuristic used by the recorder: ceil(chars / 4). Mirrors
/// the documented heuristic in `prefixity-core` so recorded estimates are
/// coherent with analysis.
pub fn estimate_tokens(text: &str) -> u64 {
    let chars = text.chars().count() as u64;
    if chars == 0 {
        0
    } else {
        chars.div_ceil(4)
    }
}

/// A xorshift64* PRNG for deterministic content generation.
struct Xorshift {
    state: u64,
}

impl Xorshift {
    fn new(seed: u64) -> Xorshift {
        // SplitMix64-style initialization so adjacent seeds diverge.
        let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        Xorshift {
            state: z ^ (z >> 31),
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

/// Generate a deterministic neutral text block whose Prefixity token estimate
/// is at least `target_tokens`.
///
/// The text is a sequence of lines of the form
/// `sp000000 <16-hex> <16-hex> ... neutral`, which is obviously synthetic,
/// neutral, and reproducible from the seed.
pub fn generate_prefix(seed: u64, target_tokens: u64) -> String {
    let mut rng = Xorshift::new(seed);
    let mut text = String::new();
    let mut index = 0u64;
    while estimate_tokens(&text) < target_tokens {
        if index > 0 {
            text.push('\n');
        }
        text.push_str(&format!("sp{index:06} "));
        for _ in 0..8 {
            text.push_str(&format!("{:016x} ", rng.next_u64()));
        }
        text.pop(); // drop trailing space
        text.push_str(" neutral");
        index += 1;
    }
    text
}

/// The experiment header block text (identical across turns, except for
/// early-divergence, where it diverges from the plan's configured turn
/// onward: turn B for OpenAI/Anthropic, turn C for DeepSeek).
pub fn header_for(experiment_id: &str, seed: u64) -> String {
    format!("prefixity-live experiment {experiment_id} seed {seed}")
}

/// The per-turn tail instruction text.
pub fn tail_for(scenario: Scenario, turn: usize) -> String {
    match scenario {
        Scenario::SchemaSmoke => "Experiment schema smoke. Reply exactly with: OK".to_string(),
        Scenario::StablePrefix => {
            format!(
                "Experiment stable-prefix turn {}. Reply exactly with: OK",
                turn_label(turn)
            )
        }
        Scenario::EarlyDivergence => {
            format!(
                "Experiment early-divergence turn {}. Reply exactly with: OK",
                turn_label(turn)
            )
        }
        Scenario::LateDivergence => {
            format!(
                "Experiment late-divergence turn {}. Reply exactly with: OK",
                turn_label(turn)
            )
        }
    }
}

/// Single-letter label for a turn (A/B/C) used in content and reports.
pub fn turn_label(turn: usize) -> &'static str {
    match turn {
        1 => "A",
        2 => "B",
        3 => "C",
        _ => "X",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_is_deterministic() {
        let a = generate_prefix(42, 8000);
        let b = generate_prefix(42, 8000);
        assert_eq!(a, b);
    }

    #[test]
    fn different_seeds_differ() {
        let a = generate_prefix(42, 8000);
        let b = generate_prefix(43, 8000);
        assert_ne!(a, b);
    }

    #[test]
    fn reaches_approximately_target_size() {
        let text = generate_prefix(7, 8000);
        let estimated = estimate_tokens(&text);
        assert!(estimated >= 8000, "expected >= 8000, got {estimated}");
        assert!(
            estimated < 9000,
            "should stop shortly after target, got {estimated}"
        );
    }

    #[test]
    fn no_secret_or_private_markers_in_content() {
        let text = generate_prefix(1234, 8000);
        assert!(!text.contains("BEGIN ") && !text.contains("PRIVATE"));
        assert!(text.starts_with("sp000000"));
    }

    #[test]
    fn tails_are_stable_and_distinct() {
        assert_ne!(
            tail_for(Scenario::StablePrefix, 1),
            tail_for(Scenario::StablePrefix, 2)
        );
        assert!(tail_for(Scenario::SchemaSmoke, 1).contains("OK"));
    }
}
