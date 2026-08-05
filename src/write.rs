//! Writing-side helpers for hand-editable YAML frontmatter: values the
//! strict parser (and any other YAML reader) will round-trip unchanged.

/// Render a scalar for a frontmatter value: plain when safe, double-quoted
/// when YAML would misread it (trailing `:` — the `cargo test foo::` trap —
/// comments, leading indicators, bool/number lookalikes).
#[must_use]
pub fn yaml_scalar(s: &str) -> String {
    if needs_quoting(s) {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

/// Clamp text to a BYTE budget (MW-D5 — bytes, never lines), char-boundary
/// safe, with a visible `…` when anything was cut.
#[must_use]
pub fn clamp_bytes(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut cut = max.saturating_sub('…'.len_utf8());
    while cut > 0 && !s.is_char_boundary(cut) {
        cut -= 1;
    }
    format!("{}…", &s[..cut])
}

fn needs_quoting(s: &str) -> bool {
    if s.is_empty() || s != s.trim() {
        return true;
    }
    if s.contains(": ") || s.contains(" #") || s.ends_with(':') || s.contains(',') {
        return true;
    }
    let first = s.chars().next().unwrap_or(' ');
    if "[]{}#&*!|>'\"%@`-?:".contains(first) {
        return true;
    }
    // Bool/number lookalikes would change type under YAML.
    matches!(
        s,
        "true" | "false" | "null" | "~" | "yes" | "no" | "on" | "off"
    ) || s.parse::<f64>().is_ok()
}
