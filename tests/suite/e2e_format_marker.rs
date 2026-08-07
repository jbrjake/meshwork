// mw-n6nvzpa: store format version marker. init writes `format = 1`;
// absent means 1 (every pre-marker store); a format newer than this binary
// knows is refused loudly — semantic changes need detection, and that is
// unretrofittable archaeology once stores multiply.

#[test]
fn format_marker_written_on_init() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let cfg = std::fs::read_to_string(repo.join("docs/meshwork/config.toml")).unwrap();
    assert!(cfg.contains("format = 1"), "{cfg}");
}

#[test]
fn format_marker_absent_means_one() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let path = repo.join("docs/meshwork/config.toml");
    let kept: String = std::fs::read_to_string(&path)
        .unwrap()
        .lines()
        .filter(|l| !l.starts_with("format"))
        .fold(String::new(), |acc, l| acc + l + "\n");
    std::fs::write(&path, kept).unwrap();

    add_task(&repo, "pre-marker stores keep working");
    meshwork(&repo).arg("ready").assert().success();
}

#[test]
fn format_marker_newer_is_refused_loudly() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let path = repo.join("docs/meshwork/config.toml");
    let bumped = std::fs::read_to_string(&path)
        .unwrap()
        .replace("format = 1", "format = 2");
    std::fs::write(&path, bumped).unwrap();

    meshwork(&repo)
        .arg("ready")
        .assert()
        .failure()
        .stderr(predicates::str::contains("format 2"))
        .stderr(predicates::str::contains("upgrade"));
}
