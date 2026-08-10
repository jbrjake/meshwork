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

// mw-5ckb (PLAN 2.1): full repos.local.toml override semantics, MW-G2.
// Values: absolute as-is; `~/` expands against HOME (loud when HOME can't
// resolve — an explicit override is never guessed); relative resolves
// against the portfolio dir. Keys share the name+alias namespace: former
// names apply (rename durability) but warn; an unknown key warns (the file
// is gitignored — a typo has no other review surface); two keys overriding
// one entry is an error. Absent local file is the normal state; a present
// but broken one is loud. Observability: two stores minting the same ID
// alias prefix collide only when BOTH paths resolve, so `alias-collision`
// is the probe that a path override actually landed.

/// A git store at an exact path whose config.toml alias is forced.
fn aliased_store(path: &Path, alias: &str) {
    std::fs::create_dir_all(path).unwrap();
    git(path, &["init", "-q"]);
    git(path, &["config", "user.name", "Fixture User"]);
    git(path, &["config", "user.email", "fixture@example.invalid"]);
    init_store(path);
    let cfg_path = path.join("docs/meshwork/config.toml");
    let cfg = std::fs::read_to_string(&cfg_path)
        .unwrap()
        .lines()
        .map(|l| {
            if l.starts_with("alias = ") {
                format!("alias = \"{alias}\"")
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&cfg_path, cfg).unwrap();
}

const TWO_REPOS: &str = "[[repo]]\nname = \"mwov-a\"\nremote = \"x\"\n\n\
                         [[repo]]\nname = \"mwov-b\"\nremote = \"x\"\n";

/// Default resolution finds `~/Documents/code/<name>`; a `[paths]` entry
/// redirects elsewhere. Without the override the second repo is absent
/// (skipped, MW-G5); with it, both resolve and the collision surfaces.
#[test]
fn registry_overrides_default_path_and_local_win() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    aliased_store(&home.join("Documents/code/mwov-a"), "zz");
    let store_b = dir.path().join("elsewhere/checkout-b");
    aliased_store(&store_b, "zz");

    let p_plain = write_portfolio(&dir.path().join("p1"), TWO_REPOS, None);
    meshwork(&home.join("Documents/code/mwov-a"))
        .env("MESHWORK_PORTFOLIO", &p_plain)
        .env("HOME", &home)
        .arg("lint")
        .assert()
        .success();

    let p_over = write_portfolio(
        &dir.path().join("p2"),
        TWO_REPOS,
        Some(&format!("[paths]\n\"mwov-b\" = \"{}\"\n", store_b.display())),
    );
    let out = stdout_of(
        &meshwork(&home.join("Documents/code/mwov-a"))
            .env("MESHWORK_PORTFOLIO", &p_over)
            .env("HOME", &home)
            .arg("lint")
            .assert()
            .code(1),
    );
    assert!(out.contains("alias-collision"), "{out}");
    assert!(out.contains("mwov-a") && out.contains("mwov-b"), "{out}");
}

/// `~/` expands against HOME; a relative path resolves against the
/// portfolio directory (the only deterministic anchor — cwd varies).
#[test]
fn registry_overrides_tilde_and_relative() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    let store_a = home.join("checkouts/aaa");
    aliased_store(&store_a, "zz");
    let parent = dir.path().join("p");
    aliased_store(&parent.join("bbb"), "zz");

    let portfolio = write_portfolio(
        &parent,
        TWO_REPOS,
        Some("[paths]\n\"mwov-a\" = \"~/checkouts/aaa\"\n\"mwov-b\" = \"../bbb\"\n"),
    );
    let out = stdout_of(
        &meshwork(&store_a)
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .env("HOME", &home)
            .arg("lint")
            .assert()
            .code(1),
    );
    assert!(out.contains("alias-collision"), "{out}");
}

