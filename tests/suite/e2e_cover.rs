// e2e part-file: `cover` (mw-psbn61z, MW-T1–T3) — a clause pin is a ref
// plus the hash of the clause's text as read now; the heading line is
// outside the text; a second cover refuses unless --repin; a hand-written
// entry is lint's covers-malformed and --repin is its fix. Included by
// e2e.rs — tests here are `e2e::<name>`.

const SPEC: &str = "# Spec\n\n## Close gate {#sp-close-gate}\n\nA task closes on exit 0.\n\n\
## Waive {#sp-waive}\n\nLoud and recorded.\n\n## Plain\n\nno anchor here\n";

fn pin_line(repo: &Path, id: &str) -> Option<String> {
    let text = std::fs::read_to_string(task_file(repo, id)).unwrap();
    text.lines()
        .skip_while(|l| !l.starts_with("covers:"))
        .find(|l| l.trim_start().starts_with("sha:"))
        .map(|l| l.trim().trim_start_matches("sha: ").to_string())
}

/// MW-T1/T3: `cover` resolves the ref, writes `{ref, sha}`, logs the pin
/// and projects a resolved `covers` row; the same ref refuses a second
/// time; `--repin` re-reads after the clause moved.
#[test]
fn cover_pins_hash() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    std::fs::create_dir_all(repo.join("docs")).unwrap();
    std::fs::write(repo.join("docs/SPEC.md"), SPEC).unwrap();
    let id = add_task(&repo, "Implement the close gate");

    let out = stdout_of(
        &meshwork(&repo)
            .args(["cover", &id, "docs/SPEC.md#sp-close-gate"])
            .assert()
            .success(),
    );
    let sha = pin_line(&repo, &id).expect("a sha under covers:");
    assert_eq!(sha.len(), 64, "{sha}");
    assert_eq!(out.trim(), format!("{id} covers docs/SPEC.md#sp-close-gate @{}", &sha[..12]));
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(
        text.contains(&format!("covers:\n  - ref: docs/SPEC.md#sp-close-gate\n    sha: {sha}\n")),
        "{text}"
    );
    assert!(text.contains(&format!(" cover docs/SPEC.md#sp-close-gate @{}", &sha[..12])), "{text}");

    let rows = stdout_of(
        &meshwork(&repo)
            .args(["q", "SELECT gid, ref, sha, resolved FROM covers"])
            .assert()
            .success(),
    );
    assert!(rows.contains(&format!("work#{id} | docs/SPEC.md#sp-close-gate | {sha} | true")), "{rows}");
    let shown = stdout_of(&meshwork(&repo).args(["show", &id]).assert().success());
    assert!(shown.contains(&format!("covers: docs/SPEC.md#sp-close-gate @{}", &sha[..12])), "{shown}");
    let v: serde_json::Value =
        serde_json::from_str(&stdout_of(&meshwork(&repo).args(["show", &id, "--json"]).assert().success())).unwrap();
    assert_eq!(v["data"]["covers"][0]["ref"], "docs/SPEC.md#sp-close-gate");
    assert_eq!(v["data"]["covers"][0]["sha"], sha);

    // The same ref, same text: nothing to do.
    let err = stderr_of(
        &meshwork(&repo)
            .args(["cover", &id, "docs/SPEC.md#sp-close-gate"])
            .assert()
            .failure(),
    );
    assert!(err.contains("already covers") && err.contains("nothing to do"), "{err}");

    // A retitled heading moves nothing.
    std::fs::write(repo.join("docs/SPEC.md"), SPEC.replace("## Close gate {", "## The close gate {")).unwrap();
    meshwork(&repo).args(["cover", &id, "--repin"]).assert().success();
    assert_eq!(pin_line(&repo, &id).unwrap(), sha, "heading text is outside the clause");

    // Changed words under the heading: a plain cover refuses and names
    // --repin, which re-pins to the new hash.
    std::fs::write(repo.join("docs/SPEC.md"), SPEC.replace("exit 0", "exit 0, never a waive")).unwrap();
    let err = stderr_of(
        &meshwork(&repo)
            .args(["cover", &id, "docs/SPEC.md#sp-close-gate"])
            .assert()
            .failure(),
    );
    assert!(err.contains("reads differently") && err.contains("--repin"), "{err}");
    meshwork(&repo)
        .args(["cover", &id, "docs/SPEC.md#sp-close-gate", "--repin"])
        .assert()
        .success();
    let moved = pin_line(&repo, &id).unwrap();
    assert_ne!(moved, sha);

    // A second pin appends; the table carries both.
    meshwork(&repo).args(["cover", &id, "docs/SPEC.md#sp-waive"]).assert().success();
    let rows = stdout_of(
        &meshwork(&repo)
            .args(["q", "SELECT count(*) FROM covers WHERE resolved"])
            .assert()
            .success(),
    );
    assert!(rows.contains('2'), "{rows}");

    // Not a clause ref, or a clause that is not there: refused, with the
    // anchored ids named.
    let err = stderr_of(&meshwork(&repo).args(["cover", &id, "docs/SPEC.md#§-close-gate"]).assert().failure());
    assert!(err.contains("sp-<slug>"), "{err}");
    let err = stderr_of(&meshwork(&repo).args(["cover", &id, "docs/SPEC.md#sp-nope"]).assert().failure());
    assert!(err.contains("sp-close-gate, sp-waive"), "{err}");
    let err = stderr_of(&meshwork(&repo).args(["cover", &id, "../etc/passwd#sp-x"]).assert().failure());
    assert!(err.contains("escapes"), "{err}");

    // A hand-written entry is an error until --repin mints the pin.
    let other = add_task(&repo, "Hand-pinned");
    let path = task_file(&repo, &other);
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, text.replace("\n---\n", "\ncovers:\n  - docs/SPEC.md#sp-waive\n---\n")).unwrap();
    let lint = meshwork(&repo).arg("lint").assert().failure();
    let lint_out = stdout_of(&lint);
    assert!(lint_out.contains("covers-malformed") && lint_out.contains(&other), "{lint_out}");
    let rows = stdout_of(
        &meshwork(&repo)
            .args(["q", &format!("SELECT sha, resolved FROM covers WHERE gid = 'work#{other}'")])
            .assert()
            .success(),
    );
    assert!(rows.contains(" | true"), "an unpinned ref still projects, NULL sha: {rows}");
    meshwork(&repo).args(["cover", &other, "--repin"]).assert().success();
    let lint_out = stdout_of(&meshwork(&repo).arg("lint").assert().success());
    assert!(!lint_out.contains("covers-malformed"), "{lint_out}");
}
