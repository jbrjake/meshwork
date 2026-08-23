/// With no args the binary prints usage and exits 2 — it never pretends.
#[test]
fn no_args_shows_usage_exit_2() {
    let (_g, repo) = git_repo("work");
    meshwork(&repo)
        .assert()
        .code(2)
        .stderr(predicates::str::contains("Usage"));
}

/// PLAN 0.4 / MW-A3, MW-I1: `init` writes the full layout at the git
/// toplevel — config, union merge attribute, cache gitignore — and
/// installs no hooks, touches nothing outside the repo.
#[test]
fn init_layout() {
    let (_g, repo) = git_repo("work");
    let hooks_before = std::fs::read_dir(repo.join(".git/hooks")).map_or(0, Iterator::count);

    meshwork(&repo)
        .arg("init")
        .assert()
        .success()
        .stdout(predicates::str::contains("docs/meshwork/config.toml"));

    let mw = repo.join("docs").join("meshwork");
    let config = std::fs::read_to_string(mw.join("config.toml")).unwrap();
    assert!(config.contains("alias = \"wo\""), "config: {config}");
    assert!(
        config.contains("default_author = \"Fixture User\""),
        "seeded from git user.name: {config}"
    );
    assert_eq!(
        std::fs::read_to_string(mw.join(".gitattributes")).unwrap(),
        "/*.md merge=union\n/archive/*.md merge=union\n",
        "the committed union attr is MW-I1's whole mechanism"
    );
    assert_eq!(
        std::fs::read_to_string(mw.join(".cache/.gitignore")).unwrap(),
        "*\n!.gitignore\n"
    );
    assert!(!mw.join("tasks").exists(), "flat store: no tasks/ level");
    assert!(mw.join("attachments").is_dir());

    // MW-A3: no hooks installed, no hooksPath redirection.
    let hooks_after = std::fs::read_dir(repo.join(".git/hooks")).map_or(0, Iterator::count);
    assert_eq!(hooks_before, hooks_after, "no git hooks installed");
    let out = std::process::Command::new("git")
        .args(["config", "core.hooksPath"])
        .current_dir(&repo)
        .output()
        .unwrap();
    assert!(!out.status.success(), "core.hooksPath must stay unset");
}

/// `init` from a subdirectory still writes at the repo root.
#[test]
fn init_from_subdir_writes_at_root() {
    let (_g, repo) = git_repo("work");
    let sub = repo.join("src/deep");
    std::fs::create_dir_all(&sub).unwrap();
    meshwork(&sub).arg("init").assert().success();
    assert!(repo.join("docs/meshwork/config.toml").is_file());
    assert!(!sub.join("docs").join("meshwork").exists());
}

/// MW-A3: refuses to write anywhere that isn't a git repo.
#[test]
fn init_refuses_outside_git_repo() {
    let dir = tempfile::tempdir().unwrap();
    meshwork(dir.path())
        .arg("init")
        .assert()
        .failure()
        .stderr(predicates::str::contains("git repo"));
    assert!(!dir.path().join("docs").join("meshwork").exists());
}

/// Re-running init must not clobber an existing store.
#[test]
fn init_twice_refuses() {
    let (_g, repo) = git_repo("work");
    meshwork(&repo).arg("init").assert().success();
    std::fs::write(
        repo.join("docs/meshwork/config.toml"),
        "alias = \"xx\"\n", // hand-edited; init must not overwrite
    )
    .unwrap();
    meshwork(&repo)
        .arg("init")
        .assert()
        .failure()
        .stderr(predicates::str::contains("already"));
    let config = std::fs::read_to_string(repo.join("docs/meshwork/config.toml")).unwrap();
    assert!(config.contains("xx"), "hand-edited config untouched");
}

