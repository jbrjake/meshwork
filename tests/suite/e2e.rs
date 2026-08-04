//! E2E scenario tests — the real binary against real git in tempdirs
//! (DESIGN §13). Zero network anywhere (MW-J6).

use crate::common::git;
use assert_cmd::Command;
use std::path::Path;

/// Fresh tempdir with an initialized git repo inside, isolated from the
/// machine's git config.
fn git_repo(name: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join(name);
    std::fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "-q"]);
    git(&repo, &["config", "user.name", "Fixture User"]);
    git(&repo, &["config", "user.email", "fixture@example.invalid"]);
    (dir, repo)
}

fn meshwork(dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("meshwork").unwrap();
    cmd.current_dir(dir);
    cmd
}

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
        .stdout(predicates::str::contains("meshwork/config.toml"));

    let mw = repo.join("meshwork");
    let config = std::fs::read_to_string(mw.join("config.toml")).unwrap();
    assert!(config.contains("alias = \"wo\""), "config: {config}");
    assert!(
        config.contains("default_author = \"Fixture User\""),
        "seeded from git user.name: {config}"
    );
    assert_eq!(
        std::fs::read_to_string(mw.join(".gitattributes")).unwrap(),
        "tasks/*.md merge=union\n",
        "the committed union attr is MW-I1's whole mechanism"
    );
    assert_eq!(
        std::fs::read_to_string(mw.join(".cache/.gitignore")).unwrap(),
        "*\n!.gitignore\n"
    );
    assert!(mw.join("tasks").is_dir());
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
    assert!(repo.join("meshwork/config.toml").is_file());
    assert!(!sub.join("meshwork").exists());
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
    assert!(!dir.path().join("meshwork").exists());
}

/// Re-running init must not clobber an existing store.
#[test]
fn init_twice_refuses() {
    let (_g, repo) = git_repo("work");
    meshwork(&repo).arg("init").assert().success();
    std::fs::write(
        repo.join("meshwork/config.toml"),
        "alias = \"xx\"\n", // hand-edited; init must not overwrite
    )
    .unwrap();
    meshwork(&repo)
        .arg("init")
        .assert()
        .failure()
        .stderr(predicates::str::contains("already"));
    let config = std::fs::read_to_string(repo.join("meshwork/config.toml")).unwrap();
    assert!(config.contains("xx"), "hand-edited config untouched");
}

/// MW-C3: every command supports --json with the stable envelope.
#[test]
fn init_json_envelope() {
    let (_g, repo) = git_repo("work");
    let out = meshwork(&repo).args(["init", "--json"]).assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(v["v"], 1);
    assert_eq!(v["verb"], "init");
    assert!(v["data"]["created"].as_array().unwrap().len() >= 4);
}

fn init_store(repo: &Path) {
    meshwork(repo).arg("init").assert().success();
}

fn stdout_of(assert: &assert_cmd::assert::Assert) -> String {
    String::from_utf8(assert.get_output().stdout.clone()).unwrap()
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
        id.starts_with("wo-") && id.len() == 7,
        "add prints the id: {id}"
    );

    // File on disk, filename = <id>-<slug>.md, fields verbatim.
    let path = repo.join(format!("meshwork/tasks/{id}-fix-the-spill-cliff.md"));
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
    let path = repo.join("meshwork/tasks");
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
    let ctx = meshwork::tables::session_for(&[store]).unwrap();
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

fn task_file(repo: &Path, id: &str) -> std::path::PathBuf {
    std::fs::read_dir(repo.join("meshwork/tasks"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(&format!("{id}-"))
        })
        .unwrap_or_else(|| panic!("no file for {id}"))
}

fn add_task(repo: &Path, title: &str) -> String {
    let out = meshwork(repo)
        .args(["add", title, "--verify", "true"])
        .assert()
        .success();
    stdout_of(&out).lines().next().unwrap().to_string()
}

/// PLAN 0.6 / MW-E1: start/block/drop/reopen move status along the legal
/// lifecycle; block demands --reason; illegal moves leave the file
/// untouched; a status edit is a one-frontmatter-line diff (MW-I1).
#[test]
fn transitions() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "Lifecycle");
    let path = task_file(&repo, &id);

    // start: open → doing; exactly one line replaced + one log line added.
    let before = std::fs::read_to_string(&path).unwrap();
    meshwork(&repo).args(["start", &id]).assert().success();
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
    meshwork(&repo).args(["drop", &id]).assert().success();
    assert!(std::fs::read_to_string(&path)
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
    let date = meshwork::clock::today();
    assert!(log[1].starts_with(&format!("- {date} ")), "dated entries");
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
                "test -f meshwork/config.toml",
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
    let ctx = meshwork::tables::session_for(&[store]).unwrap();
    let rows = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(crate::common::sql_rows(
            &ctx,
            "SELECT id FROM tasks WHERE waived IS NOT NULL",
        ));
    assert_eq!(rows, [[id]]);
}

fn add_id(repo: &Path, args: &[&str]) -> String {
    let out = meshwork(repo).args(args).assert().success();
    stdout_of(&out).lines().next().unwrap().to_string()
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
