// mw-17hnhzk: nested checkboxes were folded into parent bodies as prose —
// exit 0, plausible count, no warning; the sazed pilot lost 15 of 124
// items that way, including open work entombed inside a done parent that
// auto-archived. Nested checkboxes are REAL tasks now: `parent:` from the
// enclosing checkbox (any depth, MW-B8), status from their OWN marker.
// Silent loss is the one forbidden outcome.

/// Every checkbox imports as a task; nesting becomes parent edges.
#[test]
fn import_nested_checkboxes() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    std::fs::write(
        repo.join("TODO.md"),
        "# TODO\n\n## Now\n\n\
         - [ ] **Top parent** — has children.\n\
         \x20 - [x] **Child done**\n\
         \x20 - [ ] **Child open**\n\
         \x20   - [ ] **Grandchild open**\n\
         - [x] **Done parent** — closed at import.\n\
         \x20 - [ ] **Entombed open child** — the pilot's exact failure.\n\
         \x20     verify: `true` exits 0\n",
    )
    .unwrap();

    let out = stdout_of(
        &meshwork(&repo)
            .args(["import", "todo", "TODO.md"])
            .assert()
            .success(),
    );
    assert!(out.contains("6 imported"), "every checkbox is a task: {out}");
    assert!(out.contains("4 nested"), "nesting is reported: {out}");

    let q = |sql: &str| stdout_of(&meshwork(&repo).args(["q", sql]).assert().success());
    assert!(q("SELECT count(*) FROM tasks").contains('6'), "no drops");
    assert!(
        q("SELECT count(*) FROM edges WHERE kind='parent'").contains('4'),
        "each nested checkbox carries a parent edge"
    );

    // Status comes from the child's own marker — the entombed-open case:
    // an open child of a done (archived) parent stays open and visible.
    let rows = q(
        "SELECT t.status FROM tasks t JOIN edges e ON e.src_gid = t.gid \
         JOIN tasks p ON e.dst_gid = p.gid \
         WHERE e.kind = 'parent' AND p.status = 'done'",
    );
    assert!(rows.contains("open"), "open child of done parent: {rows}");
    let ready = stdout_of(&meshwork(&repo).arg("ready").assert().success());
    assert!(
        ready.contains("Entombed open child"),
        "the pilot's lost item is actionable, not entombed: {ready}"
    );

    // Depth works (MW-B8): the grandchild's parent is the middle child.
    let deep = q(
        "SELECT p.title FROM tasks t JOIN edges e ON e.src_gid = t.gid \
         JOIN tasks p ON e.dst_gid = p.gid \
         WHERE t.title = 'Grandchild open' AND e.kind = 'parent'",
    );
    assert!(deep.contains("Child open"), "{deep}");

    // A continuation line under a nested item attaches to THAT item.
    assert!(
        q("SELECT verify FROM tasks WHERE title = 'Entombed open child'").contains("true"),
        "nested verify: attaches to the nested task"
    );

    // And the forbidden outcome: no child title folded into a parent body.
    let parent_file = task_file(&repo, &first_id_titled(&repo, "Top parent"));
    let body = std::fs::read_to_string(parent_file).unwrap();
    assert!(
        !body.contains("Child open") && !body.contains("Child done"),
        "children are tasks, never prose: {body}"
    );
}

// mw-mrjhwws: hard-wrapped checkbox lines truncated titles at the wrap and
// multiline verifies at their first line — the wrapped remainder leaked
// into body prose. Continuation lines join their item before field
// extraction, markdown-paragraph style: a blank line ends the join.

