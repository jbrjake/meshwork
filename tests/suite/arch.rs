//! Architecture guards (mw-5pq334y): the model layer stays CLI-free by
//! rule, not by accident — a future model-crate split (embedding a reader
//! in another binary or UI) must stay a Cargo.toml exercise, not surgery.

use std::path::Path;

/// Model modules: everything in src/ that is not the CLI shell or the
/// crate roots. None of them may mention the CLI layer or clap.
const MODEL_MODULES: &[&str] = &[
    "clock.rs",
    "docs.rs",
    "edit.rs",
    "id.rs",
    "lint.rs",
    "parse.rs",
    "registry.rs",
    "registry_hygiene.rs",
    "store.rs",
    "tables.rs",
    "trust.rs",
    "verify_dsl.rs",
    "verify_exec.rs",
    "write.rs",
];

#[test]
fn model_boundary_holds() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for file in MODEL_MODULES {
        let text = std::fs::read_to_string(src.join(file)).unwrap();
        for needle in ["crate::cli", "clap"] {
            assert!(
                !text.contains(needle),
                "{file} mentions `{needle}` — model modules never import the \
                 CLI layer (mw-5pq334y); move the CLI-facing part to src/cli/"
            );
        }
    }
}

/// The guard list itself can't rot: every non-CLI src file is either listed
/// or one of the known roots, so a new model module is guarded by default.
#[test]
fn model_boundary_list_is_complete() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let roots = ["lib.rs", "main.rs"];
    for entry in std::fs::read_dir(&src).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            continue; // src/cli/ — the one place clap belongs
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        assert!(
            MODEL_MODULES.contains(&name.as_str()) || roots.contains(&name.as_str()),
            "src/{name} is neither a known root nor in MODEL_MODULES — add it \
             to the guard (mw-5pq334y)"
        );
    }
}
