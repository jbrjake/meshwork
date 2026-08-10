// mw-af4kbjy: `add --batch <file|->` — several §2 task documents in one
// atomic operation, local `handle:` names usable anywhere an id is; meshwork
// mints ids and rewrites the refs. Partial failure writes nothing.

const BATCH: &str = "\
---
handle: parent
title: Verify security epic
category: core/verify
---
Umbrella for the verify-as-untrusted-input sequence.
---
handle: grammar
title: Define the verify grammar
category: core/verify
parent: @parent
verify: \"true\"
---
---
title: Wire grammar into close
category: core/verify
parent: @parent
needs: [@grammar]
verify: \"true\"
---
";

#[test]
fn add_batch_mints_ids_and_wires_handles() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);

    let out = stdout_of(
        &meshwork(&repo)
            .args(["add", "--batch", "-"])
            .write_stdin(BATCH)
            .assert()
            .success(),
    );
    let ids: Vec<&str> = out
        .lines()
        .filter(|l| !l.starts_with(' '))
        .collect();
    assert_eq!(ids.len(), 3, "{out}");

    // Handles resolved to the minted ids; the handle key never persists.
    let child = std::fs::read_to_string(task_file(&repo, ids[2])).unwrap();
    assert!(child.contains(&format!("parent: {}", ids[0])), "{child}");
    assert!(child.contains(&format!("needs: [{}]", ids[1])), "{child}");
    assert!(!child.contains('@') && !child.contains("handle:"), "{child}");

    // The wired graph is real: the grammar task blocks its sibling.
    let why = stdout_of(&meshwork(&repo).args(["why", ids[2]]).assert().success());
    assert!(why.contains(ids[1]), "{why}");

    // Defaults injected like plain `add`: open + created + a created log line.
    let grammar = std::fs::read_to_string(task_file(&repo, ids[1])).unwrap();
    assert!(grammar.contains("status: open"), "{grammar}");
    assert!(grammar.contains("created: "), "{grammar}");
    assert!(grammar.contains("## log"), "{grammar}");
}

#[test]
fn add_batch_dry_run_writes_nothing() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let before = crate::common::file_inventory(&repo);

    let out = stdout_of(
        &meshwork(&repo)
            .args(["add", "--batch", "-", "--dry-run"])
            .write_stdin(BATCH)
            .assert()
            .success(),
    );
    assert!(out.contains("title: Define the verify grammar"), "{out}");
    assert!(!out.contains('@'), "handles must be resolved even dry: {out}");
    assert_eq!(before, crate::common::file_inventory(&repo));
}

#[test]
fn add_batch_unknown_handle_is_atomic() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let before = crate::common::file_inventory(&repo);

    meshwork(&repo)
        .args(["add", "--batch", "-"])
        .write_stdin("---\ntitle: Dangler\nneeds: [@nosuch]\n---\n")
        .assert()
        .failure()
        .stderr(predicates::str::contains("@nosuch"));
    assert_eq!(before, crate::common::file_inventory(&repo), "nothing written");
}

#[test]
fn add_batch_rejects_explicit_ids_and_duplicate_handles() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);

    meshwork(&repo)
        .args(["add", "--batch", "-"])
        .write_stdin("---\nid: zz-forged1\ntitle: Forged\n---\n")
        .assert()
        .failure()
        .stderr(predicates::str::contains("minted"));

    meshwork(&repo)
        .args(["add", "--batch", "-"])
        .write_stdin("---\nhandle: dup\ntitle: One\n---\n---\nhandle: dup\ntitle: Two\n---\n")
        .assert()
        .failure()
        .stderr(predicates::str::contains("dup"));
}

/// mw-0wvndqa: §6 — `--dry-run` prints the would-be files, writes nothing.
/// Pilot evidence (sazed): bare `add --dry-run` wrote real 247-byte files
/// and printed only id+path. Covers bare add (text + json) and batch --json
/// (one clean envelope, never a text dump with JSON appended — MW-C3).
#[test]
fn dry_run_writes_nothing() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let before = crate::common::file_inventory(&repo);

    // Bare add prints the would-be file itself, not id+path.
    let out = stdout_of(
        &meshwork(&repo)
            .args(["add", "Spec probe", "--cat", "core/x", "--verify", "true", "--dry-run"])
            .assert()
            .success(),
    );
    assert!(out.contains("--- docs/meshwork/"), "{out}");
    assert!(out.contains("title: Spec probe"), "{out}");
    assert!(out.contains("status: open"), "{out}");

    // Bare add --json: one envelope carrying the content.
    let out = stdout_of(
        &meshwork(&repo)
            .args(["add", "Spec probe", "--verify", "true", "--dry-run", "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value =
        serde_json::from_str(&out).unwrap_or_else(|e| panic!("bad json ({e}): {out}"));
    assert_eq!(v["data"]["dry_run"], true, "{out}");
    let content = v["data"]["content"].as_str().unwrap_or_default();
    assert!(content.contains("title: Spec probe"), "{out}");

    // Batch --json: same contract — an envelope, content per task.
    let out = stdout_of(
        &meshwork(&repo)
            .args(["add", "--batch", "-", "--dry-run", "--json"])
            .write_stdin(BATCH)
            .assert()
            .success(),
    );
    let v: serde_json::Value =
        serde_json::from_str(&out).unwrap_or_else(|e| panic!("bad json ({e}): {out}"));
    assert_eq!(v["data"]["dry_run"], true, "{out}");
    let tasks = v["data"]["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 3, "{out}");
    let content = tasks[1]["content"].as_str().unwrap_or_default();
    assert!(content.contains("title: Define the verify grammar"), "{out}");

    assert_eq!(before, crate::common::file_inventory(&repo), "dry-run wrote files");
}

#[test]
fn add_batch_reads_from_file_too() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let batch_path = repo.join("batch.md");
    std::fs::write(&batch_path, "---\ntitle: From a file\nverify: \"true\"\n---\n").unwrap();

    let out = stdout_of(
        &meshwork(&repo)
            .args(["add", "--batch", "batch.md"])
            .assert()
            .success(),
    );
    let id = out.lines().next().unwrap();
    let text = std::fs::read_to_string(task_file(&repo, id)).unwrap();
    assert!(text.contains("title: From a file"), "{text}");
}