/// MW-C3: every command supports --json with the stable envelope.
#[test]
fn init_json_envelope() {
    let (_g, repo) = git_repo("work");
    let out = meshwork(&repo).args(["init", "--json"]).assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(v["meshwork"]["schema"], 1);
    assert_eq!(v["meshwork"]["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(v["verb"], "init");
    assert!(v["data"]["created"].as_array().unwrap().len() >= 4);
}
/// PLAN 0.5 / MW-A1, E4, K4: `add` with every flag writes a well-formed
/// file and prints the id; hand-edited comments round-trip through `show`,
/// capped at last-3 with the `… and N more` marker.
#[test]
fn add_show_roundtrip() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let out = meshwork(&repo)
        .args([
            "add",
            "Fix the spill cliff",
            "--cat",
            "engine/spill",
            "--label",
            "perf",
            "--label",
            "p0",
            "--needs",
            "wo-aaaa",
            "--needs",
            "beta#bz-c0r3",
            "--parent",
            "wo-cccc",
            "--from",
            "wo-bbbb",
            "--verify",
            "cargo test spill::",
        ])
        .assert()
        .success();
    let id = stdout_of(&out).lines().next().unwrap().to_string();
    assert!(
        id.starts_with("wo-") && id.len() == 10,
        "add prints the id (7-char suffix, mw-1b09): {id}"
    );

    // File on disk, filename = <id>-<slug>.md, fields verbatim.
    let path = repo.join(format!("docs/meshwork/{id}-fix-the-spill-cliff.md"));
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains(&format!("id: {id}")), "{text}");
    assert!(text.contains("status: open"));
    assert!(text.contains("category: engine/spill"));
    assert!(text.contains("labels: [perf, p0]"));
    assert!(text.contains("needs: [wo-aaaa, beta#bz-c0r3]"));
    assert!(text.contains("parent: wo-cccc"));
    assert!(text.contains("discovered-from: wo-bbbb"));
    assert!(
        text.contains("verify: \"cargo test spill::\""),
        "trailing :: needs YAML quoting: {text}"
    );
    assert!(text.contains("## log"));

    // Hand-edit tolerance (MW-A1): append comments in an editor.
    let mut edited = text.clone();
    edited.push_str(
        "\n## comments\n- 2026-08-04 [jon] one\n- 2026-08-04 [maya] two\n- 2026-08-04 [jon] three\n- 2026-08-04 [claude/f10a7561] four\n",
    );
    std::fs::write(&path, edited).unwrap();

    // show: full task, last-3 comments + explicit more-marker (MW-K4/D2).
    let shown = stdout_of(&meshwork(&repo).args(["show", &id]).assert().success());
    assert!(shown.contains("Fix the spill cliff"));
    assert!(shown.contains("open"));
    assert!(shown.contains("engine/spill"));
    assert!(shown.contains("four") && shown.contains("three") && shown.contains("two"));
    assert!(!shown.contains("[jon] one"), "oldest comment capped away");
    assert!(shown.contains("… and 1 more"), "marker missing:\n{shown}");

    // --comments renders everything.
    let all = stdout_of(
        &meshwork(&repo)
            .args(["show", &id, "--comments"])
            .assert()
            .success(),
    );
    assert!(all.contains("[jon] one"));

    // JSON parity: total + shown, stable envelope.
    let js = stdout_of(
        &meshwork(&repo)
            .args(["show", &id, "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    assert_eq!(v["verb"], "show");
    assert_eq!(v["data"]["id"], serde_json::json!(id));
    assert_eq!(v["data"]["comments"]["total"], 4);
    assert_eq!(v["data"]["comments"]["shown"].as_array().unwrap().len(), 3);
}

/// mw-s3905fv (§6 ruling 2026-08-21): `add --body` lands the description
/// at creation — literal text, `@file`, or `-` (stdin) per the prose
/// spellings — placed above the tail sections, rendered by show. Every
/// substantive pilot task went add-then-append before this existed.
#[test]
fn add_body_lands_above_tail_sections() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_id(
        &repo,
        &[
            "add",
            "Described task",
            "--verify",
            "true",
            "--body",
            "Two lines of context.\nSecond line.",
        ],
    );
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    let body_at = text.find("Two lines of context.").unwrap();
    let log_at = text.find("## log").unwrap();
    assert!(body_at < log_at, "body above the tail sections: {text}");
    let shown = stdout_of(&meshwork(&repo).args(["show", &id]).assert().success());
    assert!(shown.contains("Two lines of context."), "{shown}");

    // `-` reads stdin; the payload never transits shell quoting.
    let out = meshwork(&repo)
        .args(["add", "Stdin body", "--verify", "true", "--body", "-"])
        .write_stdin("From stdin.\n")
        .assert()
        .success();
    let id2 = stdout_of(&out).lines().next().unwrap().to_string();
    let text = std::fs::read_to_string(task_file(&repo, &id2)).unwrap();
    assert!(text.contains("From stdin."), "{text}");

    // `@file` reads a file.
    std::fs::write(repo.join("body.md"), "From a file.\n").unwrap();
    let id3 = add_id(
        &repo,
        &["add", "File body", "--verify", "true", "--body", "@body.md"],
    );
    let text = std::fs::read_to_string(task_file(&repo, &id3)).unwrap();
    assert!(text.contains("From a file."), "{text}");

    // An empty payload writes no body at all — no stray blank scaffold.
    let id4 = add_id(&repo, &["add", "Bare", "--verify", "true", "--body", ""]);
    let text = std::fs::read_to_string(task_file(&repo, &id4)).unwrap();
    assert!(text.contains("---\n\n## log"), "no body block: {text}");
}

/// MW-D2/A5: caps with explicit `… and N more`, `--comments` opts out.
#[test]
fn show_caps() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let out = meshwork(&repo)
        .args(["add", "Capped", "--verify", "true"])
        .assert()
        .success();
    let id = stdout_of(&out).lines().next().unwrap().to_string();
    let path = repo.join("docs/meshwork");
    let file = std::fs::read_dir(&path)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.file_name().unwrap().to_string_lossy().starts_with(&id))
        .unwrap();
    let mut text = std::fs::read_to_string(&file).unwrap();
    text.push_str("\n## comments\n");
    for i in 1..=5 {
        use std::fmt::Write as _;
        let _ = writeln!(text, "- 2026-08-04 [jon] comment number {i}");
    }
    std::fs::write(&file, text).unwrap();

    let shown = stdout_of(&meshwork(&repo).args(["show", &id]).assert().success());
    assert!(shown.contains("comment number 5"));
    assert!(!shown.contains("comment number 2"));
    assert!(shown.contains("… and 2 more"), "{shown}");
}

