//! `unit::` — byte-budget primitives (MW-D5): budgets are bytes, never
//! lines — the `wc -l` gaming hole stays closed.

use meshwork::write::clamp_bytes;

#[test]
fn budget_bytes_not_lines() {
    // Multibyte text: em dashes are 3 bytes each. A char-count or line
    // budget would pass this; only a byte budget clamps it.
    let text = "—".repeat(100); // 300 bytes, 100 chars, 1 line
    let clamped = clamp_bytes(&text, 100);
    assert!(clamped.len() <= 100, "byte len: {}", clamped.len());
    assert!(
        clamped.chars().count() < 100,
        "fewer chars than a char budget would keep"
    );
    assert!(clamped.ends_with('…'), "truncation is visible");

    // Char-boundary safety: never panics, never splits a codepoint.
    for max in 0..12 {
        let c = clamp_bytes("aé—b", max);
        assert!(c.len() <= max + '…'.len_utf8());
        assert!(std::str::from_utf8(c.as_bytes()).is_ok());
    }

    // Under budget: unchanged, no marker.
    assert_eq!(clamp_bytes("short", 100), "short");

    // One line of 10KB still gets cut — lines don't matter (MW-D5).
    let one_line = "x".repeat(10_240);
    assert!(clamp_bytes(&one_line, 6144).len() <= 6144);
}
