//! Architecture guards (mw-5pq334y): the model layer stays CLI-free by
//! rule, not by accident — a future model-crate split (embedding a reader
//! in another binary or UI) must stay a Cargo.toml exercise, not surgery.

use std::path::Path;

/// Model modules: everything in src/ that is not the CLI shell or the
/// crate roots. None of them may mention the CLI layer or clap.
const MODEL_MODULES: &[&str] = &[
    "addressed.rs",
    "cache.rs",
    "clock.rs",
    "docs.rs",
    "edit.rs",
    "id.rs",
    "lint.rs",
    "lint_tail.rs",
    "lint_verify.rs",
    "parse.rs",
    "provenance.rs",
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

/// Internal-process citations that mean nothing to a user of the binary:
/// requirement IDs, task-id provenance, design-doc/plan/section references,
/// milestone tags. They belong in rustdoc and git history — NEVER in
/// anything the binary prints or writes for a person (mw-jhcjknt).
const CITATION_PATTERNS: &[&str] = &[
    r"MW-[A-Z]",       // requirement IDs (MW-C3, …)
    r"mw-[0-9a-z]{4}", // task-id citations (mw-0f4j, …)
    r"DESIGN",         // DESIGN-meshwork.md references
    r"REQUIREMENTS",   // REQUIREMENTS-meshwork.md references
    r"TRACE\.md",
    r"PLAN [0-9]", // plan-item references (PLAN 3.1)
    r"§\s?[0-9]",  // section citations (§12b) — `§-anchor` link syntax is fine
    r"\bM[1-9]\b", // milestone tags (M2, M3)
];

fn citation_in(text: &str) -> Option<&'static str> {
    CITATION_PATTERNS
        .iter()
        .find(|p| regex::Regex::new(p).unwrap().is_match(text))
        .copied()
}

/// Every string literal in src/ — help text, errors, warnings, log lines,
/// generated files — is user-facing until proven otherwise. Comments are
/// exempt (rustdoc on non-clap items never prints); string contents are not.
#[test]
fn printed_text_carries_no_internal_citations() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();
    scan_dir(&src, &mut offenders);
    assert!(
        offenders.is_empty(),
        "internal citations in string literals — users don't know these; \
         say what matters to THEM instead:\n{}",
        offenders.join("\n")
    );
}

fn scan_dir(dir: &Path, offenders: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            scan_dir(&entry.path(), offenders);
        } else if entry.path().extension().is_some_and(|e| e == "rs") {
            let text = std::fs::read_to_string(entry.path()).unwrap();
            for lit in string_literals(&text) {
                if let Some(pattern) = citation_in(&lit) {
                    offenders.push(format!(
                        "{}: `{}` matches {pattern}",
                        entry.path().display(),
                        lit.replace('\n', "\\n")
                    ));
                }
            }
        }
    }
}

/// Extract string-literal contents from Rust source, skipping comments,
/// char literals, and lifetimes. Handles `\`-escapes, raw strings
/// (`r"…"`/`r#"…"#`), and nested block comments.
fn string_literals(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '/' if chars.get(i + 1) == Some(&'/') => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            '/' if chars.get(i + 1) == Some(&'*') => {
                let mut depth = 1;
                i += 2;
                while i < chars.len() && depth > 0 {
                    if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                        depth += 1;
                        i += 2;
                    } else if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                        depth -= 1;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
            }
            'r' if matches!(chars.get(i + 1), Some(&'"' | &'#')) => {
                let mut hashes = 0;
                let mut j = i + 1;
                while chars.get(j) == Some(&'#') {
                    hashes += 1;
                    j += 1;
                }
                if chars.get(j) == Some(&'"') {
                    j += 1;
                    let start = j;
                    'raw: while j < chars.len() {
                        if chars[j] == '"' && chars[j + 1..].iter().take(hashes).all(|c| *c == '#')
                        {
                            out.push(chars[start..j].iter().collect());
                            j += 1 + hashes;
                            break 'raw;
                        }
                        j += 1;
                    }
                    i = j;
                } else {
                    i += 1;
                }
            }
            '"' => {
                let start = i + 1;
                i += 1;
                while i < chars.len() {
                    if chars[i] == '\\' {
                        i += 2;
                    } else if chars[i] == '"' {
                        break;
                    } else {
                        i += 1;
                    }
                }
                out.push(chars[start..i.min(chars.len())].iter().collect());
                i += 1;
            }
            '\'' => {
                // Char literal (`'x'`, `'\n'`) vs lifetime (`'a`): a
                // backslash or a closing quote two ahead means literal.
                if chars.get(i + 1) == Some(&'\\') {
                    i += 2;
                    while i < chars.len() && chars[i] != '\'' {
                        i += 1;
                    }
                    i += 1;
                } else if chars.get(i + 2) == Some(&'\'') {
                    i += 3;
                } else {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }
    out
}

/// Every help screen, recursively: what `--help` renders (clap doc-comments
/// and `about` strings) is the product's front door — no internal citations.
#[test]
fn help_carries_no_internal_citations() {
    fn walk(args: &[String], offenders: &mut Vec<String>) {
        let mut cmd = assert_cmd::Command::cargo_bin("meshwork").unwrap();
        let out = cmd.args(args).arg("--help").assert().success();
        let help = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        if let Some(pattern) = citation_in(&help) {
            offenders.push(format!(
                "`meshwork {} --help` matches {pattern}",
                args.join(" ")
            ));
        }
        let verbs: Vec<String> = help
            .lines()
            .skip_while(|l| !l.starts_with("Commands:"))
            .skip(1)
            .take_while(|l| l.starts_with("  "))
            .filter_map(|l| l.split_whitespace().next().map(ToString::to_string))
            .filter(|v| v != "help")
            .collect();
        for verb in verbs {
            let mut next = args.to_vec();
            next.push(verb);
            walk(&next, offenders);
        }
    }
    let mut offenders = Vec::new();
    walk(&[], &mut offenders);
    assert!(
        offenders.is_empty(),
        "internal citations in help output:\n{}",
        offenders.join("\n")
    );
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