/// A `[paths]` key naming no registered repo is a warning — the local file
/// is gitignored, so a typo there has no other review surface.
#[test]
fn registry_overrides_unknown_key_warns() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    let repo = dir.path().join("mwov-solo");
    aliased_store(&repo, "zz");
    let portfolio = write_portfolio(
        dir.path(),
        "[[repo]]\nname = \"mwov-solo\"\nremote = \"x\"\n",
        Some("[paths]\n\"mwov-sol\" = \"/nowhere/particular\"\n"),
    );
    let assert = meshwork(&repo)
        .env("MESHWORK_PORTFOLIO", &portfolio)
        .env("HOME", &home)
        .arg("lint")
        .assert()
        .success(); // a typo warns; it breaks nothing by itself
    let out = stdout_of(&assert);
    assert!(out.contains("unknown-path-override"), "{out}");
    assert!(out.contains("mwov-sol"), "{out}");
}

/// A key written against a former name still applies (refs survive a
/// rename; so do local overrides) but warns with the exact rewrite.
#[test]
fn registry_overrides_former_name_key() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    let store_a = dir.path().join("a");
    aliased_store(&store_a, "zz");
    let store_b = dir.path().join("b");
    aliased_store(&store_b, "zz");

    let portfolio = write_portfolio(
        dir.path(),
        "[[repo]]\nname = \"canon\"\nremote = \"x\"\naliases = [\"oldcanon\"]\n\n\
         [[repo]]\nname = \"beta\"\nremote = \"x\"\n",
        Some(&format!(
            "[paths]\n\"oldcanon\" = \"{}\"\n\"beta\" = \"{}\"\n",
            store_a.display(),
            store_b.display()
        )),
    );
    let out = stdout_of(
        &meshwork(&store_a)
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .env("HOME", &home)
            .arg("lint")
            .assert()
            .code(1), // the collision proves the oldcanon-keyed path applied
    );
    assert!(out.contains("alias-collision"), "{out}");
    assert!(out.contains("renamed-repo"), "{out}");
    assert!(
        out.contains("oldcanon") && out.contains("canon"),
        "names the former name and the rewrite: {out}"
    );
}

/// Two keys resolving to one entry (name + former name) make the override
/// ambiguous — an error, never a silent pick.
#[test]
fn registry_overrides_ambiguous_key_error() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    let repo = dir.path().join("mwov-solo");
    aliased_store(&repo, "zz");
    let portfolio = write_portfolio(
        dir.path(),
        "[[repo]]\nname = \"canon\"\nremote = \"x\"\naliases = [\"oldcanon\"]\n",
        Some("[paths]\n\"canon\" = \"/x\"\n\"oldcanon\" = \"/y\"\n"),
    );
    let out = stdout_of(
        &meshwork(&repo)
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .env("HOME", &home)
            .arg("lint")
            .assert()
            .code(1),
    );
    assert!(out.contains("override-collision"), "{out}");
    assert!(out.contains("canon") && out.contains("oldcanon"), "{out}");
}

/// A present-but-broken local file is loud (absent is the only silent
/// state), and `~` that cannot resolve is loud too — never guessed.
#[test]
fn registry_overrides_broken_local_loud() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    let repo = dir.path().join("mwov-solo");
    aliased_store(&repo, "zz");

    let broken = write_portfolio(
        &dir.path().join("p1"),
        "[[repo]]\nname = \"mwov-solo\"\nremote = \"x\"\n",
        Some("not toml at all [[["),
    );
    let assert = meshwork(&repo)
        .env("MESHWORK_PORTFOLIO", &broken)
        .env("HOME", &home)
        .arg("lint")
        .assert()
        .code(1);
    let err = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(err.contains("repos.local.toml"), "{err}");

    let tilde = write_portfolio(
        &dir.path().join("p2"),
        "[[repo]]\nname = \"mwov-solo\"\nremote = \"x\"\n",
        Some("[paths]\n\"mwov-solo\" = \"~/somewhere\"\n"),
    );
    let assert = meshwork(&repo)
        .env("MESHWORK_PORTFOLIO", &tilde)
        .env_remove("HOME")
        .arg("lint")
        .assert()
        .code(1);
    let err = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(err.contains("HOME"), "{err}");
}
