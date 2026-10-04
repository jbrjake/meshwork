//! Architecture guards (mw-5pq334y): the model layer stays CLI-free by
//! rule, not by accident — a future model-crate split (embedding a reader
//! in another binary or UI) must stay a Cargo.toml exercise, not surgery.

use std::path::Path;

/// Model modules: everything in src/ that is not the CLI shell or the
/// crate roots. None of them may mention the CLI layer or clap.
const MODEL_MODULES: &[&str] = &[
    "addressed.rs",
    "archive.rs",
    "cache.rs",
    "clock.rs",
    "docs.rs",
    "edit.rs",
    "facts.rs",
    "grammar.rs",
    "graph.rs",
    "id.rs",
    "lint.rs",
    "lint_covers.rs",
    "lint_prose.rs",
    "lint_shim.rs",
    "lint_tail.rs",
    "lint_verify.rs",
    "lint_views.rs",
    "parse.rs",
    "paths.rs",
    "provenance.rs",
    "pulse.rs",
    "registry.rs",
    "registry_hygiene.rs",
    "spec.rs",
    "store.rs",
    "tables.rs",
    "trust.rs",
    "verify_dsl.rs",
    "verify_exec.rs",
    "views.rs",
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

/// What `meshwork <args…> --help` prints.
fn help_of(args: &[String]) -> String {
    let mut cmd = assert_cmd::Command::cargo_bin("meshwork").unwrap();
    let out = cmd.args(args).arg("--help").assert().success();
    String::from_utf8(out.get_output().stdout.clone()).unwrap()
}

/// The verbs a help screen's `Commands:` block lists, minus clap's own
/// `help` and any verb the screen marks `not built yet` — those are not
/// shipped surface and the skill owes them nothing.
fn verbs_of(help: &str) -> Vec<String> {
    help.lines()
        .skip_while(|l| !l.starts_with("Commands:"))
        .skip(1)
        .take_while(|l| l.starts_with("  "))
        .filter(|l| !l.contains("not built yet"))
        .filter_map(|l| l.split_whitespace().next().map(ToString::to_string))
        .filter(|v| v != "help")
        .collect()
}

/// The skill file the plugin ships, read whole.
fn skill_md() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(".claude/skills/meshwork/SKILL.md");
    std::fs::read_to_string(&path).unwrap()
}

