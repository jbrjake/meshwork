// `e2e::archive_on_close` — mw-45e2qf4: terminal tasks (done/dropped) move
// to docs/meshwork/archive/ automagically; archived tasks stay fully
// queryable (owner-confirmed) — only the file's location changes.

/// show prints the path the store actually holds: `archive/` once close
/// has moved the file, the root again after reopen — text and JSON alike.
#[test]
fn show_archived_file_path() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "Archived path");
    let root_path = format!("docs/meshwork/{id}-archived-path.md");
    let archived = format!("docs/meshwork/archive/{id}-archived-path.md");

    let shown = stdout_of(&meshwork(&repo).args(["show", &id]).assert().success());
    assert!(shown.contains(&format!("file: {root_path}")), "{shown}");

    meshwork(&repo).args(["close", &id]).assert().success();
    let shown = stdout_of(&meshwork(&repo).args(["show", &id]).assert().success());
    assert!(shown.contains(&format!("file: {archived}")), "{shown}");
    assert!(!shown.contains(&format!("file: {root_path}")), "{shown}");
    let js = stdout_of(
        &meshwork(&repo)
            .args(["show", &id, "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    assert_eq!(v["data"]["path"], archived, "{v}");

    meshwork(&repo).args(["reopen", &id]).assert().success();
    let shown = stdout_of(&meshwork(&repo).args(["show", &id]).assert().success());
    assert!(shown.contains(&format!("file: {root_path}")), "{shown}");
}

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

/// mw-bvxpeef: past the loose-file threshold, lint warns and `lint --fix`
/// concatenates archived singles into size-capped bundles. Bundled tasks
/// stay queryable, showable, dep-resolvable and commentable; `reopen`
/// splits one back out into a live file; the store declares format 2
/// once a bundle exists; below the threshold nothing is touched.
#[test]
fn archive_compact() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let store = repo.join("docs/meshwork");
    let archive = store.join("archive");
    std::fs::create_dir_all(&archive).unwrap();
    // 120 archived singles of ~5KB: past the threshold, and past one
    // bundle's byte cap, so the second bundle opens.
    let filler = "history that nobody will trim, kept whole. ".repeat(110);
    for n in 1..=120 {
        let id = format!("wo-c{n:04}");
        std::fs::write(
            archive.join(format!("{id}-closed-thing-{n}.md")),
            format!(
                "---\nid: {id}\ntitle: Closed thing {n}\nstatus: done\nverify: exists docs/meshwork/config.toml\n---\n\
                 {filler}\n\n```\n---\nid: not-a-boundary\n```\n\n---\n\n\
                 ## log\n- 2026-08-01 created\n- 2026-08-02 open\u{2192}done\n"
            ),
        )
        .unwrap();
    }
    let live = add_id(&repo, &["add", "Needs a bundled one", "--needs", "wo-c0007", "--verify", "true"]);

    let lint = stdout_of(&meshwork(&repo).arg("lint").assert().success());
    assert!(
        lint.contains("archive-loose") && lint.contains("120") && lint.contains("lint --fix"),
        "past the threshold lint names the repair:\n{lint}"
    );

    let fixed = stdout_of(&meshwork(&repo).args(["lint", "--fix"]).assert().success());
    assert!(fixed.contains("120"), "the fix reports what it bundled:\n{fixed}");
    let names: Vec<String> = std::fs::read_dir(&archive)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        names.iter().all(|n| n.starts_with("bundle-")),
        "no loose singles remain: {names:?}"
    );
    assert!(
        names.contains(&"bundle-0001.md".to_string())
            && names.contains(&"bundle-0002.md".to_string()),
        "the byte cap opened a second bundle: {names:?}"
    );
    let first = std::fs::metadata(archive.join("bundle-0001.md")).unwrap().len();
    assert!(first <= 512 * 1024, "bundle-0001 is within the cap: {first}");
    let cfg = std::fs::read_to_string(store.join("config.toml")).unwrap();
    assert!(cfg.contains("format = 2"), "a bundle makes the store format 2:\n{cfg}");

    // Still fully loaded: every row, the dep met, show by id with the
    // bundle as its file, a comment spliced into the bundle.
    let q = stdout_of(
        &meshwork(&repo)
            .args(["q", "SELECT count(*) AS n FROM tasks WHERE status = 'done'", "--json"])
            .assert()
            .success(),
    );
    assert!(q.contains("[[120]]"), "{q}");
    let ready = stdout_of(&meshwork(&repo).arg("ready").assert().success());
    assert!(ready.contains(&live), "a dep on a bundled done task is met:\n{ready}");
    let shown = stdout_of(&meshwork(&repo).args(["show", "wo-c0007"]).assert().success());
    assert!(
        shown.contains("Closed thing 7") && shown.contains("file: docs/meshwork/archive/bundle-0001.md"),
        "{shown}"
    );
    meshwork(&repo)
        .args(["comment", "wo-c0007", "a note on a bundled task", "--as", "tester"])
        .assert()
        .success();
    let shown = stdout_of(&meshwork(&repo).args(["show", "wo-c0007"]).assert().success());
    assert!(shown.contains("a note on a bundled task"), "{shown}");
    let bundle = std::fs::read_to_string(archive.join("bundle-0001.md")).unwrap();
    assert_eq!(
        bundle.matches("\nid: wo-c").count(),
        bundle.matches("\ntitle: Closed thing").count(),
        "the splice kept every document whole"
    );
    let lint = stdout_of(&meshwork(&repo).arg("lint").assert().success());
    assert!(lint.contains("0 error(s)") && !lint.contains("archive-loose"), "{lint}");

    // reopen splits the document back out into a live file in the root.
    meshwork(&repo).args(["reopen", "wo-c0007"]).assert().success();
    let reopened = store.join("wo-c0007-closed-thing-7.md");
    assert!(reopened.is_file(), "reopen wrote a live single: {reopened:?}");
    let bundle = std::fs::read_to_string(archive.join("bundle-0001.md")).unwrap();
    assert!(!bundle.contains("id: wo-c0007\n"), "the bundle no longer holds it");
    let q = stdout_of(
        &meshwork(&repo)
            .args(["q", "SELECT count(*) AS n FROM tasks WHERE status = 'done'", "--json"])
            .assert()
            .success(),
    );
    assert!(q.contains("[[119]]"), "{q}");
    let text = std::fs::read_to_string(&reopened).unwrap();
    assert!(text.contains("a note on a bundled task") && text.contains("done\u{2192}open"), "{text}");

    // Below the threshold a fresh close stays a loose single, and lint is quiet about it.
    meshwork(&repo).args(["close", "wo-c0007"]).assert().success();
    assert!(archive.join("wo-c0007-closed-thing-7.md").is_file(), "loose again");
    let lint = stdout_of(&meshwork(&repo).arg("lint").assert().success());
    assert!(lint.contains("0 error(s)") && !lint.contains("archive-loose"), "{lint}");
    meshwork(&repo).args(["lint", "--fix"]).assert().success();
    assert!(
        archive.join("wo-c0007-closed-thing-7.md").is_file(),
        "--fix below the threshold leaves the archive alone"
    );
}
