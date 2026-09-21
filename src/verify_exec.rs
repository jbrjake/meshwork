//! Executor for the verify DSL (mw-dthxs3q; DESIGN §12b, MW-E5): the
//! half that runs what `verify_dsl` parsed. No shell exists in this
//! module — `run` spawns its argv array directly, env scrubbed to a
//! pinned pass-through set, cwd pinned to the repo root, a wall-clock
//! timeout and a byte cap on captured output. `exists`/`absent`/
//! `contains` never spawn at all — they load no code and may run
//! without the MW-E5 trust gate. `run` executes approval-free ONLY
//! under the ride-along guard (owner directive 2026-08-14, DESIGN §12b
//! gate routing): no commit or merge that delivered the task may also
//! have delivered code — a PR carrying a task plus the test its verify
//! names would self-verify against attacker code, and argv safety pins
//! which program starts, not what code cargo loads. Callers gate `run`
//! like legacy shell whenever the guard fails.

use crate::verify_dsl::{Pattern, Predicate};
use std::io::Read;
use std::path::Path;
use std::time::{Duration, Instant};

/// Wall clock for one `run` predicate — generous for a filtered
/// `cargo test`, fatal for a hang.
pub const RUN_TIMEOUT: Duration = Duration::from_mins(5);

/// The wall clock in force: `RUN_TIMEOUT`, or `MESHWORK_RUN_TIMEOUT`
/// (whole seconds) when set — the determinism hook that lets a test
/// watch a hang die without waiting five minutes for it (DESIGN §15.6).
#[must_use]
pub fn run_timeout() -> Duration {
    std::env::var("MESHWORK_RUN_TIMEOUT")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .filter(|s| *s > 0)
        .map_or(RUN_TIMEOUT, Duration::from_secs)
}

/// Why a spawn produced no exit status.
#[derive(Debug)]
pub enum SpawnError {
    /// The wall clock expired; the child was killed.
    Timeout(Duration),
    /// The child never started, or waiting on it failed.
    Other(String),
}

impl std::fmt::Display for SpawnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Timeout(t) => write!(f, "timeout after {}s", t.as_secs_f32()),
            Self::Other(e) => f.write_str(e),
        }
    }
}
/// Captured-output byte cap per `run`; the child may write more — the
/// excess is drained and dropped, never buffered.
pub const OUTPUT_CAP: usize = 262_144;
/// Env vars the child inherits; everything else is scrubbed. `PATH`
/// resolves the runner; `HOME`/`CARGO_HOME`/`TMPDIR` keep `cargo`
/// functional.
const KEPT_ENV: &[&str] = &["PATH", "HOME", "CARGO_HOME", "TMPDIR"];

/// Evaluate predicates as a conjunction, fail-fast.
///
/// # Errors
/// The first failing predicate, with why — a missing/present path, a
/// non-matching or unreadable `contains`, or a `run` that exited
/// nonzero, timed out, or failed to spawn.
pub fn execute(root: &Path, preds: &[Predicate]) -> Result<(), String> {
    for p in preds {
        match p {
            Predicate::Exists { path } if path.contains('*') => {
                if !glob_exists(root, path)? {
                    return Err(format!("exists {path}: nothing matches"));
                }
            }
            Predicate::Exists { path } => {
                if !safe_join(root, path)?.exists() {
                    return Err(format!("exists {path}: no such path"));
                }
            }
            Predicate::Absent { path } => {
                if safe_join(root, path)?.exists() {
                    return Err(format!("absent {path}: path exists"));
                }
            }
            Predicate::Contains { path, pattern } => {
                let text = read_file(root, "contains", path)?;
                if !matches(&text, pattern)? {
                    return Err(format!("contains {path} {pattern}: no match"));
                }
            }
            // MW-N3: the inverse, with one asymmetry — a file that is not
            // there is not a file that lacks the pattern.
            Predicate::Lacks { path, pattern } => {
                let text = read_file(root, "lacks", path)?;
                if matches(&text, pattern)? {
                    return Err(format!("lacks {path} {pattern}: found"));
                }
            }
            Predicate::Run { argv } => {
                let spawned = spawn_argv(argv);
                let out = run_argv(root, &spawned, run_timeout(), OUTPUT_CAP)
                    .map_err(|e| format!("run {}: {e}", argv.join(" ")))?;
                require_non_vacuous(&spawned, &out)?;
            }
        }
    }
    Ok(())
}