/// Wrapped titles and multiline verifies import whole.
#[test]
fn import_wrapped_titles_and_multiline_verifies() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    std::fs::write(
        repo.join("TODO.md"),
        "# TODO\n\n## Now\n\n\
         - [ ] **Spillway rebuild: the engine landed late (owner-\n\
         \x20 ruled follow-through)** — wrap context tail.\n\
         - [ ] Plain wrapped title that continues\n\
         \x20 onto a second line\n\
         - [ ] **Multi verify task**\n\
         \x20 verify: `out=$(cargo test import_smoke 2>&1) &&\n\
         \x20   echo \"$out\" | grep -q ok` exits 0\n\
         - [ ] **Boundary task**\n\
         \n\
         \x20 Body paragraph, not title.\n",
    )
    .unwrap();

    let out = stdout_of(
        &meshwork(&repo)
            .args(["import", "todo", "TODO.md"])
            .assert()
            .success(),
    );
    assert!(out.contains("4 imported"), "{out}");

    let q = |sql: &str| stdout_of(&meshwork(&repo).args(["q", sql]).assert().success());
    let titles = q("SELECT title FROM tasks ORDER BY seq");
    assert!(
        titles.contains("follow-through)"),
        "bold title joined across the wrap: {titles}"
    );
    assert!(
        titles.contains("continues onto a second line"),
        "plain title joined across the wrap: {titles}"
    );

    // The headline's own context survives the join.
    let body = std::fs::read_to_string(task_file(
        &repo,
        &first_id_titled(&repo, "Spillway rebuild: the engine landed late (owner- ruled follow-through)"),
    ))
    .unwrap();
    assert!(body.contains("wrap context tail"), "{body}");

    // Multiline verify: the whole command, not its first physical line.
    let verify = q("SELECT verify FROM tasks WHERE title = 'Multi verify task'");
    assert!(
        verify.contains("grep -q ok"),
        "verify joined across the wrap: {verify}"
    );

    // A blank line ends the headline: paragraph stays body, title stays put.
    assert!(
        q("SELECT count(*) FROM tasks WHERE title = 'Boundary task'").contains('1'),
        "blank line ends the title join"
    );
    let boundary =
        std::fs::read_to_string(task_file(&repo, &first_id_titled(&repo, "Boundary task"))).unwrap();
    assert!(boundary.contains("Body paragraph, not title."), "{boundary}");
}

// mw-gsgh8s7: column-0 prose outside any checkbox — preambles,
// interstitial section notes, trailing ledgers — vanished with exit 0.
// A whole asks-section disappeared that way in a real migration. Now it
// carries whole into one clearly-marked triage task, loudly counted;
// silent drops are the one forbidden outcome.

/// Non-checkbox prose imports whole into a triage task.
#[test]
fn import_prose() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    std::fs::write(
        repo.join("TODO.md"),
        "# TODO\n\n\
         Preamble: how this file works.\n\n\
         ## Now\n\n\
         - [ ] **Real task** — with context.\n\n\
         Interstitial note between sections.\n\n\
         ## Asks ledger\n\n\
         ask 8: the door check needs a matrix row.\n\
         ask 9: spill budget wants a bench.\n",
    )
    .unwrap();

    let out = stdout_of(
        &meshwork(&repo)
            .args(["import", "todo", "TODO.md"])
            .assert()
            .success(),
    );
    // The checkbox task + the triage task, and the carry is loud.
    assert!(out.contains("2 imported"), "{out}");
    assert!(
        out.contains("4 prose line(s) carried"),
        "the carry is counted, never silent: {out}"
    );

    let q = |sql: &str| stdout_of(&meshwork(&repo).args(["q", sql]).assert().success());
    let titles = q("SELECT title FROM tasks");
    assert!(titles.contains("triage"), "{titles}");

    let body = std::fs::read_to_string(task_file(
        &repo,
        &first_id_titled(&repo, "Imported prose needing triage (TODO.md)"),
    ))
    .unwrap();
    for line in [
        "Preamble: how this file works.",
        "Interstitial note between sections.",
        "## Asks ledger",
        "ask 8: the door check needs a matrix row.",
        "ask 9: spill budget wants a bench.",
    ] {
        assert!(body.contains(line), "carried whole, missing {line:?}: {body}");
    }
    // The checkbox task keeps its own body clean of carried prose.
    let real = std::fs::read_to_string(task_file(&repo, &first_id_titled(&repo, "Real task"))).unwrap();
    assert!(!real.contains("Interstitial"), "{real}");

    // A prose-free TODO mints no triage task and stays quiet.
    std::fs::write(
        repo.join("CLEAN.md"),
        "# TODO\n\n## Now\n\n- [ ] **Only checkboxes here**\n",
    )
    .unwrap();
    let out = stdout_of(
        &meshwork(&repo)
            .args(["import", "todo", "CLEAN.md"])
            .assert()
            .success(),
    );
    assert!(out.contains("1 imported"), "{out}");
    assert!(!out.contains("carried"), "no phantom triage: {out}");
}

