// e2e part-file: comment + attach (PLAN 1.4; MW-K1/K2/K3). Included by e2e.rs.

/// MW-K1/K2: comments append with the author fallback chain; attachments
/// copy into attachments/<id>/ and land in frontmatter; --force gates
/// overwrite; oversized attachments draw the lint warning.
#[test]
fn comment_attach() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "Task with notes");
    let today = meshwork::clock::today();

    // --as wins the chain; identity is recorded as claimed.
    meshwork(&repo)
        .args(["comment", &id, "--as", "maya", "hand-checked the numbers"])
        .assert()
        .success();
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    // Minted stamps are minute-resolution (mw-zp1h12d) — assert the civil
    // date prefix and the payload separately.
    assert!(text.contains(&format!("- {today}")), "{text}");
    assert!(text.contains("[maya] hand-checked the numbers"), "{text}");

    // $MESHWORK_AUTHOR is next.
    meshwork(&repo)
        .env("MESHWORK_AUTHOR", "claude/f10a7561")
        .args(["comment", &id, "bisected while maya slept"])
        .assert()
        .success();
    // config default_author ("Fixture User", seeded from git) is last.
    meshwork(&repo)
        .args(["comment", &id, "falls back to config"])
        .assert()
        .success();
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(text.contains("[claude/f10a7561]"), "{text}");
    assert!(text.contains("[Fixture User]"), "{text}");

    // Multi-line text becomes two-space continuations; parse re-joins it.
    meshwork(&repo)
        .args(["comment", &id, "--as", "maya", "first line\nsecond line detail"])
        .assert()
        .success();
    let shown = stdout_of(&meshwork(&repo).args(["show", &id, "--comments"]).assert().success());
    assert!(shown.contains("second line detail"), "{shown}");
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(text.contains("\n  second line detail"), "continuation: {text}");

    // No author anywhere → loud error naming the chain.
    let mut config = std::fs::read_to_string(repo.join("docs/meshwork/config.toml")).unwrap();
    config = config
        .lines()
        .filter(|l| !l.starts_with("default_author"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(repo.join("docs/meshwork/config.toml"), config).unwrap();
    meshwork(&repo)
        .args(["comment", &id, "authorless"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("--as"));

    // attach: copy in, record in frontmatter, refuse silent overwrite.
    let src_dir = tempfile::tempdir().unwrap();
    let log = src_dir.path().join("p99-excerpt.log");
    std::fs::write(&log, "cliff onset at sample 9481\n").unwrap();
    meshwork(&repo)
        .args(["attach", &id, log.to_str().unwrap()])
        .assert()
        .success();
    let rel = format!("attachments/{id}/p99-excerpt.log");
    assert!(repo.join("docs").join("meshwork").join(&rel).is_file());
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(text.contains(&format!("attachments: [{rel}]")), "{text}");

    meshwork(&repo)
        .args(["attach", &id, log.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicates::str::contains("--force"));
    meshwork(&repo)
        .args(["attach", &id, log.to_str().unwrap(), "--force"])
        .assert()
        .success();

    // A second file grows the recorded list.
    let big = src_dir.path().join("full-profile.log");
    std::fs::write(&big, vec![b'x'; 1_200_000]).unwrap();
    meshwork(&repo)
        .args(["attach", &id, big.to_str().unwrap()])
        .assert()
        .success();
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(
        text.contains(&format!("attachments: [{rel}, attachments/{id}/full-profile.log")),
        "{text}"
    );

    // MW-K3: the oversized attachment draws the lint warning.
    let lint = stdout_of(&meshwork(&repo).arg("lint").assert().success());
    assert!(lint.contains("attachment-size"), "{lint}");

    // JSON envelopes for both verbs.
    let js = stdout_of(
        &meshwork(&repo)
            .args(["comment", &id, "--as", "maya", "enveloped", "--json"])
            .assert()
            .success(),
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&js).unwrap()["verb"],
        "comment"
    );
}
