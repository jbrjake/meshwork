// e2e part-file: prime as the materialized handoff (DESIGN §7b, mw-a8tv).
// include!d from e2e.rs so test paths stay flat (`e2e::prime_handoff_sections`).

/// Write a minimal open task file directly — the rollup-cap scenario needs
/// exact categories and seqs, which `add` doesn't expose.
fn write_rollup_task(repo: &Path, id: &str, cat: &str, seq: i64) {
    let path = repo.join("meshwork/tasks").join(format!("{id}-t.md"));
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\ntitle: rollup probe {cat}\nstatus: open\ncategory: {cat}\nseq: {seq}\ncreated: 2026-08-01\n---\n"
        ),
    )
    .unwrap();
}

/// DESIGN §7b (mw-a8tv): prime is the materialized handoff. Headline rollup
/// capped at top-5 groups by min seq; derived weather (doing + blocked +
/// freshest comments); next-task block led by its `handoff:` commentary,
/// then mechanics and the blocks-line; recently-done dated from log lines,
/// newest first. All inside the 6KB budget (MW-D3).
#[test]
fn prime_handoff_sections() {
    let (_g, repo) = fixture_repo("alpha");
    let out = stdout_of(&meshwork(&repo).arg("prime").assert().success());

    // Headline: counts line first, rollup second — groups keyed by the first
    // two category segments, ranked by min seq among open members.
    assert!(out.lines().next().unwrap().contains("open"), "counts first:\n{out}");
    let rollup = out.lines().nth(1).unwrap().to_string();
    let pos = |s: &str| rollup.find(s).unwrap_or_else(|| panic!("{s} in rollup: {rollup}"));
    assert!(
        pos("engine/spill") < pos("docs")
            && pos("docs") < pos("tools/bench")
            && pos("tools/bench") < pos("engine/exec"),
        "min-seq group order: {rollup}"
    );
    assert!(!rollup.contains("tools/config"), "done-only group absent: {rollup}");
    assert!(!rollup.contains("engine/spill/budget"), "subcats fold into group: {rollup}");

    // Weather — all derived: doing with last log, blocked with reason,
    // freshest comments on the active frontier.
    let weather_at = out.find("weather:").expect("weather section");
    assert!(out.contains("az-t5k1") && out.contains("bisecting"), "doing + last log:\n{out}");
    assert!(out.contains("az-b10k") && out.contains("datafusion 52"), "blocked + reason:\n{out}");
    assert!(out.contains("wakeup=250ms"), "freshest comment (az-c0m9 2026-08-04):\n{out}");

    // Next block: az-n33d, its handoff: voice FIRST, mechanics after.
    let next_at = out.find("next →").expect("next block");
    let voice_at = out.find("» Cliff numbers are already in the bench notes")
        .expect("handoff commentary renders, » -prefixed");
    let verify_at = out.find("verify: test -f docs/spill-report.md").expect("verify line");
    assert!(weather_at < next_at && next_at < voice_at && voice_at < verify_at,
        "section order weather < next < voice < mechanics:\n{out}");
    // az-n33d has no dependents (az-r3l8 only relates:, a soft link) — the
    // blocks-line appears where the graph has real edges: az-q2r4's row.
    assert!(out.contains("blocks: az-cw55, az-z7a1"), "what az-q2r4 unblocks:\n{out}");
    assert!(out.contains("[docs]"), "category rides along:\n{out}");

    // Recently done: dated from `→done` log lines, newest first.
    let rd = out.find("recently done").expect("recently done section");
    let tail = &out[rd..];
    let dpos = |s: &str| tail.find(s).unwrap_or_else(|| panic!("{s} in dones: {tail}"));
    assert!(dpos("az-d0n3") < dpos("az-j6h5") && dpos("az-j6h5") < dpos("az-m6t7"),
        "done dates descend:\n{tail}");
    assert!(tail.contains("2026-08-01"), "done-date shown:\n{tail}");

    // Budget holds with every section live.
    assert!(out.len() <= 6144, "budget (MW-D3): {} bytes", out.len());

    // Rollup cap: 7 distinct groups → top 5 shown, rest collapses to +N.
    let (_g2, capped) = git_repo("capped");
    init_store(&capped);
    for (i, cat) in ["aa/one", "bb/two", "cc/three", "dd/four", "ee/five", "ff/six", "gg/seven"]
        .iter()
        .enumerate()
    {
        write_rollup_task(&capped, &format!("cp-r{i:03}"), cat, (i64::try_from(i).unwrap() + 1) * 10);
    }
    let out = stdout_of(&meshwork(&capped).arg("prime").assert().success());
    let rollup = out.lines().nth(1).unwrap_or_default().to_string();
    assert!(rollup.contains("aa/one") && rollup.contains("ee/five"), "top-5 kept: {rollup}");
    assert!(!rollup.contains("ff/six") && !rollup.contains("gg/seven"), "past-5 cut: {rollup}");
    assert!(rollup.contains("+2"), "cut is loud (MW-D2): {rollup}");
}