/// The skill is the agent's front door to the binary. Every verb, every
/// sub-verb, and every `add`/`set` flag the help screens list is named in
/// SKILL.md, so a surface that lands without its skill line fails here in
/// the same commit — the skill is part of the feature, never a trailing
/// documentation task (mw-nq6rew9).
#[test]
fn skill_names_every_verb_and_flag() {
    /// `needle` occurs and is not the prefix of a longer word.
    fn named(text: &str, needle: &str) -> bool {
        text.match_indices(needle).any(|(i, _)| {
            let after = text[i + needle.len()..].chars().next();
            !after.is_some_and(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        })
    }
    fn walk(args: &[String], skill: &str, missing: &mut Vec<String>) {
        let help = help_of(args);
        for verb in verbs_of(&help) {
            let mut path = args.to_vec();
            path.push(verb.clone());
            let needle = match path.len() {
                1 => format!("`{verb}"),
                n => path[n - 2..].join(" "),
            };
            if !named(skill, &needle) {
                missing.push(format!("verb `meshwork {}`", path.join(" ")));
            }
            walk(&path, skill, missing);
        }
        if args.len() == 1 && (args[0] == "add" || args[0] == "set") {
            let flags = help
                .lines()
                .skip_while(|l| !l.starts_with("Options:"))
                .filter_map(|l| l.split_whitespace().find(|w| w.starts_with("--")))
                .filter(|f| !matches!(*f, "--json" | "--help"));
            for flag in flags {
                if !named(skill, flag) {
                    missing.push(format!("flag `{} {flag}`", args[0]));
                }
            }
        }
    }

    let skill = skill_md();
    let mut missing = Vec::new();
    walk(&[], &skill, &mut missing);
    assert!(
        missing.is_empty(),
        "SKILL.md does not name these surfaces — teach them in the same \
         commit as the code:\n{}",
        missing.join("\n")
    );
}

/// The two spellings a grant takes per verb: Claude Code matches the command
/// text as typed, and agents type the committed shim both ways.
const SHIM_SPELLINGS: [&str; 2] = ["docs/meshwork/meshwork", "./docs/meshwork/meshwork"];

/// SKILL.md's frontmatter `allowed-tools:` list — the plugin's permission
/// grant. Frontmatter that is missing, does not parse, lacks the key, or
/// holds anything but a non-empty list of strings fails here by name, so
/// the grant test can never pass on "zero grants checked".
fn skill_grants() -> Vec<String> {
    let skill = skill_md();
    let yaml = skill
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---\n"))
        .map(|(front, _)| front)
        .expect("SKILL.md opens with a `---`-fenced frontmatter block");
    let front: serde_yaml_ng::Value =
        serde_yaml_ng::from_str(yaml).expect("SKILL.md's frontmatter parses as YAML");
    let grants = front
        .get("allowed-tools")
        .expect("SKILL.md's frontmatter carries an `allowed-tools:` key")
        .as_sequence()
        .expect("`allowed-tools:` is a list");
    let grants: Vec<String> = grants
        .iter()
        .map(|g| {
            g.as_str()
                .expect("every `allowed-tools` entry is a string")
                .to_string()
        })
        .collect();
    assert!(!grants.is_empty(), "`allowed-tools:` lists no grants");
    grants
}

/// The plugin's permission grant is part of the feature. Every top-level
/// verb `--help` lists is granted in SKILL.md's frontmatter in both shim
/// spellings, so a verb that lands taught but still stops the agent for
/// approval on every call fails here in the same commit (mw-gh067xt: verbs
/// shipped for months with no grant because only the prose was checked).
#[test]
fn skill_grants_every_verb() {
    let grants = skill_grants();
    let mut missing = Vec::new();
    for verb in verbs_of(&help_of(&[])) {
        for shim in SHIM_SPELLINGS {
            let want = format!("Bash({shim} {verb} *)");
            if !grants.contains(&want) {
                missing.push(format!("  - {want}"));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "SKILL.md's `allowed-tools:` does not grant these — a verb that \
         prompts for approval on every call is shipped broken; add each \
         line to the frontmatter in the same commit as the verb:\n{}",
        missing.join("\n")
    );
}

/// Every git-tracked path with its mode, as `git ls-files -s` reports them.
fn tracked_modes(root: &Path) -> std::collections::BTreeMap<String, String> {
    let out = std::process::Command::new("git")
        .args(["ls-files", "-s", "-z"])
        .current_dir(root)
        .output()
        .expect("git ls-files runs in the repo");
    assert!(out.status.success(), "git ls-files failed");
    String::from_utf8(out.stdout)
        .unwrap()
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(|entry| {
            let (meta, path) = entry.split_once('\t').expect("ls-files -s entry has a tab");
            let mode = meta.split_whitespace().next().unwrap().to_string();
            (path.to_string(), mode)
        })
        .collect()
}

/// The plugin's hook is the upgrade, and the manifest is its only entry
/// point: a hook `command` or argument naming a file under
/// `${CLAUDE_PLUGIN_ROOT}` must be a git-tracked executable the plugin
/// ships, or every adopter gets a dead hook — no upgrade, no prime — while
/// the gate stays green (mw-s7399xj). `claude plugin validate` checks
/// component paths and never a hook's command. A manifest registering no
/// hook fails too: the upgrade would have no entry point at all.
#[test]
fn manifest_hook_paths_ship() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join(".claude-plugin/plugin.json")).unwrap(),
    )
    .expect("plugin.json parses");
    let hooks = manifest
        .get("hooks")
        .and_then(serde_json::Value::as_object)
        .expect("plugin.json carries a `hooks` object");
    let mut paths = Vec::new();
    for (event, matchers) in hooks {
        let matchers = matchers
            .as_array()
            .unwrap_or_else(|| panic!("hooks.{event} is not a list of matchers"));
        for (m, matcher) in matchers.iter().enumerate() {
            let entries = matcher
                .get("hooks")
                .and_then(serde_json::Value::as_array)
                .unwrap_or_else(|| panic!("hooks.{event}[{m}] carries no `hooks` list"));
            for (h, hook) in entries.iter().enumerate() {
                let at = format!("hooks.{event}[{m}].hooks[{h}]");
                let command = hook
                    .get("command")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_else(|| panic!("{at} has no `command` string"));
                paths.push((format!("{at}.command"), command.to_string()));
                let args = hook
                    .get("args")
                    .and_then(serde_json::Value::as_array)
                    .map(Vec::as_slice)
                    .unwrap_or_default();
                for (a, arg) in args.iter().enumerate() {
                    if let Some(arg) = arg.as_str().filter(|s| s.contains("${CLAUDE_PLUGIN_ROOT}"))
                    {
                        paths.push((format!("{at}.args[{a}]"), arg.to_string()));
                    }
                }
            }
        }
    }
    assert!(
        !paths.is_empty(),
        "plugin.json registers no hook — the upgrade has no entry point"
    );

    let tracked = tracked_modes(root);
    let mut broken = Vec::new();
    for (at, raw) in paths {
        let Some(rel) = raw.strip_prefix("${CLAUDE_PLUGIN_ROOT}/") else {
            broken.push(format!(
                "  {at} = `{raw}` does not start with `${{CLAUDE_PLUGIN_ROOT}}/` — \
                 a plugin hook runs what the plugin ships, nothing outside it"
            ));
            continue;
        };
        if !root.join(rel).is_file() {
            broken.push(format!("  {at} = `{raw}` names no file in the tree"));
            continue;
        }
        match tracked.get(rel).map(String::as_str) {
            None => broken.push(format!("  {at} = `{raw}` is not git-tracked")),
            Some("100755") => {}
            Some(mode) => broken.push(format!(
                "  {at} = `{raw}` is tracked with mode {mode}, not executable (100755)"
            )),
        }
    }
    assert!(
        broken.is_empty(),
        "plugin.json's hook commands do not resolve to shipped executables — \
         every adopter would get a dead hook:\n{}",
        broken.join("\n")
    );
}

