// e2e part-file: lint / lint --fix (PLAN 0.9). Included by e2e.rs.

/// PLAN 0.9 / MW-A4, A6, B2, B3, I2: `lint` finds every planted failure in
/// the broken corpus (golden-pinned), and `--fix` repairs exactly the
/// mechanical damage — duplicate keys, duplicate IDs, missing union
/// attributes (mw-mtn4hp8) — leaving real modeling errors for humans.
#[test]
fn lint_broken_corpus() {
    let (_g, repo) = fixture_repo("alpha-broken");
    let assert = meshwork(&repo).args(["lint", "--json"]).assert().code(1);
    let js = stdout_of(&assert);
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    let codes: Vec<&str> = v["data"]["findings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["code"].as_str().unwrap())
        .collect();
    for expected in [
        "cycle-needs",
        "cycle-parent",
        "parent-crossrepo",
        "blocked-no-reason",
        "duplicate-id",
        "duplicate-key",
        "parse",
        "dangling",
        "unknown-key",
    ] {
        assert!(codes.contains(&expected), "missing {expected}: {codes:?}");
    }
    crate::common::assert_golden("lint-broken.json", &js);

    // --fix repairs union poison + duplicate IDs, and only those.
    meshwork(&repo)
        .env("MESHWORK_ID_SEED", "42")
        .args(["lint", "--fix"])
        .assert()
        .code(1); // modeling errors (cycles, …) remain — still exit 1

    let after = stdout_of(&meshwork(&repo).args(["lint", "--json"]).assert().code(1));
    let v: serde_json::Value = serde_json::from_str(&after).unwrap();
    let codes: Vec<&str> = v["data"]["findings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["code"].as_str().unwrap())
        .collect();
    assert!(!codes.contains(&"duplicate-key"), "union poison repaired: {codes:?}");
    assert!(!codes.contains(&"duplicate-id"), "duplicate re-slugged: {codes:?}");
    assert!(codes.contains(&"cycle-needs"), "real errors stay: {codes:?}");

    // The union-poisoned file kept its first status and logged the repair
    // (the log line quotes the dropped value, so scope checks to the fm).
    let un10 = std::fs::read_to_string(task_file(&repo, "ax-un10")).unwrap();
    let fm = un10.split("\n---").next().unwrap();
    assert!(fm.contains("status: doing"), "{un10}");
    assert_eq!(fm.matches("status:").count(), 1, "one status line: {un10}");
    assert!(un10.contains("lint --fix"), "repair logged: {un10}");

    // Exactly one ax-dup1 file remains; the other carries a fresh id.
    let dup_files: Vec<_> = std::fs::read_dir(repo.join("docs/meshwork"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("ax-dup1-"))
        .collect();
    assert_eq!(dup_files.len(), 1, "one keeper: {dup_files:?}");
}

/// MW-I2 (visibility half): unparseable tasks stay loud everywhere —
/// lint, q, and show — never silently dropped.
#[test]
fn invalid_visible() {
    let (_g, repo) = fixture_repo("alpha-broken");

    let lint_out = stdout_of(&meshwork(&repo).arg("lint").assert().code(1));
    assert!(lint_out.contains("ax-brk9"), "{lint_out}");
    assert!(lint_out.contains("ax-un10"), "{lint_out}");

    let q_out = stdout_of(
        &meshwork(&repo)
            .args(["q", "SELECT id FROM tasks WHERE status='invalid' ORDER BY id"])
            .assert()
            .success(),
    );
    assert!(q_out.contains("ax-brk9") && q_out.contains("ax-un10"), "{q_out}");

    meshwork(&repo)
        .args(["show", "ax-brk9"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("INVALID"));
}

/// mw-dkwf26w: the no-verify warning covers `doing` tasks, not just open —
/// the task closest to closing is where a missing definition of done
/// matters most. The CLI can't mint this state (`start` refuses without a
/// verify), but imports and merges create it directly.
#[test]
fn lint_doing_missing_verify() {
    let (_g, repo) = git_repo("doing-no-verify");
    init_store(&repo);
    let id = add_task(&repo, "Imported mid-flight");

    // Strip the verify and flip to doing, as an import would deliver it.
    let path = task_file(&repo, &id);
    let text = std::fs::read_to_string(&path).unwrap();
    let text = text
        .lines()
        .filter(|l| !l.starts_with("verify:"))
        .map(|l| if l == "status: open" { "status: doing" } else { l })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    std::fs::write(&path, text).unwrap();

    let out = stdout_of(&meshwork(&repo).arg("lint").assert().success());
    let line = out
        .lines()
        .find(|l| l.contains("no-verify"))
        .unwrap_or_else(|| panic!("no-verify warning missing for a doing task:\n{out}"));
    assert!(line.contains(&id), "{out}");
}

/// mw-t01ek6s/mw-n3xgfs0 wiring: `lint` flags the `cat >>` damage on
/// disk, `lint --fix` relocates it above the tail with a log entry, and
/// the finding clears.
#[test]
fn lint_fix_relocates_stray_tail() {
    let (_g, repo) = git_repo("stray-tail");
    init_store(&repo);
    let id = add_task(&repo, "Damaged by cat");

    let path = task_file(&repo, &id);
    let mut text = std::fs::read_to_string(&path).unwrap();
    text.push_str("Prose appended below the tail by cat >>.\n");
    std::fs::write(&path, text).unwrap();

    let out = stdout_of(&meshwork(&repo).arg("lint").assert().success());
    assert!(
        out.lines()
            .any(|l| l.contains("stray-tail-content") && l.contains(&id)),
        "damage flagged:\n{out}"
    );

    let out = stdout_of(&meshwork(&repo).args(["lint", "--fix"]).assert().success());
    assert!(out.contains("fixed 1 file(s)"), "{out}");
    assert!(
        !out.contains("stray-tail-content"),
        "finding cleared after fix:\n{out}"
    );
    let repaired = std::fs::read_to_string(&path).unwrap();
    let body_at = repaired.find("Prose appended").expect("prose kept");
    let log_at = repaired.find("## log").expect("log section");
    assert!(body_at < log_at, "prose sits above the tail:\n{repaired}");
    assert!(
        repaired.contains("lint --fix: relocated 1 stray line"),
        "repair is logged:\n{repaired}"
    );
}

/// mw-csdzc20: repair the damage pre-fix `dep add` left behind — a
/// flow-style `needs:` line with the old block items stranded beneath it,
/// invalid YAML. The flow line carries the union, so the stray items drop
/// iff every one is already in the flow list; anything else stays for a
/// human.
#[test]
fn lint_fix_needs_collision() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let ta = add_task(&repo, "A the old dep");
    let tb = add_task(&repo, "B the new dep");
    let tx = add_task(&repo, "X the damaged one");

    meshwork(&repo)
        .args(["dep", "add", &tx, "--needs", &ta])
        .assert()
        .success();
    // Re-create an old binary's damage: flow line already unioned, the
    // old block item stranded beneath it.
    let path = task_file(&repo, &tx);
    let text = std::fs::read_to_string(&path).unwrap();
    let damaged = text.replace(
        &format!("needs: [{ta}]"),
        &format!("needs: [{ta}, {tb}]\n  - {ta}"),
    );
    std::fs::write(&path, damaged).unwrap();

    // The file is invalid YAML — lint errors on it.
    meshwork(&repo).arg("lint").assert().failure();

    let out = stdout_of(&meshwork(&repo).args(["lint", "--fix"]).assert().success());
    assert!(out.contains("fixed 1 file(s)"), "{out}");

    let repaired = std::fs::read_to_string(&path).unwrap();
    assert!(
        repaired.contains(&format!("needs: [{ta}, {tb}]")),
        "{repaired}"
    );
    assert!(!repaired.contains(&format!("  - {ta}")), "{repaired}");
    assert!(
        repaired.contains("lint --fix: dropped 1 stranded needs item"),
        "repair is logged:\n{repaired}"
    );

    // The repaired graph is real: both edges land in the tables.
    let q = stdout_of(
        &meshwork(&repo)
            .args([
                "q",
                &format!("SELECT count(*) FROM edges WHERE src_gid='work#{tx}' AND kind='needs'"),
            ])
            .assert()
            .success(),
    );
    assert!(q.contains('2'), "{q}");

    // A stray item NOT in the flow list is no mechanical repair — the
    // file stays invalid (and lint keeps failing) rather than losing an
    // edge silently.
    let text = std::fs::read_to_string(&path).unwrap();
    let damaged = text.replace(
        &format!("needs: [{ta}, {tb}]"),
        &format!("needs: [{ta}]\n  - {tb}"),
    );
    std::fs::write(&path, damaged).unwrap();
    meshwork(&repo).args(["lint", "--fix"]).assert().failure();
    let after = std::fs::read_to_string(&path).unwrap();
    assert!(
        after.contains(&format!("  - {tb}")),
        "unmatched stray kept for a human: {after}"
    );
}

/// mw-svbdkvd: a `## log` (or any heading) quoted inside a fenced code
/// block is body prose, not a section boundary — for the parser, for
/// section appends, and for the stray-tail scan. This store documents its
/// own format, so quoted tail grammar in bodies is the common case.
#[test]
fn fenced_heading_stays_body() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "Documents the tail grammar");

    let path = task_file(&repo, &id);
    let text = std::fs::read_to_string(&path).unwrap();
    let quoted = "Quoting the format:\n\n```markdown\n## log\n\
                  - 2020-01-01T00:00Z pretend entry\n\n## comments\n\
                  - pretend comment\n```\n\nAfter the fence.\n\n";
    let text = text.replacen("## log\n", &format!("{quoted}## log\n"), 1);
    std::fs::write(&path, &text).unwrap();

    // The parser keeps the whole fence (and what follows) in the body:
    // exactly one real log entry, zero comments.
    let count = |table: &str| -> i64 {
        let q = stdout_of(
            &meshwork(&repo)
                .args([
                    "q",
                    &format!("SELECT count(*) FROM {table} WHERE gid='work#{id}'"),
                    "--json",
                ])
                .assert()
                .success(),
        );
        let v: serde_json::Value = serde_json::from_str(&q).unwrap();
        v["data"]["rows"][0][0].as_i64().unwrap()
    };
    assert_eq!(count("log"), 1, "only the real created entry");
    assert_eq!(count("comments"), 0, "the fenced comment is prose");

    // Appends land in the real tail sections, below the fence — a status
    // transition (log) and a comment (comments).
    meshwork(&repo)
        .args(["start", &id, "--as", "tester"])
        .assert()
        .success();
    meshwork(&repo)
        .args(["comment", &id, "a real comment", "--as", "tester"])
        .assert()
        .success();
    let after = std::fs::read_to_string(&path).unwrap();
    let fence_close = after.rfind("```").unwrap();
    let log_entry = after.find("open→doing").expect("transition logged");
    let comment_at = after.find("a real comment").expect("comment appended");
    assert!(
        log_entry > fence_close && comment_at > fence_close,
        "appends landed inside the fence:\n{after}"
    );

    // The stray-tail scan sees nothing to relocate and lint stays clean.
    let out = stdout_of(&meshwork(&repo).arg("lint").assert().success());
    assert!(!out.contains("stray-tail-content"), "{out}");
}

/// mw-4n00yte: the migration-pressure rows fold — N legacy shell verifies
/// become one line naming `lint --explain verify-shell`, which lists them;
/// the counts and the JSON keep every finding.
#[test]
fn lint_verify_shell_folded() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let ids: Vec<String> = (0..3)
        .map(|i| add_id(&repo, &["add", &format!("Legacy {i}"), "--verify", "cargo test --lib"]))
        .collect();

    let out = stdout_of(&meshwork(&repo).arg("lint").assert().success());
    assert!(
        out.contains("3 legacy shell verifies \u{2014} lint --explain verify-shell"),
        "{out}"
    );
    assert!(!out.contains("[verify-shell]"), "per-id rows folded: {out}");
    assert!(out.contains("0 error(s), 3 warning(s)"), "the count keeps them: {out}");

    let explained = stdout_of(
        &meshwork(&repo)
            .args(["lint", "--explain", "verify-shell"])
            .assert()
            .success(),
    );
    for id in &ids {
        assert!(
            explained.contains(&format!("[verify-shell] {id}")),
            "{id} explained:\n{explained}"
        );
    }
    assert!(!explained.contains("legacy shell verifies \u{2014}"), "{explained}");

    let js = stdout_of(&meshwork(&repo).args(["lint", "--json"]).assert().success());
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    let shell = v["data"]["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["code"] == "verify-shell")
        .count();
    assert_eq!(shell, 3, "JSON keeps every finding: {v}");
}

/// mw-xb9prd6: `--explain <code>` on a heuristic prints what the heuristic
/// is and what it cannot know, above its rows.
#[test]
fn lint_explain_prints_rationale() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "Self-satisfying");
    let path = task_file(&repo, &id);
    let rel = path.strip_prefix(&repo).unwrap().to_string_lossy().into_owned();
    meshwork(&repo)
        .args(["set", &id, "--verify", &format!("contains {rel} shipped")])
        .assert()
        .success();
    let out = stdout_of(
        &meshwork(&repo)
            .args(["lint", "--explain", "verify-self-satisfying"])
            .assert()
            .success(),
    );
    assert!(out.contains("heuristic"), "{out}");
    assert!(
        out.contains(&format!("[verify-self-satisfying] {id}")),
        "{out}"
    );
    let plain = stdout_of(&meshwork(&repo).arg("lint").assert().success());
    assert!(plain.contains("[verify-self-satisfying]"), "{plain}");
}
