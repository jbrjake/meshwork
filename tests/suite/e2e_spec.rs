// e2e part-file: `spec list` / `spec audit` and `portfolio spec`
// (mw-y1qwnz3, MW-T5) — coverage and drift over one document as a
// report, the five questions answered in one. Included by e2e.rs — tests
// here are `e2e::<name>`.

const AUDIT_SPEC: &str = "# Spec\n\n## Gate {#sp-gate}\n\nA task closes on exit 0.\n\n\
## Waive {#sp-waive}\n\nLoud and recorded.\n\n## Gone {#sp-gone}\n\nabout to vanish\n\n\
## Orphan {#sp-orphan}\n\nonly a dropped task pins me\n";

/// MW-T5: `spec list` names every clause with its hash and who covers
/// it; `spec audit` sorts the pins into unclaimed, orphaned, stale,
/// re-open candidates and dangling — text and JSON.
#[test]
fn spec_audit() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    std::fs::create_dir_all(repo.join("docs")).unwrap();
    std::fs::write(repo.join("docs/SPEC.md"), AUDIT_SPEC).unwrap();
    let cover = |id: &str, clause: &str| {
        meshwork(&repo)
            .args(["cover", id, &format!("docs/SPEC.md#{clause}")])
            .assert()
            .success();
    };
    let live = add_task(&repo, "Live on the gate");
    cover(&live, "sp-gate");
    let done = add_task(&repo, "Done on the gate");
    cover(&done, "sp-gate");
    meshwork(&repo).args(["close", &done]).assert().success();
    let gone = add_task(&repo, "Pinned to a clause that vanishes");
    cover(&gone, "sp-gone");
    let dropped = add_task(&repo, "Dropped on the orphan");
    cover(&dropped, "sp-orphan");
    meshwork(&repo).args(["drop", &dropped]).assert().success();

    // Before anything moves: everything covered reads as pinned.
    let list = stdout_of(&meshwork(&repo).args(["spec", "list", "docs/SPEC.md"]).assert().success());
    assert!(list.starts_with("docs/SPEC.md: 4 clauses\n"), "{list}");
    assert!(
        list.contains("sp-gate  Gate  @") && list.contains(&format!("covered by 2: work#{live} (open), work#{done} (done)")),
        "{list}"
    );
    assert!(list.contains("sp-waive  Waive  @") && list.contains("  unclaimed\n"), "{list}");
    let audit = stdout_of(&meshwork(&repo).args(["spec", "audit", "docs/SPEC.md"]).assert().success());
    assert!(audit.contains("docs/SPEC.md: 4 clauses, 4 pins across 4 tasks\n"), "{audit}");
    assert!(audit.contains("unclaimed (1): sp-waive\n"), "{audit}");
    assert!(audit.contains(&format!("orphaned (1): sp-orphan \u{2190} work#{dropped}\n")), "{audit}");
    assert!(audit.contains("stale (0): none\n") && audit.contains("re-open candidates (0): none\n"), "{audit}");
    assert!(audit.contains("dangling (0): none\n"), "{audit}");

    // The gate's words move and the gone clause vanishes.
    std::fs::write(
        repo.join("docs/SPEC.md"),
        AUDIT_SPEC
            .replace("exit 0", "exit 0, never a waive")
            .replace("## Gone {#sp-gone}\n\nabout to vanish\n\n", ""),
    )
    .unwrap();
    let audit = stdout_of(&meshwork(&repo).args(["spec", "audit", "docs/SPEC.md"]).assert().success());
    assert!(audit.contains(&format!("stale (1): work#{live} docs/SPEC.md#sp-gate pinned ")), "{audit}");
    assert!(audit.contains(&format!("re-open candidates (1): work#{done} docs/SPEC.md#sp-gate pinned ")), "{audit}");
    assert!(audit.contains(&format!("dangling (1): work#{gone} docs/SPEC.md#sp-gone\n")), "{audit}");
    let shown = stdout_of(&meshwork(&repo).args(["show", &done]).assert().success());
    assert!(shown.contains("[done]"), "a re-open candidate is surfaced, never reopened: {shown}");

    let v: serde_json::Value = serde_json::from_str(&stdout_of(
        &meshwork(&repo)
            .args(["spec", "audit", "docs/SPEC.md", "--json"])
            .assert()
            .success(),
    ))
    .unwrap();
    assert_eq!(v["verb"], "spec audit");
    assert_eq!(v["data"]["clauses"], 3);
    assert_eq!(v["data"]["unclaimed"], serde_json::json!(["sp-waive"]));
    assert_eq!(v["data"]["orphaned"][0]["id"], "sp-orphan");
    assert_eq!(v["data"]["stale"][0]["gid"], format!("work#{live}"));
    assert_ne!(v["data"]["stale"][0]["pinned"], v["data"]["stale"][0]["now"]);
    assert_eq!(v["data"]["reopen_candidates"][0]["status"], "done");
    assert_eq!(v["data"]["dangling"][0]["ref"], "docs/SPEC.md#sp-gone");
    let v: serde_json::Value = serde_json::from_str(&stdout_of(
        &meshwork(&repo)
            .args(["spec", "list", "docs/SPEC.md", "--json"])
            .assert()
            .success(),
    ))
    .unwrap();
    assert_eq!(v["verb"], "spec list");
    assert_eq!(v["data"]["clauses"][0]["covered_by"].as_array().map(Vec::len), Some(2), "{v}");

    // A document with no anchors, or none at all, is said plainly.
    std::fs::write(repo.join("docs/PLAIN.md"), "# Nothing anchored\n\ntext\n").unwrap();
    let out = stdout_of(&meshwork(&repo).args(["spec", "audit", "docs/PLAIN.md"]).assert().success());
    assert!(out.starts_with("docs/PLAIN.md: 0 clauses, 0 pins across 0 tasks\n"), "{out}");
    meshwork(&repo)
        .args(["spec", "list", "docs/NOPE.md"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("not readable"));
}

