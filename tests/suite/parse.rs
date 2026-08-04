//! `parse::` — unit tier for the task-file parser (PLAN 0.1; MW-A1/A6/I2/K1).
//! Exercises the public API only; the normative input is DESIGN §2 verbatim.

use meshwork::parse::{parse_task_str, ParsedTask, Status};

const NORMATIVE: &str = r"---
id: sz-k7f3
title: Fix spill cliff at 600M keys
status: doing            # open | doing | blocked | done | dropped
category: engine/spill   # one slash-path, arbitrary depth (MW-B4/B8)
labels: [perf, p0]       # many, flat (MW-B5)
needs: [sz-a2m9, leras#lr-x4x1]   # hard deps; repo#id crosses repos (MW-B3)
parent: sz-b881          # nesting (MW-B1)
discovered-from: sz-c7q2 # provenance (MW-E4)
verify: cargo test -p sazed-spill -- --exact spill::cliff_600m
docs:
  - docs/PLAN-spill-durable-unification.md#§-budget-path   # drill-through (MW-F1)
attachments:
  - attachments/sz-k7f3/spill-p99.log                      # MW-K2
seq: 40                  # per-repo order weight; portfolio overlay supersedes
github: 214              # mirror issue number; set once, absent until mirrored (MW-H3)
created: 2026-08-04
blocked-reason:          # required non-empty iff status: blocked (MW-E1)
---
Why/context, written to act cold — a few lines, not a narrative.
Acceptance beyond `verify:` if any. Long design lives behind docs:, never here (MW-A5).

## log
- 2026-08-04 open→doing — repro landed, bisecting spill batch size

## comments
- 2026-08-04 [jon] p99 only degrades with governed-spill on; see spill-p99.log
- 2026-08-04 [claude/f10a7561] bisected to batch=64k; excerpt attached
";

fn valid(file_name: &str, text: &str) -> meshwork::parse::Task {
    match parse_task_str(file_name, text) {
        ParsedTask::Valid(t) => *t,
        ParsedTask::Invalid(inv) => panic!("expected valid, got invalid: {}", inv.error),
    }
}

fn invalid(file_name: &str, text: &str) -> meshwork::parse::Invalid {
    match parse_task_str(file_name, text) {
        ParsedTask::Invalid(inv) => inv,
        ParsedTask::Valid(t) => panic!("expected invalid, got valid task {}", t.id),
    }
}

/// MW-A1: the tool tolerates hand edits — the DESIGN §2 example, inline
/// comments, empty keys and all, parses to exactly the documented fields.
#[test]
fn roundtrip_hand_edited() {
    let t = valid("sz-k7f3-fix-spill-cliff-at-600m-keys.md", NORMATIVE);
    assert_eq!(t.id, "sz-k7f3");
    assert_eq!(t.title, "Fix spill cliff at 600M keys");
    assert_eq!(t.status, Status::Doing);
    assert_eq!(t.category.as_deref(), Some("engine/spill"));
    assert_eq!(t.labels, ["perf", "p0"]);
    assert_eq!(t.needs, ["sz-a2m9", "leras#lr-x4x1"]);
    assert_eq!(t.parent.as_deref(), Some("sz-b881"));
    assert_eq!(t.discovered_from.as_deref(), Some("sz-c7q2"));
    assert_eq!(
        t.verify.as_deref(),
        Some("cargo test -p sazed-spill -- --exact spill::cliff_600m")
    );
    assert_eq!(t.attachments, ["attachments/sz-k7f3/spill-p99.log"]);
    assert_eq!(t.seq, Some(40));
    assert_eq!(t.github, Some(214));
    assert_eq!(t.created.as_deref(), Some("2026-08-04"));
    assert_eq!(
        t.blocked_reason, None,
        "empty value is None, not empty string"
    );
    assert_eq!(t.waived, None);
    assert!(t.description.contains("act cold"));
    assert_eq!(t.log.len(), 1);
    assert!(t.log[0].contains("open→doing"));
    assert_eq!(t.comments.len(), 2);
    assert!(
        t.warnings.is_empty(),
        "unexpected warnings: {:?}",
        t.warnings
    );
}

/// MW-A6: unknown frontmatter keys warn — never fail the parse.
#[test]
fn unknown_field_warns() {
    let text =
        "---\nid: az-unk1\ntitle: Unknown field\nstatus: open\nsprint-points: 5\n---\nbody\n";
    let t = valid("az-unk1-unknown-field.md", text);
    assert_eq!(t.status, Status::Open);
    assert!(
        t.warnings.iter().any(|w| w.contains("sprint-points")),
        "warnings must name the unknown key: {:?}",
        t.warnings
    );
}

/// MW-F1: docs links keep their repo-relative path + anchor, verbatim.
#[test]
fn docs_links() {
    let t = valid("sz-k7f3-fix-spill-cliff-at-600m-keys.md", NORMATIVE);
    assert_eq!(
        t.docs,
        ["docs/PLAN-spill-durable-unification.md#§-budget-path"]
    );
}

/// MW-K1: `- <date> [<author>] text`, continuation lines two-space indented;
/// identity is a free string, recorded as claimed.
#[test]
fn comment_format() {
    let text = "---\nid: az-c0m9\ntitle: Comments\nstatus: open\n---\nbody\n\n## comments\n- 2026-08-03 [claude/f10a7561] bisected to batch=64k; the cliff tracks the\n  governor wakeup interval, not the batch size itself\n- 2026-08-04 [maya] ship it\n";
    let t = valid("az-c0m9-comments.md", text);
    assert_eq!(t.comments.len(), 2);
    assert_eq!(t.comments[0].date, "2026-08-03");
    assert_eq!(t.comments[0].author, "claude/f10a7561");
    assert!(
        t.comments[0].text.contains("governor wakeup interval"),
        "continuation line must join the entry: {:?}",
        t.comments[0].text
    );
    assert_eq!(t.comments[1].date, "2026-08-04");
    assert_eq!(t.comments[1].author, "maya");
    assert_eq!(t.comments[1].text, "ship it");
}

/// Log entries parse as ordered raw entries; continuations join.
#[test]
fn log_entries_parse() {
    let text = "---\nid: az-d0w1\ntitle: Log\nstatus: doing\n---\nbody\n\n## log\n- 2026-08-02 open→doing — started, longer note\n  spilling onto a second line\n- 2026-08-03 note — checkpoint\n";
    let t = valid("az-d0w1-log.md", text);
    assert_eq!(t.log.len(), 2);
    assert!(t.log[0].contains("second line"));
    assert!(t.log[1].starts_with("2026-08-03"));
}

/// MW-I2: a file that fails to parse surfaces as invalid with the ID
/// recovered from the filename — it must never silently vanish.
#[test]
fn invalid_carries_filename_id() {
    let text = "---\nid: ax-brk9\ntitle: [unclosed bracket, this YAML does not parse\nstatus: open\n---\nbody\n";
    let inv = invalid("ax-brk9-unparseable.md", text);
    assert_eq!(inv.id, "ax-brk9");
    assert!(!inv.error.is_empty());
}

/// MW-I1/I2: union merge's failure mode — duplicate top-level keys — is
/// rejected by strict parsing (lint --fix repairs it later).
#[test]
fn duplicate_key_rejected() {
    let text = "---\nid: ax-un10\ntitle: Union poison\nstatus: doing\nstatus: blocked\n---\nbody\n";
    let inv = invalid("ax-un10-union-poison.md", text);
    assert!(
        inv.error.contains("duplicate"),
        "error must say duplicate key: {}",
        inv.error
    );
    assert_eq!(inv.id, "ax-un10");
}

/// Strict schema: required fields missing → invalid row, not a default.
#[test]
fn missing_required_field_is_invalid() {
    let text = "---\nid: az-nost1\ntitle: No status\n---\nbody\n";
    let inv = invalid("az-nost-no-status.md", text);
    assert!(inv.error.contains("status"), "error: {}", inv.error);
}

/// Unknown status values are schema violations, not new lifecycle states.
#[test]
fn bogus_status_is_invalid() {
    let text = "---\nid: az-bad1\ntitle: Bad status\nstatus: someday\n---\nbody\n";
    let inv = invalid("az-bad1-bad-status.md", text);
    assert!(!inv.error.is_empty());
}

/// A frontmatter id that disagrees with the filename is tolerated but warned
/// about — by-ID lookup globs on the filename prefix (DESIGN §2).
#[test]
fn filename_id_mismatch_warns() {
    let t = valid("az-zzzz-something-else.md", NORMATIVE);
    assert!(
        t.warnings.iter().any(|w| w.contains("az-zzzz")),
        "warnings: {:?}",
        t.warnings
    );
}

/// The committed corpus parses exactly as planted: alpha and beta fully
/// valid and warning-free except the one unknown-key file lives in
/// alpha-broken, which yields exactly its two invalid rows.
#[test]
fn corpus_parses_as_planted() {
    use meshwork::parse::parse_task_file;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    for repo in ["alpha", "beta"] {
        for entry in std::fs::read_dir(root.join(repo).join("meshwork/tasks")).unwrap() {
            let path = entry.unwrap().path();
            match parse_task_file(&path) {
                ParsedTask::Valid(t) => assert!(
                    t.warnings.is_empty(),
                    "{}: unplanted warnings {:?}",
                    path.display(),
                    t.warnings
                ),
                ParsedTask::Invalid(inv) => {
                    panic!("{}: unplanted invalid: {}", path.display(), inv.error)
                }
            }
        }
    }
    let mut invalid_ids = Vec::new();
    for entry in std::fs::read_dir(root.join("alpha-broken/meshwork/tasks")).unwrap() {
        if let ParsedTask::Invalid(inv) = parse_task_file(&entry.unwrap().path()) {
            invalid_ids.push(inv.id);
        }
    }
    invalid_ids.sort();
    assert_eq!(
        invalid_ids,
        ["ax-brk9", "ax-un10"],
        "exactly the planted YAML-error and union-poison rows"
    );
}
