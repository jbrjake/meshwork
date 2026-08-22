// mw-8fmsws3 (DESIGN §12b adjacent): task bodies, log lines, comments,
// and SQL cells from untrusted files render straight to the operator's
// terminal — and, via the SessionStart hook, into agent context. Raw
// ESC/CSI/OSC can spoof output or worse; controls strip at render time
// only, storage keeps bytes as written. Frontmatter can't smuggle them
// at all: YAML forbids raw controls, so such a file parses INVALID —
// loud and inert by construction.

/// A hostile fixture renders inert through every text view: no ESC, no
/// C1, no BEL, no CR survives into stdout — while the file on disk
/// still carries the original bytes.
#[test]
fn render_sanitized() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "Sneaky but valid");
    let path = task_file(&repo, &id);
    let mut text = std::fs::read_to_string(&path).unwrap();
    // Body, log, and comments live outside the frontmatter — raw
    // controls are hand-writable there and the file stays valid.
    text = text.replace(
        "## log\n",
        "body line with \x1b[2Jclear, owned \u{9b}31m C1, and a\r carriage\n\n## log\n",
    );
    text.push_str("- 2026-08-06 checkpoint \x1b[31mred\x1b[0m note\n");
    text.push_str("\n## comments\n- 2026-08-06 [mallory] comment \x1b]8;;http://x\x07link\n");
    std::fs::write(&path, text).unwrap();

    let evil = |out: &str, verb: &str| {
        for (what, needle) in [
            ("ESC", '\u{1b}'),
            ("C1 CSI", '\u{9b}'),
            ("BEL", '\u{7}'),
            ("CR", '\r'),
        ] {
            assert!(
                !out.contains(needle),
                "{what} survived into `{verb}` output:\n{out:?}"
            );
        }
    };

    let show = stdout_of(&meshwork(&repo).args(["show", &id]).assert().success());
    evil(&show, "show");
    assert!(show.contains("owned"), "text survives, controls do not: {show}");
    assert!(show.contains("checkpoint"), "{show}");

    // prime's weather carries the hostile comment; ready lists the task.
    let prime = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    evil(&prime, "prime");
    let ready = stdout_of(&meshwork(&repo).arg("ready").assert().success());
    evil(&ready, "ready");

    // Search snippets and raw SQL cells render the body text too.
    let search = stdout_of(
        &meshwork(&repo)
            .args(["search", "owned"])
            .assert()
            .success(),
    );
    evil(&search, "search");
    let q = stdout_of(
        &meshwork(&repo)
            .args(["q", "SELECT body FROM tasks"])
            .assert()
            .success(),
    );
    evil(&q, "q");

    // Storage is untouched: the file keeps its bytes as written.
    let raw = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(raw.contains('\u{1b}'), "render-time only, never storage");

    // Frontmatter controls never render because they never parse: YAML
    // forbids raw controls, the file goes INVALID, show is loud e2e.
    let evil_fm = repo.join("docs/meshwork/wo-fm3v1l1-evil-frontmatter.md");
    std::fs::write(
        &evil_fm,
        "---\nid: wo-fm3v1l1\ntitle: Evil \x1b]0;owned\x07 title\nstatus: open\n---\nbody\n",
    )
    .unwrap();
    meshwork(&repo)
        .args(["show", "wo-fm3v1l1"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("INVALID"));
}
