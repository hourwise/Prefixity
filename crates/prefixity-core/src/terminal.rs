//! Safety helpers for rendering untrusted trace content into a terminal.
//!
//! Trace metadata and (rarely) content come from files that may be malicious.
//! Before any string from a trace is printed to a terminal, it must pass
//! through [`sanitize_for_terminal`] so that terminal escape/control
//! sequences cannot be smuggled into the user's terminal.

/// Replace every control character (other than `\n` and `\t`) with a visible
/// replacement character, so terminal escape sequences in untrusted input are
/// neutralised.
pub fn sanitize_for_terminal(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '\n' | '\t' => out.push(c),
            c if c.is_control() => out.push('\u{FFFD}'), // U+FFFD REPLACEMENT CHARACTER
            c => out.push(c),
        }
    }
    out
}

/// Truncate a string to at most `max` characters, appending `...` when cut.
/// The result is always at most `max` chars long. Used for IDs and hashes in
/// human-readable output.
pub fn truncate_middle(input: &str, max: usize) -> String {
    if input.chars().count() <= max {
        return input.to_string();
    }
    let keep = max.saturating_sub(3);
    let head: String = input.chars().take(keep / 2).collect();
    let tail: String = input
        .chars()
        .rev()
        .take(keep - keep / 2)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{head}...{tail}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_control_characters() {
        assert_eq!(sanitize_for_terminal("a\x1b[31mb"), "a\u{FFFD}[31mb");
        assert_eq!(sanitize_for_terminal("a\nb\tc"), "a\nb\tc");
    }

    #[test]
    fn truncates_from_middle() {
        assert_eq!(truncate_middle("abc", 10), "abc");
        let t = truncate_middle("abcdefghijklmnop", 9);
        assert_eq!(t.chars().count(), 9);
        assert!(t.starts_with("abc") && t.ends_with("nop"));
        assert!(t.contains("..."));
    }
}
