// `e2e::archive_on_close` — mw-45e2qf4: terminal tasks (done/dropped) move
// to docs/meshwork/archive/ automagically; archived tasks stay fully
// queryable (owner-confirmed) — only the file's location changes.

/// close/drop archive the file, reopen un-archives it; deps on archived
/// done tasks still count as met; lint --fix sweeps misplaced files.
#[test]
fn archive_on_close() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let store = repo.join("docs/meshwork");

    // close → file moves to archive/, root copy gone.
    let a = add_task(&repo, "Archived on close");
    meshwork(&repo).args(["close", &a]).assert().success();
    let archived = repo.join(format!("docs/meshwork/archive/{a}-archived-on-close.md"));
    assert!(archived.is_file(), "moved to archive");
    assert!(
        !store.join(format!("{a}-archived-on-close.md")).exists(),
        "gone from root"
    );

    // Still fully queryable: status done, and a dep on it counts as met.
    let b = add_id(&repo, &["add", "Depends on archived", "--needs", &a, "--verify", "true"]);
    let ready = stdout_of(&meshwork(&repo).arg("ready").assert().success());
    assert!(ready.contains(&b), "dep on archived done task is met:\n{ready}");
    let q = stdout_of(
        &meshwork(&repo)
            .args(["q", "SELECT id, status FROM tasks ORDER BY id"])
            .assert()
            .success(),
    );
    assert!(q.contains(&a) && q.contains("done"), "archived row queryable:\n{q}");

    // show still finds it by id, and prime's recently-done still lists it.
    meshwork(&repo).args(["show", &a]).assert().success();
    let prime = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    assert!(prime.contains(&a), "recently done sees archive:\n{prime}");

    // reopen → file moves back to the store root.
    meshwork(&repo).args(["reopen", &a]).assert().success();
    assert!(
        store.join(format!("{a}-archived-on-close.md")).is_file(),
        "reopen un-archives"
    );
    assert!(!archived.exists(), "archive copy gone after reopen");

    // drop archives too.
    meshwork(&repo).args(["drop", &a]).assert().success();
    assert!(archived.is_file(), "dropped task archived");

    // A hand-misplaced terminal task: lint warns, --fix moves it.
    let stray = store.join("wo-stray99-hand-closed.md");
    std::fs::write(
        &stray,
        "---\nid: wo-stray99\ntitle: Hand closed\nstatus: done\nverify: \"true\"\n---\n\n## log\n- 2026-08-06 created\n",
    )
    .unwrap();
    let lint = stdout_of(&meshwork(&repo).arg("lint").assert().success());
    assert!(lint.contains("misplaced"), "lint warns on stray:\n{lint}");
    meshwork(&repo).args(["lint", "--fix"]).assert().success();
    assert!(!stray.exists(), "--fix moved the stray");
    assert!(
        store.join("archive/wo-stray99-hand-closed.md").is_file(),
        "stray now archived"
    );

    // Minting collision-checks the archive: an archived id is never reused.
    let expected = meshwork::id::IdGen::with_seed(7).next_id("wo");
    std::fs::write(
        store.join(format!("archive/{expected}-taken.md")),
        format!("---\nid: {expected}\ntitle: Taken\nstatus: done\nverify: \"true\"\n---\n"),
    )
    .unwrap();
    let minted = stdout_of(
        &meshwork(&repo)
            .env("MESHWORK_ID_SEED", "7")
            .args(["add", "Fresh", "--verify", "true"])
            .assert()
            .success(),
    );
    let minted_id = minted.lines().next().unwrap();
    assert_ne!(minted_id, expected, "archived id forces a re-roll");
}