/// MW-B3: parent never crosses repos — refused at creation, not just lint.
#[test]
fn add_refuses_crossrepo_parent() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    meshwork(&repo)
        .args(["add", "Bad parent", "--parent", "beta#bz-c0r3"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("parent"));
}

/// The `MESHWORK_ID_SEED` hook drives deterministic IDs (e2e scenario 2 relies
/// on forcing the same mint in two clones).
#[test]
fn add_seeded_id_deterministic() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let expected = meshwork::id::IdGen::with_seed(7).next_id("wo");
    let out = meshwork(&repo)
        .env("MESHWORK_ID_SEED", "7")
        .args(["add", "Seeded"])
        .assert()
        .success();
    assert_eq!(stdout_of(&out).lines().next().unwrap(), expected);
}

/// MW-E4: `--from` records discovered-from provenance end to end — file
/// field, show output, and the typed edge in the SQL tables.
#[test]
fn discovered_from_edge() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let origin = stdout_of(
        &meshwork(&repo)
            .args(["add", "Origin task"])
            .assert()
            .success(),
    )
    .lines()
    .next()
    .unwrap()
    .to_string();
    let found = stdout_of(
        &meshwork(&repo)
            .args(["add", "Found while working", "--from", &origin])
            .assert()
            .success(),
    )
    .lines()
    .next()
    .unwrap()
    .to_string();

    let shown = stdout_of(&meshwork(&repo).args(["show", &found]).assert().success());
    assert!(shown.contains(&format!("discovered-from: {origin}")));

    let store = meshwork::store::load_repo(&repo).unwrap();
    let ctx = meshwork::tables::session_for(&[store], &[]).unwrap();
    let rows = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(crate::common::sql_rows(
            &ctx,
            &format!(
                "SELECT dst_gid FROM edges WHERE kind='discovered-from' AND src_gid='work#{found}'"
            ),
        ));
    assert_eq!(rows, [[format!("work#{origin}")]]);
}



