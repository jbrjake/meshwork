//! Executor for the verify DSL (mw-dthxs3q; DESIGN §12b, MW-E5): the
//! half that runs what `verify_dsl` parsed. No shell exists in this
//! module — `run` spawns its argv array directly, env scrubbed to a
//! pinned pass-through set, cwd pinned to the repo root, a wall-clock
//! timeout and a byte cap on captured output. `exists`/`absent`/
//! `contains` never spawn at all — they load no code and may run
//! without the MW-E5 trust gate. `run` executes approval-free ONLY
//! under the ride-along guard (owner directive 2026-08-14, DESIGN §12b
//! gate routing): the task's git history must be store-only — a test or
//! `build.rs` arriving in the same merge as the task that names it is
//! the drive-by payload, and argv safety pins which program starts, not
//! what code cargo loads. Callers gate `run` like legacy shell whenever
//! the guard fails.

use crate::verify_dsl::{Pattern, Predicate};
use std::io::Read;
use std::path::Path;
use std::time::{Duration, Instant};

/// Wall clock for one `run` predicate — generous for a filtered
/// `cargo test`, fatal for a hang.
pub const RUN_TIMEOUT: Duration = Duration::from_mins(5);
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
                let text = std::fs::read_to_string(safe_join(root, path)?)
                    .map_err(|e| format!("contains {path}: {e}"))?;
                if !matches(&text, pattern)? {
                    return Err(format!("contains {path} {pattern}: no match"));
                }
            }
            Predicate::Run { argv } => {
                run_argv(root, argv, RUN_TIMEOUT, OUTPUT_CAP)
                    .map_err(|e| format!("run {}: {e}", argv.join(" ")))?;
            }
        }
    }
    Ok(())
}

/// Belt over the parser's braces: re-refuse absolute and traversing
/// paths even on programmatically built predicates.
fn safe_join(root: &Path, rel: &str) -> Result<std::path::PathBuf, String> {
    if rel.starts_with('/') || rel.split('/').any(|seg| seg == "..") {
        return Err(format!("unsafe path: {rel}"));
    }
    Ok(root.join(rel))
}

fn matches(text: &str, pattern: &Pattern) -> Result<bool, String> {
    match pattern {
        Pattern::Literal(lit) => Ok(text.contains(lit)),
        Pattern::Regex(re) => regex::Regex::new(re)
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
    let (first, rest) = argv.split_first().ok_or("empty argv")?;
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
    let mut child = cmd.spawn().map_err(|e| format!("spawn {first}: {e}"))?;
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
                return Err(format!("timeout after {}s", timeout.as_secs_f32()));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(25)),
            Err(e) => return Err(format!("wait {first}: {e}")),
        }
    };
    let mut out = stdout.join().unwrap_or_default();
    let err_tail = stderr.join().unwrap_or_default();
    if out.len() < out_cap {
        let spare = out_cap - out.len();
        out.push_str(&err_tail[..err_tail.len().min(spare)]);
    }
    if status.success() {
        Ok(out)
    } else {
        let tail = out.chars().rev().take(400).collect::<String>();
        let tail: String = tail.chars().rev().collect();
        Err(format!("{status}; output tail: {tail}"))
    }
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
