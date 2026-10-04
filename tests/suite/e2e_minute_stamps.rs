// mw-zp1h12d: minted log/comment/created stamps carry UTC minute resolution
// (2026-08-06T21:47Z) — a §15.8 minting rule, never validation: the parser
// accepts date-only forever and MESHWORK_TODAY overrides stay verbatim so
// golden tests hold still.

/// `YYYY-MM-DDTHH:MMZ` — 17 chars, digits in the digit slots.
fn is_minute_stamp(s: &str) -> bool {
    let b = s.as_bytes();
    s.len() == 17
        && b[4] == b'-'
        && b[7] == b'-'
        && b[10] == b'T'
        && b[13] == b':'
        && b[16] == b'Z'
        && s.chars()
            .enumerate()
            .all(|(i, c)| matches!(i, 4 | 7 | 10 | 13 | 16) || c.is_ascii_digit())
}

#[test]
fn minute_stamps_on_minted_lines() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "stamped");
    meshwork(&repo).args(["start", &id]).assert().success();
    meshwork(&repo)
        .args(["comment", &id, "--as", "maya", "noted"])
        .assert()
        .success();

    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    let created = text
        .lines()
        .find_map(|l| l.strip_prefix("created: "))
        .unwrap();
    assert!(is_minute_stamp(created), "created: `{created}`");

    let log_stamp = text
        .lines()
        .find(|l| l.contains("open→doing"))
        .and_then(|l| l.strip_prefix("- "))
        .and_then(|l| l.split_whitespace().next())
        .unwrap();
    assert!(is_minute_stamp(log_stamp), "log: `{log_stamp}`");

    let comment_stamp = text
        .lines()
        .find(|l| l.contains("[maya]"))
        .and_then(|l| l.strip_prefix("- "))
        .and_then(|l| l.split_whitespace().next())
        .unwrap();
    assert!(is_minute_stamp(comment_stamp), "comment: `{comment_stamp}`");
}

#[test]
fn minute_stamps_override_stays_verbatim() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let out = meshwork(&repo)
        .args(["add", "pinned", "--verify", "true"])
        .env("MESHWORK_TODAY", "2026-08-04")
        .assert()
        .success();
    let id = stdout_of(&out).lines().next().unwrap().to_string();
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(text.contains("created: 2026-08-04\n"), "{text}");
    assert!(text.contains("- 2026-08-04 created"), "{text}");
}

/// A malformed `MESHWORK_TODAY` is refused before anything is minted —
/// the same guard the derived views apply, so a typo never reaches a file.
#[test]
fn mint_refuses_malformed_today() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "stamped");
    let before = crate::common::file_inventory(&repo);
    let path = task_file(&repo, &id);
    let text = std::fs::read_to_string(&path).unwrap();
    for (args, bad) in [
        (vec!["add", "pinned", "--verify", "true"], "banana"),
        (vec!["start", id.as_str()], "2026-13-40"),
        (vec!["comment", id.as_str(), "--as", "maya", "noted"], "2026-08-04T25:61Z"),
    ] {
        let assert = meshwork(&repo)
            .args(&args)
            .env("MESHWORK_TODAY", bad)
            .assert()
            .code(1);
        let err = stderr_of(&assert);
        assert!(
            err.contains("MESHWORK_TODAY must be YYYY-MM-DD or YYYY-MM-DDTHH:MMZ"),
            "{args:?}: {err}"
        );
        assert!(err.contains(bad), "names the value: {err}");
        assert_eq!(
            before,
            crate::common::file_inventory(&repo),
            "{args:?} wrote a file"
        );
        assert_eq!(text, std::fs::read_to_string(&path).unwrap(), "{args:?}");
    }
    assert!(!text.contains("banana"), "{text}");
}

#[test]
fn minute_stamps_date_only_files_still_parse() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    std::fs::write(
        repo.join("docs/meshwork/zz-oldstyl-date-only.md"),
        "---\nid: zz-oldstyl\ntitle: Old style\nstatus: open\nverify: \"true\"\ncreated: 2026-08-01\n---\n\n## log\n- 2026-08-01 created\n\n## comments\n- 2026-08-01 [jon] date-only forever\n",
    )
    .unwrap();
    let show = stdout_of(
        &meshwork(&repo)
            .args(["show", "zz-oldstyl"])
            .assert()
            .success(),
    );
    assert!(show.contains("created: 2026-08-01"), "{show}");
    assert!(show.contains("[jon] date-only forever"), "{show}");
}
