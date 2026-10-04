// e2e part-file: the plugin's SessionStart hook brings the session's
// project — pin, shim, cached binary — to the release of the plugin the
// session loaded (mw-x5yn4rg). include!d from e2e.rs so test paths stay
// flat (`e2e::plugin_upgrade_brings_adopter_current`). The hook is a shell
// script; these tests run it the way Claude Code does, with the stub `curl`
// on PATH replaying a release tarball — zero network (MW-J6).

/// The pre-2026-09 adopter shim, keyed on the bridge variable alone — the
/// text an adopter transcribed at adoption and nothing ever rewrote.
const STALE_SHIM: &str = r#"#!/bin/sh
# agent sessions get a session-tagged author; explicit --as still wins
if [ -z "$MESHWORK_AUTHOR" ] && [ -n "$CLAUDE_CODE_BRIDGE_SESSION_ID" ]; then
  export MESHWORK_AUTHOR="claude ($CLAUDE_CODE_BRIDGE_SESSION_ID)"
fi
exec ~/.meshwork/versions/"$(cat "$(dirname "$0")/../../.meshwork-version")"/meshwork "$@"
"#;

const OLD_TAG: &str = "v0.1.0";

/// The repo is the plugin: `${CLAUDE_PLUGIN_ROOT}` is what the marketplace
/// installs, and this checkout is that tree.
fn plugin_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// The release the plugin states — plugin.json's version, which smoke keeps
/// in lockstep with Cargo.toml's.
fn plugin_tag() -> String {
    format!("v{}", env!("CARGO_PKG_VERSION"))
}

/// The release asset the hook asks for on this machine — the same mapping
/// the hook makes from `uname -sm`.
fn release_asset(tag: &str) -> String {
    let target = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        other => panic!("no release target for {other:?}"),
    };
    format!("meshwork-{tag}-{target}.tar.gz")
}

fn canonical_shim() -> Vec<u8> {
    std::fs::read(plugin_root().join("hooks/meshwork")).unwrap()
}

fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

/// A pinned adopter behind the plugin: the alpha corpus under `home`,
/// pinned at an old release, carrying the stale shim.
fn adopter_behind(home: &Path) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let repo = home.join("alpha");
    copy_dir(&fixtures_root().join("alpha"), &repo);
    git(&repo, &["init", "-q"]);
    std::fs::write(repo.join(".meshwork-version"), format!("{OLD_TAG}\n")).unwrap();
    let shim = repo.join("docs/meshwork/meshwork");
    std::fs::write(&shim, STALE_SHIM).unwrap();
    std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755)).unwrap();
    repo
}

/// Stage what the release URL serves: a tarball whose `meshwork` is a
/// script over the real test binary, so the fetched "binary" runs prime
/// for real through the rewritten shim.
fn stage_release(home: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let stage = home.join("stage");
    std::fs::create_dir_all(&stage).unwrap();
    let fake = stage.join("meshwork");
    std::fs::write(
        &fake,
        format!("#!/bin/sh\nexec \"{}\" \"$@\"\n", env!("CARGO_BIN_EXE_meshwork")),
    )
    .unwrap();
    std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
    let canned = home.join("canned");
    std::fs::create_dir_all(&canned).unwrap();
    let tar = std::process::Command::new("tar")
        .arg("-czf")
        .arg(canned.join(release_asset(&plugin_tag())))
        .arg("-C")
        .arg(&stage)
        .arg("meshwork")
        .status()
        .unwrap();
    assert!(tar.success(), "tar stages the release tarball");
}

/// Stage a fetch that fails: the release URL answers 404.
fn stage_failed_release(home: &Path) {
    let canned = home.join("canned");
    std::fs::create_dir_all(&canned).unwrap();
    std::fs::write(
        canned.join(format!("{}.exit", release_asset(&plugin_tag()))),
        "22\n",
    )
    .unwrap();
}

