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

/// A `contains` regex is grep-like: `^`/`$` anchor lines, so a heading
/// on line three and the owner-gated `/^- <date> …/` marker both match,
/// while a mid-line occurrence of the anchored text does not.
#[test]
fn exec_contains_regex_anchors_lines() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("FORMAT.md"),
        "# Format\n\n## Views\nsee ## Views above\n- 2026-09-01 owner approved\n",
    )
    .unwrap();
    for pass in [
        "contains FORMAT.md /^## Views/",
        "contains FORMAT.md /^- 2026-09-01 owner approved$/",
        "contains FORMAT.md /^# Format$/",
    ] {
        assert!(execute(root, &dsl(pass)).is_ok(), "{pass} should pass");
    }
    for fail in [
        "contains FORMAT.md /^Views above$/",
        "contains FORMAT.md /^Views/",
        "contains FORMAT.md /^owner approved/",
    ] {
        assert!(execute(root, &dsl(fail)).is_err(), "{fail} should fail");
    }
}

/// portfolio#po-5sv8hs3: `.` stays line-bound (grep-like), so a two-phrase
/// `.*` pattern misses a marker that wrapped onto the next line — and the
/// inline `(?s)` flag is the per-predicate opt-in that lets it span the
/// wrap. Both facts pinned, so the documented idiom stays true.
#[test]
fn exec_contains_dot_all_opt_in() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("STATUS.md"),
        "# Status\n\nC2's body was rewritten and the ask is\nANSWERED by marasi (`ma-1`).\n",
    )
    .unwrap();
    assert!(
        execute(
            root,
            &dsl("contains STATUS.md /C2's body.*ANSWERED by marasi/")
        )
        .is_err(),
        "a line-bound `.` must not cross the wrap"
    );
    assert!(
        execute(
            root,
            &dsl("contains STATUS.md /(?s)C2's body.*ANSWERED by marasi/")
        )
        .is_ok(),
        "`(?s)` opts this predicate into spanning lines"
    );
    assert!(
        execute(root, &dsl("contains STATUS.md /^ANSWERED by marasi/")).is_ok(),
        "the single-line marker idiom needs no flag"
    );
}

fn dsl_argv(text: &str) -> Vec<String> {
    match &dsl(text)[0] {
        meshwork::verify_dsl::Predicate::Run { argv } => argv.clone(),
        other => panic!("{text} is not a run predicate: {other}"),
    }
}

/// MW-N1: `package=<crate>` and `target=<name>` are dash-free tokens the
/// grammar accepts on `cargo test` (`package=` on `cargo build` too); the
/// executor — never the parser — turns them into `-p`/`--test` in the
/// spawned argv, so the predicate renders as written and no author text
/// is ever a flag.
#[test]
fn run_package_target_tokens() {
    let line = "run cargo test package=leras-core target=suite spill::";
    assert_eq!(dsl(line)[0].to_string(), line);
    assert_eq!(
        meshwork::verify_exec::spawn_argv(&dsl_argv(line)),
        [
            "cargo",
            "test",
            "-p",
            "leras-core",
            "--test",
            "suite",
            "spill::"
        ]
    );
    assert_eq!(
        meshwork::verify_exec::spawn_argv(&dsl_argv("run cargo build package=leras-core")),
        ["cargo", "build", "-p", "leras-core"]
    );
    for bad in [
        "run cargo test package=",
        "run cargo test target=a/b",
        "run cargo test package=a=b",
        "run cargo fmt package=x",
        "run cargo build target=suite",
    ] {
        assert!(matches!(classify(bad), Classified::Malformed(_)), "{bad}");
    }
}

/// MW-N2: a dash-led arg still refuses, and the refusal names the
/// dash-free spelling where one exists.
#[test]
fn run_dash_refused_names_spelling() {
    for (bad, spelling) in [
        ("run cargo test -p leras", "package="),
        ("run cargo test --package leras", "package="),
        ("run cargo test --test suite", "target="),
        ("run cargo test --workspace", "no leading dash"),
    ] {
        match classify(bad) {
            Classified::Malformed(why) => assert!(why.contains(spelling), "{bad}: {why}"),
            _ => panic!("{bad} must refuse"),
        }
    }
}

/// MW-N3: `lacks` is `contains` inverted — native, confined — and a
/// missing file refuses rather than passing: deleting the file must not
/// close the task.
#[test]
fn lacks_predicate() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("door.md"),
        "the Q21 batch door\n- 2026-09-01 TODO later\n",
    )
    .unwrap();
    assert_eq!(
        dsl("lacks door.md TODO")[0].to_string(),
        "lacks door.md TODO"
    );
    assert!(execute(root, &dsl("lacks door.md Q99")).is_ok());
    assert!(execute(root, &dsl("lacks door.md /^- 2026-.* DONE/")).is_ok());
    let err = execute(root, &dsl("lacks door.md TODO")).unwrap_err();
    assert!(err.contains("found"), "{err}");
    let err = execute(root, &dsl("lacks GONE.md TODO")).unwrap_err();
    assert!(
        err.contains("GONE.md") && err.contains("missing"),
        "a missing file is not a missing pattern: {err}"
    );
    for bad in ["lacks door.md", "lacks", "lacks ../x.md a", "lacks -x.md a"] {
        assert!(matches!(classify(bad), Classified::Malformed(_)), "{bad}");
    }
}

/// MW-N4: `exists` takes one `*` inside its last path segment, resolved
/// under the same confinement; two stars, a star in an inner segment, a
/// bare star, or a glob on any other predicate refuse at parse.
#[test]
fn exists_single_segment_glob() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join("docs")).unwrap();
    std::fs::write(root.join("docs/RANK-2026-09-08.md"), "x").unwrap();
    assert_eq!(
        dsl("exists docs/RANK-*.md")[0].to_string(),
        "exists docs/RANK-*.md"
    );
    assert!(execute(root, &dsl("exists docs/RANK-*.md")).is_ok());
    assert!(execute(root, &dsl("exists docs/*.md")).is_ok());
    assert!(execute(root, &dsl("exists docs/BURN-*.md")).is_err());
    assert!(execute(root, &dsl("exists gone/*.md")).is_err());
    for bad in [
        "exists docs/*/x.md",
        "exists docs/**.md",
        "exists *",
        "absent docs/*.md",
        "contains docs/*.md x",
        "lacks docs/*.md x",
    ] {
        assert!(matches!(classify(bad), Classified::Malformed(_)), "{bad}");
    }
}

/// MW-N4: a directory is not a file `contains` can read — a trailing
/// slash refuses at parse, and a directory that exists by that name
/// refuses at execution rather than being walked.
#[test]
fn contains_dir_refused() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join("docs")).unwrap();
    assert!(matches!(
        classify("contains docs/ marker"),
        Classified::Malformed(_)
    ));
    assert!(matches!(
        classify("lacks docs/ marker"),
        Classified::Malformed(_)
    ));
    let err = execute(root, &dsl("contains docs marker")).unwrap_err();
    assert!(err.contains("directory"), "{err}");
    let err = execute(root, &dsl("lacks docs marker")).unwrap_err();
    assert!(err.contains("directory"), "{err}");
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