/// PLAN 0.6 / MW-E1: start/block/drop/reopen move status along the legal
/// lifecycle; block demands --reason; illegal moves leave the file
/// untouched; a status edit is a one-frontmatter-line diff (MW-I1).
/// Identity is stripped so `start` stays claimless here — the claimed
/// variant (a second one-line edit, mw-tb6gdr9) is `e2e_claim.rs`'s beat.
#[test]
fn transitions() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    strip_default_author(&repo);
    let id = add_task(&repo, "Lifecycle");
    let path = task_file(&repo, &id);

    // start: open → doing; exactly one line replaced + one log line added.
    let before = std::fs::read_to_string(&path).unwrap();
    meshwork(&repo)
        .args(["start", &id])
        .env_remove("MESHWORK_AUTHOR")
        .assert()
        .success();
    let after = std::fs::read_to_string(&path).unwrap();
    let b: Vec<&str> = before.lines().collect();
    let a: Vec<&str> = after.lines().collect();
    assert_eq!(a.len(), b.len() + 1, "one appended log line");
    let changed: Vec<usize> = b
        .iter()
        .zip(a.iter())
        .enumerate()
        .filter(|(_, (x, y))| x != y)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(changed.len(), 1, "exactly one line differs: {changed:?}");
    assert_eq!(a[changed[0]], "status: doing");

    // start again: illegal, file untouched.
    meshwork(&repo)
        .args(["start", &id])
        .assert()
        .failure()
        .stderr(predicates::str::contains("doing"));
    assert_eq!(after, std::fs::read_to_string(&path).unwrap());

    // block without --reason is a usage error (clap-required).
    meshwork(&repo).args(["block", &id]).assert().code(2);

    // block --reason: status + blocked-reason + log line.
    meshwork(&repo)
        .args(["block", &id, "--reason", "waiting on upstream fix"])
        .assert()
        .success();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("status: blocked"));
    assert!(text.contains("blocked-reason: waiting on upstream fix"));

    // reopen: blocked → open, reason cleared to the empty key.
    meshwork(&repo).args(["reopen", &id]).assert().success();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("status: open"));
    assert!(
        text.contains("blocked-reason:\n"),
        "cleared, key kept: {text}"
    );

    // drop; dropped is terminal for reopen (DESIGN §6: blocked|doing|done).
    // The file archives on drop (mw-45e2qf4) — re-locate it.
    meshwork(&repo).args(["drop", &id]).assert().success();
    assert!(std::fs::read_to_string(task_file(&repo, &id))
        .unwrap()
        .contains("status: dropped"));
    meshwork(&repo).args(["reopen", &id]).assert().failure();

    // JSON envelope on a transition.
    let id2 = add_task(&repo, "Json transition");
    let js = stdout_of(
        &meshwork(&repo)
            .args(["start", &id2, "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    assert_eq!(v["verb"], "start");
    assert_eq!(v["data"]["from"], "open");
    assert_eq!(v["data"]["to"], "doing");
}

/// MW-E3: every transition appends exactly one dated from→to log entry —
/// the durable handoff record.
#[test]
fn log_append_on_transitions() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "Logged");
    meshwork(&repo).args(["start", &id]).assert().success();
    meshwork(&repo)
        .args(["block", &id, "--reason", "repro needed"])
        .assert()
        .success();
    meshwork(&repo).args(["reopen", &id]).assert().success();

    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    let log: Vec<&str> = text
        .split("## log")
        .nth(1)
        .unwrap()
        .lines()
        .filter(|l| l.starts_with("- "))
        .collect();
    assert_eq!(log.len(), 4, "created + 3 transitions: {log:?}");
    assert!(log[1].contains("open→doing"));
    assert!(log[2].contains("doing→blocked — repro needed"));
    assert!(log[3].contains("blocked→open"));
    // Minute-resolution stamps (mw-zp1h12d) still lead with the civil date.
    let date = meshwork::clock::today();
    assert!(log[1].starts_with(&format!("- {date}")), "dated entries");
}