/// Run the hook as Claude Code does: the script itself (so the executable
/// bit is proven), project as cwd, `CLAUDE_PROJECT_DIR` and
/// `CLAUDE_PLUGIN_ROOT` set, the event JSON on stdin (empty here), and
/// the stub `curl` first on PATH.
fn run_hook(home: &Path, project: &Path) -> std::process::Output {
    let path = format!(
        "{}:{}",
        plugin_root().join("tests/bin").display(),
        std::env::var("PATH").unwrap_or_default()
    );
    std::process::Command::new(plugin_root().join("hooks/session-start.sh"))
        .current_dir(project)
        .env("CLAUDE_PROJECT_DIR", project)
        .env("CLAUDE_PLUGIN_ROOT", plugin_root())
        .env("HOME", home)
        .env("PATH", path)
        .env("CURL_STUB_CALLS", home.join("curl.calls"))
        .env("CURL_STUB_CANNED", home.join("canned"))
        .env("MESHWORK_TRUST", "1")
        .env_remove("MESHWORK_PORTFOLIO")
        .env_remove("MESHWORK_AUTHOR")
        .stdin(std::process::Stdio::null())
        .output()
        .expect("the hook script runs")
}

fn stdout_text(out: &std::process::Output) -> String {
    String::from_utf8(out.stdout.clone()).unwrap()
}

fn fetch_calls(home: &Path) -> String {
    std::fs::read_to_string(home.join("curl.calls")).unwrap_or_default()
}

