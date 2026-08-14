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

// mw-dthxs3q: the executor half — argv-only spawn (no shell anywhere),
// env scrubbed to a pinned set, cwd = repo root, wall-clock timeout,
// byte-capped output. DSL verifies bypass the MW-E5 trust gate because
// this module makes them safe by construction.

use meshwork::verify_exec::{execute, run_argv};
use std::time::Duration;

fn dsl(text: &str) -> Vec<meshwork::verify_dsl::Predicate> {
    match classify(text) {
        Classified::Dsl(p) => p,
        other => panic!("{text} did not parse as DSL: {:?}", render_class(&other)),
    }
}

fn render_class(c: &Classified) -> String {
    match c {
        Classified::Dsl(_) => "DSL".into(),
        Classified::Malformed(m) => format!("MALFORMED {m}"),
        Classified::LegacyShell => "SHELL".into(),
    }
}

/// exists/absent/contains evaluate natively — no process at all.
#[test]
fn exec_native_predicates() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("door.md"), "the Q21 batch door\n").unwrap();

    for pass in [
        "exists door.md",
        "absent GONE.md",
        "contains door.md Q21",
        "contains door.md /Q[0-9]+ batch/",
        "all(exists door.md, contains door.md Q21)",
    ] {
        assert!(execute(root, &dsl(pass)).is_ok(), "{pass} should pass");
    }
    for (fail, why) in [
        ("exists GONE.md", "missing file"),
        ("absent door.md", "present file"),
        ("contains door.md Q99", "literal absent"),
        ("contains GONE.md Q21", "unreadable file"),
        ("all(exists door.md, exists GONE.md)", "one conjunct fails"),
    ] {
        assert!(execute(root, &dsl(fail)).is_err(), "{fail}: {why}");
    }
}

/// run spawns argv-style: metacharacters reach the child verbatim —
/// there is no shell to give them meaning.
#[test]
fn exec_run_argv_no_shell() {
    let dir = tempfile::tempdir().unwrap();
    let argv = |v: &[&str]| v.iter().map(ToString::to_string).collect::<Vec<_>>();
    let out = run_argv(
        dir.path(),
        &argv(&["echo", "$(pwd)", ";", "a&&b"]),
        Duration::from_secs(5),
        4096,
    )
    .unwrap();
    assert!(
        out.contains("$(pwd)") && out.contains("a&&b"),
        "metacharacters must arrive literal: {out}"
    );
    // Exit status is the verdict.
    assert!(run_argv(dir.path(), &argv(&["false"]), Duration::from_secs(5), 4096).is_err());
}

/// The child sees the pinned env set, not the caller's environment.
#[test]
fn exec_env_scrubbed() {
    let dir = tempfile::tempdir().unwrap();
    let out = run_argv(
        dir.path(),
        &["printenv".to_string()],
        Duration::from_secs(5),
        65536,
    )
    .unwrap();
    // cargo sets CARGO_PKG_NAME for this test process; a scrubbed child
    // must not inherit it. PATH survives — the runner needs resolving.
    assert!(
        !out.contains("CARGO_PKG_NAME="),
        "caller env leaked into the child: {out}"
    );
    assert!(out.contains("PATH="), "pinned set keeps PATH: {out}");
}

/// A hung child dies at the wall clock, loudly.
#[test]
fn exec_timeout_kills() {
    let dir = tempfile::tempdir().unwrap();
    let started = std::time::Instant::now();
    let err = run_argv(
        dir.path(),
        &["sleep".to_string(), "30".to_string()],
        Duration::from_millis(300),
        4096,
    )
    .unwrap_err();
    assert!(err.contains("timeout"), "{err}");
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "the kill must not wait out the child"
    );
}

/// Output is byte-capped; the child still runs to completion.
#[test]
fn exec_output_capped() {
    let dir = tempfile::tempdir().unwrap();
    let out = run_argv(
        dir.path(),
        &["seq".to_string(), "1".to_string(), "200000".to_string()],
        Duration::from_secs(30),
        4096,
    )
    .unwrap();
    assert!(out.len() <= 4096, "cap held: {} bytes", out.len());
    assert!(out.starts_with("1\n"), "capped from the head: {out}");
}