/// Top-level tracked entries a cold session never needs: git plumbing and
/// files derived from a named one. Everything else at the top level is an
/// artifact, a channel, a gate or a hook, and CLAUDE.md names it by path.
const UNNAMED_TOP_LEVEL: &[&str] = &[".gitignore", "Cargo.lock", "LICENSE"];

/// CLAUDE.md is the cold session's map of what this repo ships and how it
/// is gated, and it drifts because nothing ties it to the tree (mw-nhjns62:
/// the plugin was vended for months before CLAUDE.md said so). The set:
/// every top-level tracked entry except `UNNAMED_TOP_LEVEL`, the plugin
/// manifest, the skill directory, and every workflow file — the things a
/// new one of would need a line before it can be found. Narrower is the
/// bug being fixed; wider turns CLAUDE.md into `git ls-files`. Each
/// unnamed path fails by name. A name counts only as a whole path token,
/// so `src` inside `sources` is not `src/`.
#[test]
fn claude_md_names_every_shipped_artifact() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = std::process::Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(root)
        .output()
        .expect("git ls-files runs in the repo");
    assert!(out.status.success(), "git ls-files failed");
    let files: Vec<String> = out
        .stdout
        .split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8(s.to_vec()).unwrap())
        .collect();
    assert!(!files.is_empty(), "git ls-files listed nothing");
    let mut required: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for f in &files {
        let top = f.split('/').next().unwrap();
        if !UNNAMED_TOP_LEVEL.contains(&top) {
            required.insert(top.to_string());
        }
        let is_workflow = f.starts_with(".github/workflows/")
            && Path::new(f).extension().is_some_and(|ext| {
                ext.eq_ignore_ascii_case("yml") || ext.eq_ignore_ascii_case("yaml")
            });
        if is_workflow {
            required.insert(f.clone());
        }
    }
    required.insert(".claude-plugin/plugin.json".to_string());
    required.insert(".claude/skills/meshwork".to_string());

    let claude_md = std::fs::read_to_string(root.join("CLAUDE.md")).unwrap();
    let token_char = |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.');
    let named = |path: &str| {
        claude_md.match_indices(path).any(|(i, _)| {
            let before = claude_md[..i].chars().next_back();
            let after = claude_md[i + path.len()..].chars().next();
            !before.is_some_and(token_char) && !after.is_some_and(|c| token_char(c) && c != '.')
        })
    };
    let missing: Vec<&String> = required.iter().filter(|p| !named(p)).collect();
    assert!(
        missing.is_empty(),
        "CLAUDE.md does not name these shipped artifacts — a cold session \
         cannot find them; name each by path, or add it to UNNAMED_TOP_LEVEL \
         with the reason it is not one:\n{}",
        missing
            .iter()
            .map(|p| format!("  {p}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