/// Owner ruling 2026-09-28: upgrading the plugin is the whole upgrade. An
/// adopter behind the plugin's release gets the binary fetched from the
/// release URL (curl, never gh — owner ruling 2026-09-30), the pin and the
/// shim rewritten (byte-identical to canonical), and prime run through the
/// new shim — its first line naming what changed and what to commit. A
/// project already current gets prime alone and no fetch.
#[test]
fn plugin_upgrade_brings_adopter_current() {
    let home = tempfile::tempdir().unwrap();
    let project = adopter_behind(home.path());
    stage_release(home.path());
    let tag = plugin_tag();

    let out = run_hook(home.path(), &project);
    let stdout = stdout_text(&out);
    assert!(
        out.status.success(),
        "hook exit {:?}; stderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    let first = stdout.lines().next().unwrap_or_default();
    assert!(
        first.contains(&tag)
            && first.contains("fetched the binary")
            && first.contains("commit .meshwork-version and docs/meshwork/meshwork"),
        "first line names the release, the change and the commit: {first}"
    );
    assert!(
        stdout.contains("alpha — ") && stdout.contains(" open"),
        "prime follows, through the rewritten shim:\n{stdout}"
    );

    assert_eq!(
        std::fs::read_to_string(project.join(".meshwork-version")).unwrap(),
        format!("{tag}\n"),
        "the pin is the plugin's release"
    );
    let shim = project.join("docs/meshwork/meshwork");
    assert_eq!(
        std::fs::read(&shim).unwrap(),
        canonical_shim(),
        "the shim is byte-identical to the canonical one"
    );
    assert!(is_executable(&shim), "the shim stays executable");
    let bin = home.path().join(".meshwork/versions").join(&tag).join("meshwork");
    assert!(is_executable(&bin), "the release binary is cached at {}", bin.display());
    let calls = fetch_calls(home.path());
    let url = format!(
        "https://github.com/jbrjake/meshwork/releases/download/{tag}/{}",
        release_asset(&tag)
    );
    assert!(calls.contains(&url), "the fetch is the release URL, by curl:\n{calls}");
    assert_eq!(calls.lines().count(), 1, "one fetch:\n{calls}");

    // Current already: nothing to report, nothing fetched — prime leads.
    let out = run_hook(home.path(), &project);
    let stdout = stdout_text(&out);
    assert!(out.status.success());
    assert!(
        stdout.starts_with("alpha — "),
        "a current project gets prime as its first line:\n{stdout}"
    );
    assert!(!stdout.contains("commit"), "nothing to commit:\n{stdout}");
    assert_eq!(fetch_calls(home.path()).lines().count(), 1, "no second fetch");
}

/// A failed fetch leaves the project as it was — a moved pin with no
/// binary behind it would break every verb — and says so in the first
/// line.
#[test]
fn plugin_upgrade_failed_fetch_changes_nothing() {
    let home = tempfile::tempdir().unwrap();
    let project = adopter_behind(home.path());
    stage_failed_release(home.path());
    let tag = plugin_tag();

    let out = run_hook(home.path(), &project);
    let stdout = stdout_text(&out);
    assert!(out.status.success(), "a failed fetch never fails the session start");
    let first = stdout.lines().next().unwrap_or_default();
    assert!(
        first.contains("could not fetch")
            && first.contains(&tag)
            && first.contains("404")
            && first.contains(OLD_TAG),
        "loud, naming both releases and why: {first}"
    );
    assert_eq!(
        std::fs::read_to_string(project.join(".meshwork-version")).unwrap(),
        format!("{OLD_TAG}\n"),
        "the pin stays"
    );
    assert_eq!(
        std::fs::read_to_string(project.join("docs/meshwork/meshwork")).unwrap(),
        STALE_SHIM,
        "the shim stays"
    );
    assert!(
        !home.path().join(".meshwork/versions").join(&tag).exists(),
        "nothing cached for a release that did not arrive"
    );
}

/// The hook acts only on a pinned adopter, and primes only where the
/// project's own settings do not: a repo without a store, a store without
/// a pin (built from source), and an adopter still carrying the per-repo
/// prime hook are each left alone in their own way.
#[test]
fn plugin_upgrade_skips_non_adopters_and_never_primes_twice() {
    let home = tempfile::tempdir().unwrap();
    stage_release(home.path());

    // No store: silent.
    let bare = home.path().join("bare");
    std::fs::create_dir_all(&bare).unwrap();
    git(&bare, &["init", "-q"]);
    let out = run_hook(home.path(), &bare);
    assert!(out.status.success());
    assert_eq!(stdout_text(&out), "", "a repo without a store gets nothing");

    // A store without a pin builds meshwork from source: silent.
    let source = home.path().join("source");
    copy_dir(&fixtures_root().join("alpha"), &source);
    git(&source, &["init", "-q"]);
    let out = run_hook(home.path(), &source);
    assert!(out.status.success());
    assert_eq!(stdout_text(&out), "", "an unpinned store is not upgraded or primed");
    assert_eq!(fetch_calls(home.path()), "", "no fetch for either");

    // An adopter whose own settings inject prime: the change line only.
    let project = adopter_behind(home.path());
    std::fs::create_dir_all(project.join(".claude")).unwrap();
    std::fs::write(
        project.join(".claude/settings.json"),
        "{\"hooks\":{\"SessionStart\":[{\"hooks\":[{\"type\":\"command\",\
         \"command\":\"\\\"$CLAUDE_PROJECT_DIR\\\"/docs/meshwork/meshwork prime 2>/dev/null || true\"}]}]}}\n",
    )
    .unwrap();
    let out = run_hook(home.path(), &project);
    let stdout = stdout_text(&out);
    assert!(out.status.success());
    assert_eq!(stdout.lines().count(), 1, "one line — the change, never a second prime:\n{stdout}");
    assert!(stdout.contains("commit .meshwork-version and docs/meshwork/meshwork"), "{stdout}");
    assert_eq!(std::fs::read(project.join("docs/meshwork/meshwork")).unwrap(), canonical_shim());
}

/// The root shim from before it carried an author — `./meshwork` at the
/// repo root, the pin beside it.
const ROOT_SHIM: &str = "#!/bin/sh\n\
exec ~/.meshwork/versions/\"$(cat \"$(dirname \"$0\")/.meshwork-version\")\"/meshwork \"$@\"\n";

/// The root shim keyed on the bridge variable alone.
const ROOT_SHIM_BRIDGE: &str = r#"#!/bin/sh
# agent sessions get a session-tagged author; explicit --as still wins
if [ -z "$MESHWORK_AUTHOR" ] && [ -n "$CLAUDE_CODE_BRIDGE_SESSION_ID" ]; then
  export MESHWORK_AUTHOR="claude ($CLAUDE_CODE_BRIDGE_SESSION_ID)"
fi
exec ~/.meshwork/versions/"$(cat "$(dirname "$0")/.meshwork-version")"/meshwork "$@"
"#;

/// Every shim an adopter may still carry — each text the install reference
/// ever told one to transcribe, at the path it named, oldest first — then
/// the canonical one. `shim_texts_in_history` proves the list complete
/// against git, so a canonical change that retires a text has to add it.
fn footprints() -> Vec<(&'static str, String)> {
    vec![
        ("meshwork", ROOT_SHIM.to_string()),
        ("meshwork", ROOT_SHIM_BRIDGE.to_string()),
        ("docs/meshwork/meshwork", STALE_SHIM.to_string()),
        (
            "docs/meshwork/meshwork",
            String::from_utf8(canonical_shim()).unwrap(),
        ),
    ]
}

/// git in the plugin checkout; `None` when the command fails (a path absent
/// at that commit).
fn git_show(args: &[&str]) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(plugin_root())
        .output()
        .expect("git runs");
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The shim texts a reference page's `printf` blocks write: the one-line
/// `printf '#!/bin/sh\n…' > <shim>` form, and the `printf '%s\n' \` form
/// followed by one quoted shim line per line.
fn printf_blocks(page: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut lines = page.lines().peekable();
    while let Some(line) = lines.next() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("printf '#!/bin/sh") {
            let body = rest.split("' >").next().unwrap_or_default();
            out.push(format!("#!/bin/sh{}", body.replace("\\n", "\n")));
        } else if line.starts_with("printf '%s\\n' \\") {
            let mut text = String::new();
            while let Some(quoted) = lines.peek().and_then(|next| {
                next.trim()
                    .strip_suffix(" \\")
                    .and_then(|q| q.strip_prefix('\''))
                    .and_then(|q| q.strip_suffix('\''))
            }) {
                text.push_str(quoted);
                text.push('\n');
                lines.next();
            }
            if !text.is_empty() {
                out.push(text);
            }
        }
    }
    out
}

