//! Deterministic synthetic-store generator shared by `perf::` (gate §7)
//! and `benches/startup.rs` (mw-xjyhs9y) — the bench includes this file
//! via `#[path]`, so it must stay std-only and self-contained.

use std::fmt::Write as _;
use std::path::Path;

/// Deterministic LCG — the corpus must be identical run-to-run; ids come
/// from the loop counter (unique by construction), the LCG only mixes
/// statuses and edges.
pub struct Lcg(pub u64);

impl Lcg {
    pub fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }
}

/// Write a synthetic store: config.toml + `n` task files with a realistic
/// mix — ~20% done, ~10% doing, a third carrying a needs edge, some seq.
pub fn synth_store(root: &Path, alias: &str, n: usize, lcg: &mut Lcg) {
    synth_store_commented(root, alias, n, lcg, 0);
}

/// `synth_store` plus a `comments`-deep tail of ~200-byte comment lines
/// on every task — the parse-time-input growth case (mw-4m169xc):
/// read-time output is byte-capped, parse-time input isn't.
/// `comments = 0` reproduces `synth_store` byte-for-byte.
pub fn synth_store_commented(root: &Path, alias: &str, n: usize, lcg: &mut Lcg, comments: usize) {
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
        let mut body = format!(
            "---\nid: {id}\ntitle: Synthetic task {i}\nstatus: {status}\n\
             category: synth/load\nverify: \"true\"\n{needs}{seq}\
             created: 2026-07-01\n---\nGenerated corpus row (gate §7).\n\n\
             ## log\n- 2026-07-01 created\n"
        );
        if comments > 0 {
            body.push_str("\n## comments\n");
            for c in 0..comments {
                let _ = writeln!(
                    body,
                    "- 2026-07-0{}T10:00Z [synth] Comment {c} on task {i}: \
                     a realistically sized note carrying enough prose that \
                     the parse-time cost of a long comment tail shows up in \
                     the measurement rather than rounding to nothing.",
                    (c % 7) + 2
                );
            }
        }
        std::fs::write(tasks.join(format!("{id}-synthetic-{i}.md")), body).unwrap();
        prev = Some(id);
    }
}