/// PLAN 0.7 / MW-E2: `close` runs verify: via `sh -c` from the repo root,
/// records exit + date in the log, and closes only on exit 0.
#[test]
fn close_gating() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);

    // Failing verify: not closed, attempt recorded.
    let failing = stdout_of(
        &meshwork(&repo)
            .args(["add", "Fails verify", "--verify", "exit 3"])
            .assert()
            .success(),
    )
    .lines()
    .next()
    .unwrap()
    .to_string();
    meshwork(&repo)
        .args(["close", &failing])
        .assert()
        .failure()
        .stderr(predicates::str::contains("exit 3"));
    let text = std::fs::read_to_string(task_file(&repo, &failing)).unwrap();
    assert!(text.contains("status: open"), "must not close: {text}");
    assert!(text.contains("verify exit 3"), "attempt recorded: {text}");

    // Passing verify, run from the repo root even when invoked in a subdir.
    let passing = stdout_of(
        &meshwork(&repo)
            .args([
                "add",
                "Passes verify",
                "--verify",
                "test -f docs/meshwork/config.toml",
            ])
            .assert()
            .success(),
    )
    .lines()
    .next()
    .unwrap()
    .to_string();
    let sub = repo.join("src");
    std::fs::create_dir_all(&sub).unwrap();
    meshwork(&sub).args(["close", &passing]).assert().success();
    let text = std::fs::read_to_string(task_file(&repo, &passing)).unwrap();
    assert!(text.contains("status: done"));
    assert!(text.contains("→done — verify exit 0"), "{text}");

    // Already done: refuse.
    meshwork(&repo).args(["close", &passing]).assert().failure();

    // No verify: close demands --waive (MW-E2).
    let unverified = add_id(&repo, &["add", "No verify yet"]);
    meshwork(&repo)
        .args(["close", &unverified])
        .assert()
        .failure()
        .stderr(predicates::str::contains("--waive"));
}

/// MW-E2: --waive closes without verify, recorded loud and queryable
/// (`WHERE waived IS NOT NULL`).
#[test]
fn close_waive_recorded() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_id(&repo, &["add", "Spike task"]);
    meshwork(&repo)
        .args([
            "close",
            &id,
            "--waive",
            "spike; deliverable is the follow-up verify",
        ])
        .assert()
        .success();
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(text.contains("status: done"));
    assert!(text.contains("waived: spike; deliverable is the follow-up verify"));
    assert!(text.contains("waived:"), "log too: {text}");

    let store = meshwork::store::load_repo(&repo).unwrap();
    let ctx = meshwork::tables::session_for(&[store], &[]).unwrap();
    let rows = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(crate::common::sql_rows(
            &ctx,
            "SELECT id FROM tasks WHERE waived IS NOT NULL",
        ));
    assert_eq!(rows, [[id]]);
}


/// mw-e8hg2kt: terminal transitions strip `handoff:` — the voice belongs
/// to whatever is up next, and a handoff surviving into archive/ is the
/// handoff-stale warning `--fix` can't repair, on a file nobody should
/// hand-edit. Non-terminal moves carry the block untouched; both close
/// paths (verified, waived) strip it.
#[test]
fn close_strips_handoff() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "Verified close");
    meshwork(&repo)
        .args(["set", &id, "--handoff", "resume at the spill cliff"])
        .assert()
        .success();

    // Non-terminal transitions carry the handoff along untouched.
    meshwork(&repo).args(["start", &id]).assert().success();
    meshwork(&repo)
        .args(["block", &id, "--reason", "waiting"])
        .assert()
        .success();
    meshwork(&repo).args(["reopen", &id]).assert().success();
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(text.contains("handoff: |"), "non-terminal keeps it: {text}");
    assert!(text.contains("resume at the spill cliff"));

    meshwork(&repo).args(["close", &id]).assert().success();
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(text.contains("status: done"));
    assert!(!text.contains("handoff:"), "close strips the key: {text}");
    assert!(
        !text.contains("resume at the spill cliff"),
        "…and the whole block under it: {text}"
    );

    // The waived path writes its own file — same strip.
    let waived = add_id(&repo, &["add", "Waived close"]);
    meshwork(&repo)
        .args(["set", &waived, "--handoff", "half-done; see the branch"])
        .assert()
        .success();
    meshwork(&repo)
        .args(["close", &waived, "--waive", "spike"])
        .assert()
        .success();
    let text = std::fs::read_to_string(task_file(&repo, &waived)).unwrap();
    assert!(!text.contains("handoff:"), "waive strips too: {text}");
}