/// MW-T5: `portfolio spec audit <repo>#<path>` counts every store's pins
/// — a beta task pinned to alpha's document shows there and nowhere in
/// alpha's own audit; a bare path is refused over the union.
#[test]
fn portfolio_spec_audit() {
    let (dir, portfolio) = portfolio_fixture();
    let alpha = dir.path().join("alpha");
    let beta = dir.path().join("beta");
    std::fs::create_dir_all(alpha.join("docs")).unwrap();
    std::fs::write(alpha.join("docs/SPEC.md"), "## Gate {#sp-gate}\n\nshared clause\n").unwrap();
    let pinner = add_task(&beta, "Beta implements alpha's gate");
    meshwork(&beta)
        .env("MESHWORK_PORTFOLIO", &portfolio)
        .args(["cover", &pinner, "alpha#docs/SPEC.md#sp-gate"])
        .assert()
        .success();

    let local = stdout_of(&meshwork(&alpha).args(["spec", "audit", "docs/SPEC.md"]).assert().success());
    assert!(local.contains("unclaimed (1): sp-gate"), "alpha alone sees no pin: {local}");

    let assert = meshwork(dir.path())
        .env("MESHWORK_PORTFOLIO", &portfolio)
        .args(["portfolio", "spec", "audit", "alpha#docs/SPEC.md"])
        .assert()
        .success();
    let union = stdout_of(&assert);
    assert!(union.contains("alpha#docs/SPEC.md: 1 clauses, 1 pins across 1 tasks\n"), "{union}");
    assert!(union.contains("unclaimed (0): none"), "{union}");
    assert!(stderr_of(&assert).contains("skipped gamma"));
    let list = stdout_of(
        &meshwork(dir.path())
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .args(["portfolio", "spec", "list", "alpha#docs/SPEC.md"])
            .assert()
            .success(),
    );
    assert!(list.contains(&format!("covered by 1: beta#{pinner} (open)")), "{list}");
    meshwork(dir.path())
        .env("MESHWORK_PORTFOLIO", &portfolio)
        .args(["portfolio", "spec", "audit", "docs/SPEC.md"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("<repo>#<path>"));
}

/// MW-T10 (mw-s0ptwn0): `prime`'s weather opens with how many live tasks
/// the spec moved under, ids named — nothing at zero, a done task not
/// counted, gone once the pins are re-read.
#[test]
fn prime_spec_drift_line() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    std::fs::create_dir_all(repo.join("docs")).unwrap();
    std::fs::write(repo.join("docs/SPEC.md"), AUDIT_SPEC).unwrap();
    let ids: Vec<String> = (1..=4).map(|i| add_task(&repo, &format!("Pinned {i}"))).collect();
    for id in &ids {
        meshwork(&repo).args(["cover", id, "docs/SPEC.md#sp-gate"]).assert().success();
    }
    meshwork(&repo).args(["close", &ids[3]]).assert().success();
    let out = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    assert!(!out.contains("spec moved"), "nothing at zero:\n{out}");

    std::fs::write(repo.join("docs/SPEC.md"), AUDIT_SPEC.replace("exit 0", "exit 0 only")).unwrap();
    let out = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    let line = out.lines().find(|l| l.contains("spec moved")).expect(&out);
    assert!(line.starts_with("- spec moved under 3 live tasks ("), "{line}");
    for id in &ids[..3] {
        assert!(line.contains(id.as_str()), "{id} named: {line}");
    }
    assert!(!line.contains(ids[3].as_str()), "the done task is not counted: {line}");
    let weather_at = out.find("weather:").unwrap();
    assert!(out.find("spec moved").unwrap() > weather_at, "{out}");
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&meshwork(&repo).args(["prime", "--json"]).assert().success())).unwrap();
    assert!(v["data"]["weather"].as_array().unwrap().iter().any(|l| l.as_str().unwrap().contains("spec moved under 3")), "{v}");

    for id in &ids[..3] {
        meshwork(&repo).args(["cover", id, "--repin"]).assert().success();
    }
    let out = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    assert!(!out.contains("spec moved"), "re-pinned:\n{out}");

    // Past three, the rest is a count.
    let more: Vec<String> = (5..=7).map(|i| add_task(&repo, &format!("Pinned {i}"))).collect();
    for id in &more {
        meshwork(&repo).args(["cover", id, "docs/SPEC.md#sp-waive"]).assert().success();
    }
    // The gate keeps the words its pins were re-read at; only the waive
    // clause moves now.
    let gate_as_repinned = AUDIT_SPEC.replace("exit 0", "exit 0 only");
    std::fs::write(repo.join("docs/SPEC.md"), gate_as_repinned.replace("Loud", "Louder")).unwrap();
    let out = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    let line = out.lines().find(|l| l.contains("spec moved")).expect(&out);
    assert!(line.starts_with("- spec moved under 3 live tasks (") && line.ends_with(')'), "{line}");
    let fourth = add_task(&repo, "Pinned 8");
    meshwork(&repo).args(["cover", &fourth, "docs/SPEC.md#sp-waive"]).assert().success();
    std::fs::write(repo.join("docs/SPEC.md"), gate_as_repinned.replace("Loud", "Loudest")).unwrap();
    let out = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    let line = out.lines().find(|l| l.contains("spec moved")).expect(&out);
    assert!(line.contains("under 4 live tasks (") && line.ends_with(", +1)"), "{line}");
}
