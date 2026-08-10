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