// mw-6mqm4em: sazed imported tasks titled just R11, R8, R7 — codes, not
// work orders — unintelligible in every listing three days later. A
// single-token title warns per line and in the summary so the review pass
// retitles it; the import itself still succeeds (warn, never block).

/// Single-token titles warn per line on stderr plus a summary count.
#[test]
fn import_short_title() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    std::fs::write(
        repo.join("TODO.md"),
        "# TODO\n\n## Now\n\n\
         - [ ] **R11**\n\
         - [ ] **Fix the door check** — a real work order.\n\
         - [x] R8\n",
    )
    .unwrap();

    let assert = meshwork(&repo)
        .args(["import", "todo", "TODO.md"])
        .assert()
        .success();
    let out = stdout_of(&assert);
    let err = stderr_of(&assert);

    // Per-title stderr warning names the minted id — the retitle handle.
    let r11 = first_id_titled(&repo, "R11");
    assert!(
        err.contains(&r11) && err.contains("R11"),
        "warning names id + title: {err}"
    );
    // Terminal imports warn too — archived tasks still surface in queries.
    assert!(err.contains("R8"), "done imports warn as well: {err}");
    assert!(
        !err.contains("door check"),
        "multi-word titles never warn: {err}"
    );

    // Summary count lands in the stdout block, carried_n-style.
    assert!(
        out.contains("2 single-token title(s)"),
        "summary counts the warns: {out}"
    );

    // A TODO of real work orders imports without the warning noise.
    std::fs::write(
        repo.join("CLEAN.md"),
        "# TODO\n\n## Now\n\n- [ ] **Rebuild the spillway door**\n",
    )
    .unwrap();
    let assert = meshwork(&repo)
        .args(["import", "todo", "CLEAN.md"])
        .assert()
        .success();
    assert!(
        !stdout_of(&assert).contains("single-token")
            && !stderr_of(&assert).contains("single-token"),
        "no phantom warnings on clean imports"
    );
}

// mw-x5a8g9w: [~] used to mint `status: doing` with no claimant — the
// leras import seeded instant doing-rot that way (mw-06j1wqe measured it:
// still stale five days on, across all three repos). doing without a
// claimant is a lie at import time. [~] now imports as open with the
// source marker preserved in the log, counted loudly in the summary.

/// `[~]` imports as open + a log note, never an unclaimed doing row.
#[test]
fn import_marker_doing() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    std::fs::write(
        repo.join("TODO.md"),
        "# TODO\n\n## Now\n\n\
         - [~] **Fix the spill cliff** — was mid-flight at export.\n\
         - [ ] **Untouched open item**\n",
    )
    .unwrap();

    let out = stdout_of(
        &meshwork(&repo)
            .args(["import", "todo", "TODO.md"])
            .assert()
            .success(),
    );
    assert!(out.contains("2 imported"), "{out}");
    // The downgrade is loud, carried_n-style, in the summary block.
    assert!(
        out.contains("1 [~] item(s) imported as open"),
        "downgrade is counted, never silent: {out}"
    );

    let q = |sql: &str| stdout_of(&meshwork(&repo).args(["q", sql]).assert().success());
    assert!(
        q("SELECT count(*) FROM tasks WHERE status = 'doing'").contains('0'),
        "no unclaimed doing rows minted"
    );
    assert!(
        q("SELECT status FROM tasks WHERE title = 'Fix the spill cliff'").contains("open"),
        "the [~] item lands open"
    );

    // The source marker survives in the log — the file remembers.
    let body = std::fs::read_to_string(task_file(
        &repo,
        &first_id_titled(&repo, "Fix the spill cliff"),
    ))
    .unwrap();
    assert!(body.contains("[~]"), "log notes the source marker: {body}");
    // A plain open import keeps its log free of the note.
    let clean = std::fs::read_to_string(task_file(
        &repo,
        &first_id_titled(&repo, "Untouched open item"),
    ))
    .unwrap();
    assert!(!clean.contains("[~]"), "no phantom marker note: {clean}");
}

/// Look up a task id by exact title via SQL.
fn first_id_titled(repo: &Path, title: &str) -> String {
    let out = stdout_of(
        &meshwork(repo)
            .args([
                "q",
                &format!("SELECT id FROM tasks WHERE title = '{title}'"),
            ])
            .assert()
            .success(),
    );
    out.lines()
        .nth(1)
        .unwrap_or_else(|| panic!("no task titled {title}: {out}"))
        .trim()
        .to_string()
}