/// The argv the executor spawns for a `run` predicate: the tokens as
/// written, with `package=<crate>` and `target=<name>` spelled as
/// `-p <crate>` and `--test <name>` (MW-N1). This is the only place
/// author text becomes a flag, and only from these two prefixes.
#[must_use]
pub fn spawn_argv(argv: &[String]) -> Vec<String> {
    let mut out = Vec::with_capacity(argv.len() + 2);
    for arg in argv {
        match crate::verify_dsl::SCOPING_TOKENS
            .iter()
            .find_map(|(prefix, flag)| arg.strip_prefix(prefix).map(|v| (*flag, v)))
        {
            Some((flag, value)) => {
                out.push(flag.to_string());
                out.push(value.to_string());
            }
            None => out.push(arg.clone()),
        }
    }
    out
}

/// One confined file for `contains`/`lacks`: a directory refuses rather
/// than being walked (MW-N4), and a missing file refuses by name — for
/// `lacks` that refusal is the whole point (MW-N3).
fn read_file(root: &Path, kw: &str, path: &str) -> Result<String, String> {
    let on_disk = safe_join(root, path)?;
    if on_disk.is_dir() {
        return Err(format!("{kw} {path}: is a directory — {kw} reads one file"));
    }
    if !on_disk.exists() {
        return Err(format!(
            "{kw} {path}: missing file — a file that is not there cannot be read"
        ));
    }
    std::fs::read_to_string(on_disk).map_err(|e| format!("{kw} {path}: {e}"))
}

/// `exists` with one `*` in the last segment: the parent directory is
/// confined and listed once; a name matches when it carries the literal
/// prefix and suffix around the star (MW-N4).
fn glob_exists(root: &Path, pattern: &str) -> Result<bool, String> {
    let (dir, name) = pattern.rsplit_once('/').unwrap_or((".", pattern));
    let (prefix, suffix) = name.split_once('*').unwrap_or((name, ""));
    let Ok(entries) = std::fs::read_dir(safe_join(root, dir)?) else {
        return Ok(false);
    };
    Ok(entries.flatten().any(|e| {
        let n = e.file_name().to_string_lossy().into_owned();
        n.len() >= prefix.len() + suffix.len() && n.starts_with(prefix) && n.ends_with(suffix)
    }))
}

/// `cargo test` exits 0 when a filter matches nothing — the vacuous pass
/// the store's observed-pass shell idiom existed to plug. Ruled 2026-08-14
/// (mw-4aqmf0t, DESIGN §12b): a `run cargo test` predicate is green only
/// when some suite reports `ok. N passed` with N ≥ 1 in the captured
/// output. Other runners/subcommands pass on exit 0 alone.
fn require_non_vacuous(argv: &[String], out: &str) -> Result<(), String> {
    let is_cargo_test = argv.first().map(String::as_str) == Some("cargo")
        && argv.get(1).map(String::as_str) == Some("test");
    if !is_cargo_test
        || regex::Regex::new(r"ok\. [1-9][0-9]* passed")
            .unwrap()
            .is_match(out)
    {
        return Ok(());
    }
    Err(format!(
        "run {}: vacuous pass — exit 0 but no `ok. N passed` (N ≥ 1) in the \
         output; a cargo test filter matching nothing still exits 0",
        argv.join(" ")
    ))
}

/// Belt over the parser's braces: re-refuse absolute, traversing, and
/// symlink-escaping paths even on programmatically built predicates —
/// the shared confinement (mw-2pz0zqc).
fn safe_join(root: &Path, rel: &str) -> Result<std::path::PathBuf, String> {
    crate::paths::confine(root, rel)
}

