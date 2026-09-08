//! `perf::` — gate §7 (PLAN 2.5, MW-C4): cold `ready` <100ms at 1K tasks,
//! `portfolio ready` <1s at 20 repos, cold `prime` <100ms at 1K
//! comment-tailed tasks (mw-4m169xc); N≥7 reps, median. Budgets are
//! defined for RELEASE builds on the owned machines — §7 runs
//! `cargo test --release -- --ignored perf::`; under a debug build (gate
//! §3's --include-ignored sweep) the tests print a note and skip, because
//! a debug timing is not the thresholded quantity. Corpora are seeded and
//! deterministic. Medians print as `perf-median <name> <ms>` for
//! scripts/check-perf.sh's 1.5× regression wall (baseline rule).

use crate::synth::{synth_store, synth_store_commented, Lcg};
use assert_cmd::Command;
use std::fmt::Write as _;
use std::path::Path;
use std::time::Instant;

const REPS: usize = 7;

fn median_ms(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[samples.len() / 2]
}

/// Like e2e's helper but local to perf: hermetic HOME, no ambient registry.
fn meshwork_at(dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("meshwork").unwrap();
    cmd.current_dir(dir);
    cmd.env("HOME", dir);
    cmd.env_remove("MESHWORK_PORTFOLIO");
    cmd
}

fn git_init(dir: &Path) {
    crate::common::git(dir, &["init", "-q"]);
}

/// MW-C4: cold `ready` at 1,000 tasks — parse + plan + query per
/// invocation, process start included.
#[test]
#[ignore = "gate §7 runs perf:: on release builds (MW-C4)"]
fn ready_1k_cold() {
    if cfg!(debug_assertions) {
        eprintln!("perf::ready_1k_cold: budgets are release-only; skipping in debug");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("synth1k");
    synth_store(&repo, "pf", 1000, &mut Lcg(11));
    git_init(&repo);

    let mut samples = Vec::new();
    for _ in 0..REPS {
        let t = Instant::now();
        meshwork_at(&repo).arg("ready").assert().success();
        samples.push(t.elapsed().as_millis());
    }
    let med = median_ms(samples);
    println!("perf-median ready_1k_cold {med}");
    assert!(
        med < 100,
        "MW-C4: cold ready at 1K tasks — {med}ms >= 100ms"
    );
}

/// Gate §7 covers the `SessionStart` hot path (mw-4m169xc): cold `prime`
/// at 1K tasks, every task carrying a 3-comment tail — parse-time input
/// is the axis that grows without bound; read-time output is already
/// byte-capped (§7). Same 100ms family budget as `ready`: while this
/// holds, the `.cache/tasks.jsonl` projection stays deferrable.
#[test]
#[ignore = "gate §7 runs perf:: on release builds (MW-C4)"]
fn prime_1k() {
    if cfg!(debug_assertions) {
        eprintln!("perf::prime_1k: budgets are release-only; skipping in debug");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("synth1k");
    synth_store_commented(&repo, "pf", 1000, &mut Lcg(11), 3);
    git_init(&repo);

    let mut samples = Vec::new();
    for _ in 0..REPS {
        let t = Instant::now();
        meshwork_at(&repo).arg("prime").assert().success();
        samples.push(t.elapsed().as_millis());
    }
    let med = median_ms(samples);
    println!("perf-median prime_1k {med}");
    assert!(
        med < 100,
        "gate §7: cold prime at 1K comment-tailed tasks — {med}ms >= 100ms"
    );
}

/// The derived projection's budget (PROPOSAL-analytics §7, the MW-C4
/// portfolio number): `q` over `pulse` cold at 1K tasks ≤ 1 s — every
/// view registered and the whole tree planned and run, process start
/// included. `q` over the six tables alone never registers a view and
/// stays on the `ready` budget.
#[test]
#[ignore = "gate §7 runs perf:: on release builds (MW-C4)"]
fn q_pulse_1k_cold() {
    if cfg!(debug_assertions) {
        eprintln!("perf::q_pulse_1k_cold: budgets are release-only; skipping in debug");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("synth1k");
    synth_store(&repo, "pf", 1000, &mut Lcg(11));
    git_init(&repo);

    let mut samples = Vec::new();
    for _ in 0..REPS {
        let t = Instant::now();
        meshwork_at(&repo)
            .args(["q", "SELECT * FROM pulse"])
            .assert()
            .success();
        samples.push(t.elapsed().as_millis());
    }
    let med = median_ms(samples);
    println!("perf-median q_pulse_1k_cold {med}");
    assert!(
        med < 1000,
        "the derived projection: cold q over pulse at 1K tasks — {med}ms >= 1000ms"
    );
}

/// MW-C4: `portfolio ready` over 20 registered repos (50 tasks each).
#[test]
#[ignore = "gate §7 runs perf:: on release builds (MW-C4)"]
fn portfolio_20_repos() {
    if cfg!(debug_assertions) {
        eprintln!("perf::portfolio_20_repos: budgets are release-only; skipping in debug");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let mut lcg = Lcg(23);
    let mut repos_toml = String::new();
    let mut paths = String::from("[paths]\n");
    for r in 0..20 {
        let name = format!("synth{r:02}");
        let repo = dir.path().join(&name);
        synth_store(&repo, &format!("s{r:02}"), 50, &mut lcg);
        let _ = write!(
            repos_toml,
            "[[repo]]\nname = \"{name}\"\nremote = \"x\"\n\n"
        );
        let _ = writeln!(paths, "\"{name}\" = \"{}\"", repo.display());
    }
    let portfolio = dir.path().join("portfolio");
    std::fs::create_dir_all(&portfolio).unwrap();
    std::fs::write(portfolio.join("repos.toml"), repos_toml).unwrap();
    std::fs::write(portfolio.join("repos.local.toml"), paths).unwrap();

    let mut samples = Vec::new();
    for _ in 0..REPS {
        let t = Instant::now();
        meshwork_at(dir.path())
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .args(["portfolio", "ready"])
            .assert()
            .success();
        samples.push(t.elapsed().as_millis());
    }
    let med = median_ms(samples);
    println!("perf-median portfolio_20_repos {med}");
    assert!(
        med < 1000,
        "MW-C4: portfolio ready at 20 repos — {med}ms >= 1000ms"
    );
}
