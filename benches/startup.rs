//! `cargo bench --bench startup` — the engine constant (mw-xjyhs9y).
//!
//! Isolates `DataFusion` parse/plan/query cost from bare process start by
//! timing the compiled binary end-to-end at 1K tasks: `--version` is
//! process start + clap alone; `ready` adds store parse, planning, and
//! the query. The delta is the engine constant. N≥7 medians (house
//! rule), one unmeasured warm-up per command; corpus seeded and
//! deterministic — the same shape and seed as `perf::ready_1k_cold`, so
//! the numbers line up with gate §7's budgets. Results are committed to
//! docs/bench-startup.md; re-run this bench to refresh them.

#[path = "../tests/suite/synth.rs"]
mod synth;

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

const REPS: usize = 9;

fn median_ms(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[samples.len() / 2]
}

/// One timed invocation: hermetic HOME, no ambient registry, output
/// discarded (writing to a terminal would be part of the measurement).
fn timed_run(bin: &str, dir: &Path, args: &[&str]) -> u128 {
    let t = Instant::now();
    let status = Command::new(bin)
        .args(args)
        .current_dir(dir)
        .env("HOME", dir)
        .env_remove("MESHWORK_PORTFOLIO")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    let ms = t.elapsed().as_millis();
    assert!(status.success(), "meshwork {args:?} exited nonzero");
    ms
}

fn medians_for(bin: &str, dir: &Path, args: &[&str]) -> u128 {
    timed_run(bin, dir, args); // warm-up: page/dyld caches, not measured
    median_ms((0..REPS).map(|_| timed_run(bin, dir, args)).collect())
}

fn main() {
    let bin = env!("CARGO_BIN_EXE_meshwork");
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("synth1k");
    synth::synth_store(&repo, "pf", 1000, &mut synth::Lcg(11));
    let git = Command::new("git")
        .args(["init", "-q"])
        .current_dir(&repo)
        .status()
        .unwrap();
    assert!(git.success(), "git init failed");

    let version = medians_for(bin, &repo, &["--version"]);
    let ready = medians_for(bin, &repo, &["ready"]);
    println!("bench-median startup_version {version}");
    println!("bench-median startup_ready_1k {ready}");
    println!(
        "bench-delta engine_constant_1k {}",
        ready.saturating_sub(version)
    );
}