/// `contains` over a file: a literal is a substring; a regex is grep-like —
/// `^` and `$` anchor lines, never the whole file — because every author
/// writes `/^## Heading/` and `/^- 2026-…/` meaning "a line", and the
/// owner-gated marker idiom depends on exactly that.
fn matches(text: &str, pattern: &Pattern) -> Result<bool, String> {
    match pattern {
        Pattern::Literal(lit) => Ok(text.contains(lit)),
        Pattern::Regex(re) => regex::RegexBuilder::new(re)
            .multi_line(true)
            .build()
            .map(|re| re.is_match(text))
            .map_err(|e| format!("bad regex /{re}/: {e}")),
    }
}

/// Spawn `argv` directly — argv[0] resolved via PATH, arguments passed
/// verbatim, NEVER a shell — from `root`, env scrubbed to `KEPT_ENV`.
/// `Ok` is the capped combined stdout+stderr of a zero exit.
///
/// # Errors
/// Spawn failure, a nonzero exit (with an output tail), or the wall
/// clock expiring — the child is killed on timeout, never orphaned.
pub fn run_argv(
    root: &Path,
    argv: &[String],
    timeout: Duration,
    out_cap: usize,
) -> Result<String, String> {
    let (status, out) = spawn_capped(root, argv, timeout, out_cap).map_err(|e| e.to_string())?;
    if status.success() {
        Ok(out)
    } else {
        let tail = out.chars().rev().take(400).collect::<String>();
        let tail: String = tail.chars().rev().collect();
        Err(format!("{status}; output tail: {tail}"))
    }
}

/// The spawn under [`run_argv`], exit status and all: the same argv
/// discipline, env scrub, cwd pin, output cap and wall clock, with the
/// verdict left to the caller — the start red-check reads the exit code
/// itself (0 is "already green", 127 is "no such command").
///
/// # Errors
/// Spawn failure or the wall clock expiring — the child is killed on
/// timeout, never orphaned. A nonzero exit is not an error here.
pub fn spawn_capped(
    root: &Path,
    argv: &[String],
    timeout: Duration,
    out_cap: usize,
) -> Result<(std::process::ExitStatus, String), SpawnError> {
    let (first, rest) = argv
        .split_first()
        .ok_or_else(|| SpawnError::Other("empty argv".into()))?;
    let mut cmd = std::process::Command::new(first);
    cmd.args(rest)
        .current_dir(root)
        .env_clear()
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    for key in KEPT_ENV {
        if let Ok(v) = std::env::var(key) {
            cmd.env(key, v);
        }
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| SpawnError::Other(format!("spawn {first}: {e}")))?;
    // Drain pipes on threads: the cap bounds what we keep, while the
    // drain keeps a chatty child from blocking on a full pipe.
    let stdout = drain(child.stdout.take(), out_cap);
    let stderr = drain(child.stderr.take(), out_cap);

    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(SpawnError::Timeout(timeout));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(25)),
            Err(e) => return Err(SpawnError::Other(format!("wait {first}: {e}"))),
        }
    };
    let mut out = stdout.join().unwrap_or_default();
    let err_tail = stderr.join().unwrap_or_default();
    if out.len() < out_cap {
        let spare = out_cap - out.len();
        out.push_str(&err_tail[..err_tail.len().min(spare)]);
    }
    Ok((status, out))
}

/// Read a pipe to the byte cap, then keep draining into the void so the
/// child never blocks; lossy-decode what was kept.
fn drain<R: Read + Send + 'static>(
    stream: Option<R>,
    cap: usize,
) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        let Some(mut stream) = stream else {
            return String::new();
        };
        let mut kept = Vec::new();
        let mut buf = [0u8; 8192];
        loop {
            match stream.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    let room = cap.saturating_sub(kept.len());
                    kept.extend_from_slice(&buf[..n.min(room)]);
                }
            }
        }
        String::from_utf8_lossy(&kept).into_owned()
    })
}
