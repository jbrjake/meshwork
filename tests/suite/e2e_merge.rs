// e2e part-file: merge scenarios 1–3 (PLAN 0.10; DESIGN §13). Real git,
// two clones of a shared origin, union-attribute merges. Included by e2e.rs.

/// Bare origin + configured clone `a` on branch main. Deterministic HEAD:
/// origin's symbolic-ref is set explicitly so later clones check out main.
fn origin_and_clone_a() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    git(&root, &["init", "--bare", "-q", "origin.git"]);
    git(&root.join("origin.git"), &["symbolic-ref", "HEAD", "refs/heads/main"]);
    git(&root, &["clone", "-q", "origin.git", "a"]);
    let a = root.join("a");
    configure_git_user(&a);
    git(&a, &["checkout", "-q", "-b", "main"]);
    (dir, a)
}

fn configure_git_user(repo: &Path) {
    git(repo, &["config", "user.name", "Fixture User"]);
    git(repo, &["config", "user.email", "fixture@example.invalid"]);
}

fn clone_b(root: &Path) -> std::path::PathBuf {
    git(root, &["clone", "-q", "origin.git", "b"]);
    let b = root.join("b");
    configure_git_user(&b);
    b
}

fn commit_push(repo: &Path, msg: &str) {
    git(repo, &["add", "-A"]);
    git(repo, &["commit", "-q", "-m", msg]);
    git(repo, &["push", "-q", "-u", "origin", "main"]);
}

fn commit_only(repo: &Path, msg: &str) {
    git(repo, &["add", "-A"]);
    git(repo, &["commit", "-q", "-m", msg]);
}

fn merge_origin(repo: &Path) {
    git(repo, &["fetch", "-q", "origin"]);
    git(repo, &["merge", "-q", "--no-edit", "origin/main"]);
}

fn append(path: &Path, text: &str) {
    let mut current = std::fs::read_to_string(path).unwrap();
    current.push_str(text);
    std::fs::write(path, current).unwrap();
}

/// Scenario 1 (MW-I1): two clones create tasks, close tasks, and append
/// comments to the SAME task; merge produces zero conflict markers, both
/// comments survive, lint stays clean.
#[test]
fn merge_concurrent_worktrees() {
    let (dir, a) = origin_and_clone_a();
    init_store(&a);
    let shared = add_task(&a, "Shared investigation");
    // Seed the comments section in base so both sides purely append lines.
    append(
        &task_file(&a, &shared),
        "\n## comments\n- 2026-08-04 [jon] baseline note\n",
    );
    commit_push(&a, "base");
    let b = clone_b(dir.path());

    // A: create + close a task, comment on the shared one.
    let a_task = add_task(&a, "A side work");
    meshwork(&a).args(["close", &a_task]).assert().success();
    append(
        &task_file(&a, &shared),
        "- 2026-08-04 [claude/aaaa1111] observed the cliff on branch A\n",
    );
    commit_push(&a, "a work");

    // B: create a task, comment on the same shared task.
    let b_task = add_task(&b, "B side work");
    append(
        &task_file(&b, &shared),
        "- 2026-08-04 [claude/bbbb2222] measured baseline on branch B\n",
    );
    commit_only(&b, "b work");
    merge_origin(&b);

    let merged = std::fs::read_to_string(task_file(&b, &shared)).unwrap();
    assert!(!merged.contains("<<<<<<<"), "no conflict markers:\n{merged}");
    assert!(merged.contains("observed the cliff on branch A"), "{merged}");
    assert!(merged.contains("measured baseline on branch B"), "{merged}");

    meshwork(&b).arg("lint").assert().success();
    let shown = stdout_of(&meshwork(&b).args(["show", &a_task]).assert().success());
    assert!(shown.contains("done"), "A's close survived the merge");
    meshwork(&b).args(["show", &b_task]).assert().success();
}

