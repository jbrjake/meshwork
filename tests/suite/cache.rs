//! `cache::` — key derivation for the reserved `.cache/tasks.jsonl`
//! projection (mw-n0r5jwm, FORMAT.md Projection). Decision tests only:
//! nothing builds the cache yet; these pin the key's shape before it
//! lands.

use meshwork::cache::projection_key;
use std::path::Path;

fn write_store(root: &Path, files: &[(&str, &str)]) {
    let mw = root.join("docs/meshwork");
    std::fs::create_dir_all(mw.join("archive")).unwrap();
    std::fs::write(mw.join("config.toml"), "alias = \"zz\"\n").unwrap();
    for (name, body) in files {
        std::fs::write(mw.join(name), body).unwrap();
    }
}

/// mw-n0r5jwm: `git checkout` rewrites mtimes on every branch switch, so
/// an mtime-keyed cache thrashes in exactly the worktree-heavy workflow
/// meshwork targets. The key is a pure function of store content: same
/// bytes ⇒ same key across directories, inodes, and clock perturbation;
/// any content or path change ⇒ a different key.
#[test]
fn checkout_does_not_invalidate() {
    let files: &[(&str, &str)] = &[
        (
            "zz-a1b2-first.md",
            "---\nid: zz-a1b2\ntitle: First\nstatus: open\nverify: \"true\"\n---\n",
        ),
        (
            "archive/zz-c3d4-done.md",
            "---\nid: zz-c3d4\ntitle: Done\nstatus: done\nverify: \"true\"\n---\n",
        ),
    ];
    let dir = tempfile::tempdir().unwrap();

    // The checkout simulation: the same content in a different directory,
    // written at a different moment (fresh inodes, fresh mtimes).
    let a = dir.path().join("clone-a");
    let b = dir.path().join("clone-b");
    write_store(&a, files);
    write_store(&b, files);
    let key_a = projection_key(&a).unwrap();
    let key_b = projection_key(&b).unwrap();
    assert_eq!(key_a, key_b, "same content, same key — location is noise");

    // A bare mtime touch — what checkout does to every file — is noise too.
    let touched = a.join("docs/meshwork/zz-a1b2-first.md");
    let f = std::fs::File::options().write(true).open(&touched).unwrap();
    f.set_modified(std::time::SystemTime::UNIX_EPOCH).unwrap();
    drop(f);
    assert_eq!(projection_key(&a).unwrap(), key_a, "mtime is never the key");

    // One byte of content: a different projection, so a different key.
    let body = std::fs::read_to_string(&touched).unwrap();
    std::fs::write(&touched, body.replace("open", "doing")).unwrap();
    let flipped = projection_key(&a).unwrap();
    assert_ne!(flipped, key_a, "content change invalidates");

    // A rename with identical bytes: paths are projection input (the
    // `path` column, the filename-prefix idiom), so the key moves.
    std::fs::rename(
        b.join("docs/meshwork/zz-a1b2-first.md"),
        b.join("docs/meshwork/zz-a1b2-renamed.md"),
    )
    .unwrap();
    assert_ne!(
        projection_key(&b).unwrap(),
        key_a,
        "path change invalidates"
    );
}
