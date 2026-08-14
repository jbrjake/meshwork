//! `verify_dsl::` — grammar tier for the declarative verify parser
//! (mw-sascrgs; DESIGN §12b). Parse, never execute: nothing in this
//! module runs a command. The corpus golden pins every classification —
//! DSL, MALFORMED (keyword-led but invalid, refused), SHELL (legacy text
//! for the MW-E5 trust gate).

use crate::common::{assert_golden, fixtures_root};
use meshwork::verify_dsl::{classify, Classified};

fn render(line: &str) -> String {
    match classify(line) {
        Classified::Dsl(preds) => format!(
            "DSL {}",
            preds
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" && ")
        ),
        Classified::Malformed(why) => format!("MALFORMED {why}"),
        Classified::LegacyShell => "SHELL".to_string(),
    }
}

/// Every corpus line's classification, golden-pinned.
#[test]
fn grammar_corpus_golden() {
    let corpus = std::fs::read_to_string(fixtures_root().join("verify-dsl/corpus.txt")).unwrap();
    let mut blob = String::new();
    for line in corpus.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        blob.push_str(line);
        blob.push_str("\n  = ");
        blob.push_str(&render(line));
        blob.push('\n');
    }
    assert_golden("verify-dsl.txt", &blob);
}

/// The observed store shapes all express in DSL.
#[test]
fn grammar_observed_shapes_accept() {
    for good in [
        "exists docs/batch-door.md",
        "absent HANDOFF.md",
        "contains docs/batch-door.md Q21",
        "contains PLAN-meshwork-build.md /4\\.1/",
        "run cargo test import_short_title",
        "all(exists FORMAT.md, run cargo test e2e::)",
    ] {
        assert!(
            matches!(classify(good), Classified::Dsl(_)),
            "{good}: {}",
            render(good)
        );
    }
}

/// A typo'd DSL verify refuses; it NEVER downgrades to arbitrary shell —
/// the silent-downgrade path would reopen exactly the hole the DSL closes.
#[test]
fn grammar_keyword_led_never_shell() {
    for bad in [
        "exists",
        "exists docs/a.md extra",
        "contains docs/x.md",
        "run cargo",
        "run rustc main.rs",
        "all()",
        "all(exists a.md,)",
        "all(exists a.md",
    ] {
        assert!(
            matches!(classify(bad), Classified::Malformed(_)),
            "{bad}: {}",
            render(bad)
        );
    }
}

/// Shell metacharacters and leading dashes are just characters that fail
/// the class — argument injection (Cursor GHSA-hf2x-r83r-qw5q, Flowise)
/// and traversal die at parse time.
#[test]
fn grammar_tight_classes() {
    for bad in [
        "run cargo test --workspace",
        "run cargo test -- --exact foo",
        "run cargo test a;b",
        "exists a$(x).md",
        "exists /etc/passwd",
        "exists ../outside.md",
        "exists -rf",
    ] {
        assert!(
            matches!(classify(bad), Classified::Malformed(_)),
            "{bad}: {}",
            render(bad)
        );
    }
}

/// Non-keyword text is legacy shell, verbatim, for the trust gate.
#[test]
fn grammar_legacy_shell_fallback() {
    for legacy in [
        "grep -q Q21 docs/batch-door.md",
        "out=$(cargo test F 2>&1) && echo \"$out\" | grep -qE 'ok'",
        "true",
        "./scripts/check-perf.sh",
    ] {
        assert!(
            matches!(classify(legacy), Classified::LegacyShell),
            "{legacy}: {}",
            render(legacy)
        );
    }
}