/// Every shim text git history holds: each version of `hooks/meshwork`,
/// and each `printf` block the install reference ever carried. Mined,
/// never remembered.
fn shim_texts_in_history() -> Vec<String> {
    let mut texts = Vec::new();
    for (path, mine) in [
        ("hooks/meshwork", false),
        (".claude/skills/meshwork/references/install.md", true),
    ] {
        let log = git_show(&["log", "--format=%H", "--", path]).expect("git log runs");
        for sha in log.lines() {
            let Some(text) = git_show(&["show", &format!("{sha}:{path}")]) else {
                continue;
            };
            if mine {
                texts.extend(printf_blocks(&text));
            } else {
                texts.push(text);
            }
        }
    }
    texts.sort();
    texts.dedup();
    texts
}

/// An adopter as an earlier release left it: the alpha corpus pinned one
/// release back, one shim footprint (or none), and optionally the per-repo
/// prime hook. Nothing else differs between scenarios.
fn adopter_as_left(home: &Path, shim: Option<(&str, &str)>, own_hook: bool) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let repo = home.join("alpha");
    copy_dir(&fixtures_root().join("alpha"), &repo);
    git(&repo, &["init", "-q"]);
    std::fs::write(repo.join(".meshwork-version"), format!("{OLD_TAG}\n")).unwrap();
    if let Some((path, text)) = shim {
        let shim = repo.join(path);
        std::fs::write(&shim, text).unwrap();
        std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    if own_hook {
        std::fs::create_dir_all(repo.join(".claude")).unwrap();
        std::fs::write(
            repo.join(".claude/settings.json"),
            "{\"hooks\":{\"SessionStart\":[{\"hooks\":[{\"type\":\"command\",\
             \"command\":\"\\\"$CLAUDE_PROJECT_DIR\\\"/docs/meshwork/meshwork prime 2>/dev/null || true\"}]}]}}\n",
        )
        .unwrap();
    }
    repo
}