/// Scenario 2 (MW-A4): seeded RNG forces the same ID in both clones;
/// lint detects the post-merge duplicate, --fix re-slugs, lint goes clean,
/// and existing references still resolve.
#[test]
fn merge_duplicate_id() {
    let (dir, a) = origin_and_clone_a();
    init_store(&a);
    commit_push(&a, "base");
    let b = clone_b(dir.path());

    let dup = stdout_of(
        &meshwork(&a)
            .env("MESHWORK_ID_SEED", "99")
            .args(["add", "Alpha side task", "--verify", "true"])
            .assert()
            .success(),
    )
    .lines()
    .next()
    .unwrap()
    .to_string();
    // A also references its own mint — the reference must survive the fix.
    let referencer = add_id(
        &a,
        &["add", "References dup", "--needs", &dup, "--verify", "true"],
    );
    commit_push(&a, "a mints");

    let dup_b = stdout_of(
        &meshwork(&b)
            .env("MESHWORK_ID_SEED", "99")
            .args(["add", "Zulu side task", "--verify", "true"])
            .assert()
            .success(),
    )
    .lines()
    .next()
    .unwrap()
    .to_string();
    assert_eq!(dup, dup_b, "seeded clones mint the same id (the collision)");
    commit_only(&b, "b mints");
    merge_origin(&b); // different filenames — merges clean, ids now collide

    let lint1 = stdout_of(&meshwork(&b).arg("lint").assert().code(1));
    assert!(lint1.contains("duplicate-id"), "{lint1}");

    meshwork(&b).args(["lint", "--fix"]).assert().success();
    meshwork(&b).arg("lint").assert().success();

    // Keeper is the earliest (same created date → filename order): A's
    // "alpha-side-task" file. The reference points at it, as it always did.
    let shown = stdout_of(&meshwork(&b).args(["show", &dup]).assert().success());
    assert!(shown.contains("Alpha side task"), "keeper: {shown}");
    let refshown = stdout_of(&meshwork(&b).args(["show", &referencer]).assert().success());
    assert!(refshown.contains(&dup), "reference intact: {refshown}");

    // Both tasks still exist, now under distinct ids.
    let ids = stdout_of(
        &meshwork(&b)
            .args(["q", "SELECT count(DISTINCT id) FROM tasks"])
            .assert()
            .success(),
    );
    assert!(ids.contains('3'), "{ids}");
}

/// Scenario 3 (MW-I2): both clones edit the same status line; union merge
/// turns it into a duplicate key; the row surfaces as invalid — never
/// silently dropped — and lint --fix repairs it.
#[test]
fn merge_union_poison() {
    let (dir, a) = origin_and_clone_a();
    init_store(&a);
    let shared = add_task(&a, "Contested task");
    commit_push(&a, "base");
    let b = clone_b(dir.path());

    meshwork(&a).args(["start", &shared]).assert().success();
    commit_push(&a, "a starts");

    meshwork(&b)
        .args(["block", &shared, "--reason", "waiting for repro data"])
        .assert()
        .success();
    commit_only(&b, "b blocks");
    merge_origin(&b);

    // Poisoned: duplicate status key → strict parse rejects → invalid row.
    let q_out = stdout_of(
        &meshwork(&b)
            .args(["q", "SELECT id, status FROM tasks WHERE status='invalid'"])
            .assert()
            .success(),
    );
    assert!(q_out.contains(&shared), "invalid row visible: {q_out}");
    let lint1 = stdout_of(&meshwork(&b).arg("lint").assert().code(1));
    assert!(lint1.contains("duplicate-key"), "{lint1}");

    meshwork(&b).args(["lint", "--fix"]).assert().success();
    meshwork(&b).arg("lint").assert().success();

    let repaired = std::fs::read_to_string(task_file(&b, &shared)).unwrap();
    let fm = repaired.split("\n---").next().unwrap();
    assert_eq!(fm.matches("\nstatus:").count(), 1, "one status line:\n{repaired}");
    assert!(repaired.contains("open→doing"), "A's log entry survives");
    assert!(repaired.contains("open→blocked"), "B's log entry survives");
    assert!(repaired.contains("lint --fix"), "repair is logged");
}
