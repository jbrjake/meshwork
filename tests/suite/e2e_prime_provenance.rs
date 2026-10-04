// mw-3jwwh5d: prime stamps store provenance — HEAD short-sha + uncommitted
// task-edit count (+ ahead-of-upstream when one exists), scoped to
// docs/meshwork/. An incoming session sees staleness up front instead of
// discovering it mid-work. Degrades silently when git info is unavailable.

#[test]
fn prime_provenance_line_present() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "committed work");
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-qm", "chore(store): seed"]);

    // A clean store: sha line, no uncommitted segment.
    let prime = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    assert!(prime.contains("store @ "), "{prime}");
    assert!(!prime.contains("uncommitted"), "{prime}");

    // Dirty one task file — the count appears.
    let path = task_file(&repo, &id);
    let text = std::fs::read_to_string(&path).unwrap() + "\ndirty edit\n";
    std::fs::write(&path, text).unwrap();
    let prime = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    assert!(prime.contains("store @ "), "{prime}");
    assert!(prime.contains("1 uncommitted task edit"), "{prime}");
}

/// A store whose directory has never been committed is one untracked
/// directory to git, so every new task file read as one edit; the first
/// close did the same for the new archive directory. Each task file
/// counts, and only task files count — config and shim are not edits.
#[test]
fn prime_counts_each_untracked_task_file() {
    let (_g, repo) = git_repo("work");
    std::fs::write(repo.join("README.md"), "seed\n").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-qm", "chore: seed"]);
    init_store(&repo);
    let one = add_task(&repo, "first");
    add_task(&repo, "second");
    let prime = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    assert!(prime.contains("2 uncommitted task edits"), "{prime}");

    // The first close moves one file into the new archive directory:
    // still two task files, now in two untracked directories.
    meshwork(&repo).args(["close", &one]).assert().success();
    let prime = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    assert!(prime.contains("2 uncommitted task edits"), "{prime}");

    // close's anchor counts the whole tree the same way: file by file.
    let text = std::fs::read_to_string(task_file(&repo, &one)).unwrap();
    let anchor = text
        .lines()
        .find(|l| l.contains("→done"))
        .and_then(|l| l.rsplit('+').next())
        .and_then(|n| n.trim().parse::<usize>().ok())
        .unwrap_or_else(|| panic!("no @ sha+N anchor: {text}"));
    let out = std::process::Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=all"])
        .current_dir(&repo)
        .output()
        .unwrap();
    let files = String::from_utf8_lossy(&out.stdout).lines().count();
    assert!(files > 2, "the store alone is several files: {files}");
    assert_eq!(anchor, files, "{text}");
}

#[test]
fn prime_provenance_degrades_silently() {
    // No commits yet: HEAD is unborn, git info unavailable — the line is
    // omitted and prime still works (MW-D5: never fail the digest).
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    add_task(&repo, "uncommitted era");
    let prime = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    assert!(!prime.contains("store @"), "{prime}");
    assert!(prime.contains("open"), "digest still renders: {prime}");
}