/// Run the committed shim the way a session does: as a program, from the
/// project, `HOME` at the cache, no author or session in the environment
/// beyond what `extra` sets.
fn via_shim(home: &Path, project: &Path, args: &[&str], extra: &[(&str, &str)]) -> std::process::Output {
    let mut cmd = std::process::Command::new(project.join("docs/meshwork/meshwork"));
    cmd.args(args)
        .current_dir(project)
        .env("HOME", home)
        .env("MESHWORK_TRUST", "1")
        .env_remove("MESHWORK_AUTHOR")
        .env_remove("MESHWORK_PORTFOLIO")
        .env_remove("CLAUDE_CODE_SESSION_ID")
        .env_remove("CLAUDE_CODE_BRIDGE_SESSION_ID");
    for (key, value) in extra {
        cmd.env(key, value);
    }
    cmd.output().expect("the shim runs")
}

/// mw-4ffk42q: every release is gated on the upgrade path. For each adopter
/// footprint an earlier release produced — each shim text the install
/// reference ever carried, at its path; no shim at all; the per-repo prime
/// hook; a pin one release back — the plugin's hook must leave the project
/// current: the shim canonical and executable, the pin the plugin's
/// release, nothing outside the store and the pin touched. Then every verb
/// `--help` lists runs through the resulting shim, a session carrying only
/// `CLAUDE_CODE_SESSION_ID` resolves its author through it, and prime
/// through it leads with no shim line. The footprint list is checked
/// against git history first, so a shim change that forgets to extend it
/// fails here.
#[test]
fn adopter_upgrade_path() {
    let tag = plugin_tag();
    let known: Vec<String> = footprints().into_iter().map(|(_, text)| text).collect();
    let mined = shim_texts_in_history();
    assert!(
        mined.len() >= 4,
        "the miner found {} shim text(s) in git history; the install reference alone carried four",
        mined.len()
    );
    for text in &mined {
        assert!(
            known.contains(text),
            "a shim text in git history is not a footprint — add it so the upgrade path \
             covers the adopters still carrying it:\n{text:?}"
        );
    }
    let verbs = crate::arch::verbs_of(&crate::arch::help_of(&[]));
    assert!(verbs.len() > 20, "{verbs:?}");

    let mut scenarios: Vec<Scenario> = footprints()
        .into_iter()
        .map(|(path, text)| Scenario {
            name: format!("{path} ({})", text.lines().nth(1).unwrap_or_default()),
            shim: Some((path, text)),
            own_hook: false,
        })
        .collect();
    scenarios.push(Scenario {
        name: "no shim at all".to_string(),
        shim: None,
        own_hook: false,
    });
    scenarios.push(Scenario {
        name: "stale shim plus the per-repo prime hook".to_string(),
        shim: Some(("docs/meshwork/meshwork", STALE_SHIM.to_string())),
        own_hook: true,
    });

    for scenario in scenarios {
        let home = tempfile::tempdir().unwrap();
        stage_release(home.path());
        let project = adopter_as_left(
            home.path(),
            scenario.shim.as_ref().map(|(path, text)| (*path, text.as_str())),
            scenario.own_hook,
        );
        let legacy_root = scenario
            .shim
            .as_ref()
            .filter(|(path, _)| *path == "meshwork")
            .map(|(_, text)| text.clone());
        let out = run_hook(home.path(), &project);
        assert_brought_current(&scenario.name, &tag, &project, &out, legacy_root.as_deref());
        assert_runs_through_shim(&scenario.name, home.path(), &project, &verbs);
    }
}

