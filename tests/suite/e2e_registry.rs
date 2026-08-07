// mw-mrjccx2: registry durability. Cross-repo refs bake `repo#id` into
// OTHER repos' files and every task ID bakes the store's alias prefix
// forever — so repos.toml gains per-repo `aliases = ["oldname"]` (inbound
// refs survive a rename; resolution accepts old names, lint warns and
// suggests the rewrite) and an ID-alias-prefix collision across registered
// repos is a lint error (bare-ID lookup is ambiguous the moment it
// happens). Registry context comes from `MESHWORK_PORTFOLIO=<dir>` until
// M2 lands proper portfolio discovery.

fn write_portfolio(dir: &Path, repos_toml: &str, local_toml: Option<&str>) -> std::path::PathBuf {
    let portfolio = dir.join("portfolio");
    std::fs::create_dir_all(&portfolio).unwrap();
    std::fs::write(portfolio.join("repos.toml"), repos_toml).unwrap();
    if let Some(local) = local_toml {
        std::fs::write(portfolio.join("repos.local.toml"), local).unwrap();
    }
    portfolio
}

/// A ref through a rename alias resolves and warns with the rewrite;
/// without registry context lint stays silent (cross-repo refs are the
/// registry's business, MW-B3/G5).
#[test]
fn registry_rename_alias() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    add_id(
        &repo,
        &[
            "add",
            "depends on renamed repo",
            "--verify",
            "true",
            "--needs",
            "oldleras#lr-x4x1",
        ],
    );

    // No registry context → today's behavior, no finding.
    let plain = stdout_of(&meshwork(&repo).arg("lint").assert().success());
    assert!(!plain.contains("renamed-repo"), "{plain}");

    let portfolio = write_portfolio(
        repo.parent().unwrap(),
        "[[repo]]\nname = \"leras\"\nremote = \"git@github.com:example/leras.git\"\naliases = [\"oldleras\"]\n\n\
         [[repo]]\nname = \"work\"\nremote = \"git@github.com:example/work.git\"\n",
        None,
    );
    let assert = meshwork(&repo)
        .env("MESHWORK_PORTFOLIO", &portfolio)
        .arg("lint")
        .assert()
        .success(); // a rename is a warning, never an error
    let out = stdout_of(&assert);
    assert!(out.contains("renamed-repo"), "{out}");
    assert!(
        out.contains("leras#lr-x4x1"),
        "suggests the rewrite: {out}"
    );
}

/// Two registered repos claiming the same ID alias prefix is an error —
/// and a rename alias colliding with another entry's name is one too.
#[test]
fn registry_rename_alias_collision() {
    let dir = tempfile::tempdir().unwrap();
    let mut stores = Vec::new();
    for name in ["mwtest-a", "mwtest-b"] {
        let repo = dir.path().join(name);
        std::fs::create_dir_all(&repo).unwrap();
        git(&repo, &["init", "-q"]);
        git(&repo, &["config", "user.name", "Fixture User"]);
        git(&repo, &["config", "user.email", "fixture@example.invalid"]);
        init_store(&repo);
        // Force both stores onto one ID alias prefix.
        let cfg_path = repo.join("docs/meshwork/config.toml");
        let cfg = std::fs::read_to_string(&cfg_path).unwrap();
        let cfg = cfg
            .lines()
            .map(|l| {
                if l.starts_with("alias = ") {
                    "alias = \"zz\"".to_string()
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&cfg_path, cfg).unwrap();
        stores.push(repo);
    }

    let portfolio = write_portfolio(
        dir.path(),
        "[[repo]]\nname = \"mwtest-a\"\nremote = \"x\"\n\n[[repo]]\nname = \"mwtest-b\"\nremote = \"x\"\n",
        Some(&format!(
            "[paths]\n\"mwtest-a\" = \"{}\"\n\"mwtest-b\" = \"{}\"\n",
            stores[0].display(),
            stores[1].display()
        )),
    );

    let assert = meshwork(&stores[0])
        .env("MESHWORK_PORTFOLIO", &portfolio)
        .arg("lint")
        .assert()
        .code(1);
    let out = stdout_of(&assert);
    assert!(out.contains("alias-collision"), "{out}");
    assert!(
        out.contains("mwtest-a") && out.contains("mwtest-b"),
        "names both claimants: {out}"
    );

    // Registry-internal namespace damage: an alias equal to another name.
    let portfolio2 = write_portfolio(
        &dir.path().join("p2"),
        "[[repo]]\nname = \"one\"\nremote = \"x\"\n\n\
         [[repo]]\nname = \"two\"\nremote = \"x\"\naliases = [\"one\"]\n",
        None,
    );
    let out = stdout_of(
        &meshwork(&stores[0])
            .env("MESHWORK_PORTFOLIO", &portfolio2)
            .arg("lint")
            .assert()
            .code(1),
    );
    assert!(out.contains("registry-collision"), "{out}");
}
