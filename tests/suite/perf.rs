//! `perf::` — gate §7 (PLAN 2.5, MW-C4): cold `ready` <100ms at 1K tasks,
//! `portfolio ready` <1s at 20 repos; N≥7 reps, median. Budgets are
//! defined for RELEASE builds on the owned machines — §7 runs
//! `cargo test --release -- --ignored perf::`; under a debug build (gate
//! §3's --include-ignored sweep) the tests print a note and skip, because
//! a debug timing is not the thresholded quantity. Corpora are seeded and
//! deterministic. Medians print as `perf-median <name> <ms>` for
//! scripts/check-perf.sh's 1.5× regression wall (baseline rule).

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

/// Deterministic LCG — the corpus must be identical run-to-run; ids come
/// from the loop counter (unique by construction), the LCG only mixes
/// statuses and edges.
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }
}

/// Write a synthetic store: config.toml + `n` task files with a realistic
/// mix — ~20% done, ~10% doing, a third carrying a needs edge, some seq.
fn synth_store(root: &Path, alias: &str, n: usize, lcg: &mut Lcg) {
    let tasks = root.join("docs").join("meshwork");
    std::fs::create_dir_all(&tasks).unwrap();
    std::fs::write(
        root.join("docs/meshwork/config.toml"),
        format!("alias = \"{alias}\"\ndefault_author = \"synth\"\n"),
    )
    .unwrap();
    let mut prev: Option<String> = None;
    for i in 0..n {
        let id = format!("{alias}-{i:07x}");
        let status = match lcg.next() % 10 {
            0 | 1 => "done",
            2 => "doing",
            _ => "open",
        };
        let needs = match (&prev, lcg.next() % 3) {
            (Some(p), 0) => format!("needs: [{p}]\n"),
            _ => String::new(),
        };
        let seq = if lcg.next().is_multiple_of(5) {
            format!("seq: {}\n", (i + 1) * 10)
        } else {
            String::new()
        };
        std::fs::write(
            tasks.join(format!("{id}-synthetic-{i}.md")),
            format!(
                "---\nid: {id}\ntitle: Synthetic task {i}\nstatus: {status}\n\
                 category: synth/load\nverify: \"true\"\n{needs}{seq}\
                 created: 2026-07-01\n---\nGenerated corpus row (gate §7).\n\n\
                 ## log\n- 2026-07-01 created\n"
            ),
        )
        .unwrap();
        prev = Some(id);
    }
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