/// One adopter as an earlier release left it.
struct Scenario {
    name: String,
    /// The shim's repo-relative path and text; `None` for no shim at all.
    shim: Option<(&'static str, String)>,
    /// Whether the project's own settings inject prime.
    own_hook: bool,
}

/// After the hook: exit 0, the release named first, the shim canonical and
/// executable, the pin the plugin's release, and a legacy root shim left
/// exactly as it was — the hook touches nothing outside the store and
/// the pin.
fn assert_brought_current(
    name: &str,
    tag: &str,
    project: &Path,
    out: &std::process::Output,
    legacy_root: Option<&str>,
) {
    let stdout = stdout_text(out);
    assert!(
        out.status.success(),
        "{name}: hook exit {:?}; stderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        stdout.lines().next().unwrap_or_default().contains(tag),
        "{name}: the first line names the release:\n{stdout}"
    );
    let shim_path = project.join("docs/meshwork/meshwork");
    let shim_now = std::fs::read(&shim_path).unwrap_or_else(|e| {
        panic!("{name}: no shim at docs/meshwork/meshwork after the hook ran: {e}")
    });
    assert_eq!(
        shim_now,
        canonical_shim(),
        "{name}: the shim is byte-identical to canonical"
    );
    assert!(is_executable(&shim_path), "{name}: the shim is executable");
    assert_eq!(
        std::fs::read_to_string(project.join(".meshwork-version")).unwrap(),
        format!("{tag}\n"),
        "{name}: the pin is the plugin's release"
    );
    if let Some(text) = legacy_root {
        assert_eq!(
            std::fs::read_to_string(project.join("meshwork")).unwrap(),
            text,
            "{name}: the hook touches nothing outside the store and the pin"
        );
    }
}

/// Through the resulting shim: every verb's `--help` exits 0 with no
/// not-found, a session carrying only `CLAUDE_CODE_SESSION_ID` signs its
/// comment, and prime leads with the headline — no shim line.
fn assert_runs_through_shim(name: &str, home: &Path, project: &Path, verbs: &[String]) {
    for verb in verbs {
        let out = via_shim(home, project, &[verb, "--help"], &[]);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            out.status.success() && !stderr.contains("not found"),
            "{name}: `{verb} --help` through the shim exited {:?}:\n{stderr}",
            out.status.code()
        );
    }
    let added = via_shim(
        home,
        project,
        &["add", "Probe through the shim", "--verify", "true"],
        &[],
    );
    assert!(added.status.success(), "{name}: add through the shim");
    let id = String::from_utf8_lossy(&added.stdout)
        .lines()
        .next()
        .unwrap_or_default()
        .to_string();
    let commented = via_shim(
        home,
        project,
        &["comment", &id, "through the shim"],
        &[("CLAUDE_CODE_SESSION_ID", "s-4ffk42q")],
    );
    assert!(
        commented.status.success(),
        "{name}: comment through the shim: {}",
        String::from_utf8_lossy(&commented.stderr)
    );
    let show = String::from_utf8_lossy(&via_shim(home, project, &["show", &id], &[]).stdout).into_owned();
    assert!(
        show.contains("claude (s-4ffk42q)"),
        "{name}: the session author resolves through the shim from the CLI variable alone:\n{show}"
    );
    let prime = String::from_utf8_lossy(&via_shim(home, project, &["prime"], &[]).stdout).into_owned();
    assert!(
        prime.starts_with("alpha \u{2014} "),
        "{name}: prime through the shim leads with the headline, no shim line:\n{prime}"
    );
}
