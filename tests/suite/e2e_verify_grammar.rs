// MW-N1 / MW-N3 at the close gate (owner ruling 2026-09-20, R-E): the
// scoping tokens reach the spawned cargo as `-p`/`--test`, and `lacks`
// refuses a missing file instead of closing on it.

/// A stub cargo that records the argv it was spawned with and reports a
/// pass — the child's env is scrubbed, so the record path is baked in.
fn recording_cargo(dir: &Path) -> (String, std::path::PathBuf) {
    use std::os::unix::fs::PermissionsExt;
    let bin = dir.join("stubbin");
    std::fs::create_dir_all(&bin).unwrap();
    let record = dir.join("argv.txt");
    let path = bin.join("cargo");
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" > \"{}\"\necho \"test result: ok. 1 passed; 0 failed\"\nexit 0\n",
            record.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    (
        format!("{}:{}", bin.display(), std::env::var("PATH").unwrap()),
        record,
    )
}

/// MW-N1: `close` spawns `cargo test -p <crate> --test <name> <filter>`
/// from the dash-free tokens; the task renders the predicate as written;
/// the dash spelling is refused at authoring, naming the token.
#[test]
fn close_run_scoped_argv() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let verify = "run cargo test package=work-core target=suite spill::";
    let id = add_id(&repo, &["add", "scoped", "--verify", verify]);
    let shown = stdout_of(&meshwork(&repo).args(["show", &id]).assert().success());
    assert!(shown.contains(&format!("verify: {verify}")), "{shown}");

    let (path, record) = recording_cargo(&repo);
    meshwork(&repo)
        .env("PATH", &path)
        .args(["close", &id])
        .assert()
        .success();
    let argv = std::fs::read_to_string(&record).unwrap();
    assert_eq!(argv.trim(), "test -p work-core --test suite spill::");

    let err = stderr_of(
        &meshwork(&repo)
            .args(["add", "dashed", "--verify", "run cargo test -p work-core"])
            .assert()
            .failure(),
    );
    assert!(err.contains("package="), "names the spelling: {err}");
}

/// MW-N3: a `lacks` whose file is missing refuses the close — absence of
/// the file is never absence of the pattern — and the task stays open.
#[test]
fn lacks_missing_path_refuses() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_id(&repo, &["add", "gone marker", "--verify", "lacks STATUS.md PENDING"]);
    let err = stderr_of(&meshwork(&repo).args(["close", &id]).assert().failure());
    assert!(err.contains("STATUS.md") && err.contains("missing"), "{err}");
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(text.contains("status: open"), "{text}");

    std::fs::write(repo.join("STATUS.md"), "- 2026-09-01 PENDING review\n").unwrap();
    let err = stderr_of(&meshwork(&repo).args(["close", &id]).assert().failure());
    assert!(err.contains("found"), "{err}");

    std::fs::write(repo.join("STATUS.md"), "- 2026-09-01 done\n").unwrap();
    meshwork(&repo).args(["close", &id]).assert().success();
}
