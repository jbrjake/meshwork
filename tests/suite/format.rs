//! `format::` — FORMAT.md contract checks that aren't parser or lint
//! specifics: value constraints the config table promises (mw-a6jdf5s).

use meshwork::lint::{lint_store, Severity};
use meshwork::store::load_repo;

/// mw-a6jdf5s: ID recovery from an invalid file takes the first two
/// dash-segments of the stem, so a dashed alias (`my-repo`) silently
/// corrupts recovery. The alias is `[a-z0-9]+` — lint errors on anything
/// else; `init` refuses to write one.
#[test]
fn alias_charset() {
    assert!(meshwork::id::valid_alias("mw"));
    assert!(meshwork::id::valid_alias("sazed42"));
    for bad in ["my-repo", "MW", "", "a_b", "a.b", "é"] {
        assert!(!meshwork::id::valid_alias(bad), "accepted `{bad}`");
    }

    let dir = tempfile::tempdir().unwrap();
    let mw = dir.path().join("repo/docs/meshwork");
    std::fs::create_dir_all(&mw).unwrap();
    std::fs::write(mw.join("config.toml"), "alias = \"my-repo\"\n").unwrap();
    let f = lint_store(&load_repo(&dir.path().join("repo")).unwrap());
    assert!(
        f.iter().any(|f| f.severity == Severity::Error
            && f.code == "alias-charset"
            && f.message.contains("my-repo")),
        "{f:?}"
    );
}