/// mw-e8hg2kt: drop is the other terminal transition — same strip.
#[test]
fn drop_strips_handoff() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "Dropped with a note");
    meshwork(&repo)
        .args(["set", &id, "--handoff", "was chasing the wrong symptom"])
        .assert()
        .success();
    meshwork(&repo).args(["drop", &id]).assert().success();
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(text.contains("status: dropped"));
    assert!(!text.contains("handoff:"), "drop strips the block: {text}");
    assert!(!text.contains("wrong symptom"), "{text}");
}

/// mw-bd390q6 (§6 ruling 2026-08-22): `drop --reason` records the
/// rationale to the →dropped log entry, symmetric with block's — two
/// pilot sessions guessed the flag existed and lost the reason to chat
/// scrollback when it didn't. Optional: a bare drop stays legal (terminal
/// drops are sometimes self-evident).
#[test]
fn drop_reason_lands_in_log() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "Dropped for cause");
    let js = stdout_of(
        &meshwork(&repo)
            .args(["drop", &id, "--reason", "duplicate of wo-aaaa", "--json"])
            .assert()
            .success(),
    );
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(text.contains("status: dropped"));
    assert!(
        text.contains("open→dropped — duplicate of wo-aaaa"),
        "reason on the log entry, block-style: {text}"
    );
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    assert_eq!(v["data"]["reason"], "duplicate of wo-aaaa");

    // Bare drop keeps working — the flag is optional, unlike block's.
    let bare = add_task(&repo, "Dropped bare");
    meshwork(&repo).args(["drop", &bare]).assert().success();
    let text = std::fs::read_to_string(task_file(&repo, &bare)).unwrap();
    assert!(text.contains("status: dropped"));
    assert!(text.contains("open→dropped\n"), "no dangling dash: {text}");
}

/// Unknown ids fail loudly.
#[test]
fn show_unknown_id_fails() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    meshwork(&repo)
        .args(["show", "wo-zzzz"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("wo-zzzz"));
}

/// mw-acgp (README spec, owner-ruled): the store root is `docs/meshwork/`,
/// flat — config, attributes, cache, attachments, and task files all live
/// there directly; no `tasks/` level. `add` echoes the real path.
#[test]
fn store_at_docs_meshwork() {
    let (_g, repo) = git_repo("work");
    meshwork(&repo)
        .arg("init")
        .assert()
        .success()
        .stdout(predicates::str::contains("docs/meshwork/config.toml"));
    let store = repo.join("docs/meshwork");
    assert!(store.join("config.toml").exists());
    assert!(store.join(".cache/.gitignore").exists());
    assert!(store.join("attachments").exists());
    assert!(!repo.join("meshwork").exists(), "old root must not appear");
    assert!(!store.join("tasks").exists(), "flat: no tasks/ level");
    // union-merge attribute anchored to the store dir itself (flat *.md)
    let attrs = std::fs::read_to_string(store.join(".gitattributes")).unwrap();
    assert!(attrs.contains("/*.md merge=union"), "{attrs}");

    let out = meshwork(&repo)
        .args(["add", "Flat store", "--verify", "true"])
        .assert()
        .success();
    let stdout = stdout_of(&out);
    let id = stdout.lines().next().unwrap().to_string();
    let echoed = stdout.lines().nth(1).unwrap().trim().to_string();
    assert_eq!(echoed, format!("docs/meshwork/{id}-flat-store.md"));
    assert!(repo.join(&echoed).exists(), "file at echoed path");
}
