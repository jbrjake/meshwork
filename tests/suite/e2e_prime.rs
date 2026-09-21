// e2e part-file: prime as the materialized handoff (DESIGN §7b, mw-a8tv).
// include!d from e2e.rs so test paths stay flat (`e2e::prime_handoff_sections`).

/// Write a minimal open task file directly — the rollup-cap scenario needs
/// exact categories and seqs, which `add` doesn't expose.
fn write_rollup_task(repo: &Path, id: &str, cat: &str, seq: i64) {
    let path = repo.join("docs/meshwork").join(format!("{id}-t.md"));
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

/// mw-drrvpsg: pure provenance stamps (`imported from …`, bare `created`)
/// carry zero session information — the sazed pilot spent 8 of prime's
/// weather lines on the identical import stamp. The doing-tail is the
/// newest SUBSTANTIVE log entry, or nothing after the title.
#[test]
fn weather_skips_import_log() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let mw = repo.join("docs/meshwork");
    // Imported-as-doing: provenance is the only log entry (the pilot case).
    std::fs::write(
        mw.join("wo-imp1-imported-noise.md"),
        "---\nid: wo-imp1\ntitle: Imported noise\nstatus: doing\ncreated: 2026-08-07\n---\n\n\
         ## log\n- 2026-08-07T04:20Z imported from TODO.md\n",
    )
    .unwrap();
    // Imported, then really progressed: the substantive entry wins.
    std::fs::write(
        mw.join("wo-imp2-imported-progress.md"),
        "---\nid: wo-imp2\ntitle: Imported progress\nstatus: doing\ncreated: 2026-08-07\n---\n\n\
         ## log\n- 2026-08-07T04:20Z imported from TODO.md\n- 2026-08-08T10:00Z open\u{2192}doing\n",
    )
    .unwrap();
    // A bare `created` stamp is provenance too.
    std::fs::write(
        mw.join("wo-new1-created-only.md"),
        "---\nid: wo-new1\ntitle: Created only\nstatus: doing\ncreated: 2026-08-09\n---\n\n\
         ## log\n- 2026-08-09T09:00Z created\n",
    )
    .unwrap();

    let out = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    assert!(!out.contains("imported from TODO.md"), "provenance is noise:\n{out}");
    let imp1 = out.lines().find(|l| l.contains("wo-imp1")).expect("doing line");
    assert!(!imp1.contains('\u{2014}'), "nothing after the title: {imp1}");
    assert!(out.contains("open\u{2192}doing"), "substantive entry survives:\n{out}");
    let new1 = out.lines().find(|l| l.contains("wo-new1")).expect("doing line");
    assert!(!new1.contains('\u{2014}'), "bare created is provenance: {new1}");
}

/// mw-06j1wqe: prime's weather ages the rot in place — a doing task with
/// old dated activity carries `[stale: Nd]`, a fresh one stays bare, so
/// the digest stops normalizing a doing list that only ever grows.
#[test]
fn prime_annotates_stale_doing() {
    let (_g, repo) = git_repo("doing-rot");
    init_store(&repo);
    let id = stdout_of(
        &meshwork(&repo)
            .env("MESHWORK_TODAY", "2026-08-01")
            .args(["add", "Left running", "--verify", "true"])
            .assert()
            .success(),
    )
    .lines()
    .next()
    .unwrap()
    .to_string();
    meshwork(&repo)
        .env("MESHWORK_TODAY", "2026-08-01")
        .args(["start", &id, "--as", "worker"])
        .assert()
        .success();

    let fresh = stdout_of(
        &meshwork(&repo)
            .env("MESHWORK_TODAY", "2026-08-02")
            .arg("prime")
            .assert()
            .success(),
    );
    assert!(
        fresh.contains(&id) && !fresh.contains("[stale:"),
        "one day of silence is not rot:\n{fresh}"
    );

    let later = stdout_of(
        &meshwork(&repo)
            .env("MESHWORK_TODAY", "2026-08-20")
            .arg("prime")
            .assert()
            .success(),
    );
    let line = later
        .lines()
        .find(|l| l.contains(&id))
        .unwrap_or_else(|| panic!("doing line present:\n{later}"));
    assert!(line.contains("[stale: 19d]"), "aged in place: {line}");
}

/// mw-yyf1bab: prime nudges when a live verify no longer matches what
/// this clone approved — the session hears about the edit before it
/// commits to a task; lint carries the full approved-vs-current diff.
#[test]
fn prime_flags_verify_changed_since_approval() {
    let (_g, repo) = fixture_repo("alpha");
    let before = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    assert!(
        !before.contains("verify changed since approval"),
        "no approvals recorded, nothing to flag:\n{before}"
    );
    // The operator approved one text; the store now carries another.
    meshwork::trust::record_approval(&repo, "az-n33d", "test -f docs/spill-report-draft.md")
        .unwrap();
    let out = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    assert!(
        out.contains("! verify changed since approval: az-n33d"),
        "the edited verify is named:\n{out}"
    );
}

/// mw-p6atpxh: a bare "1 invalid" count sat unactioned for two full
/// sessions while the broken task silently vanished from ready — when
/// the count is nonzero, prime spends the bytes to name each file and
/// say `run lint`.
#[test]
fn prime_names_invalid() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    add_task(&repo, "Healthy");
    std::fs::write(
        repo.join("docs/meshwork/wo-br0ken1-damaged.md"),
        "---\nid: wo-br0ken1\ntitle: [unclosed\nstatus: open\n---\nbody\n",
    )
    .unwrap();

    let out = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    assert!(
        out.contains("wo-br0ken1"),
        "the invalid file is named, not just counted:\n{out}"
    );
    assert!(out.contains("run lint"), "names the next step:\n{out}");
}

/// Write `n` open asks from alpha to beta, `created` on successive days
/// from `day` (a `YYYY-MM-` prefix); titles padded to `pad` bytes.
fn write_asks(alpha: &Path, n: usize, day: &str, pad: usize) -> Vec<String> {
    (0..n)
        .map(|i| {
            let id = format!("az-ask{i:04}");
            let created = format!("{day}{:02}", i % 28 + 1);
            let title = format!("Ask {i:02} {}", "x".repeat(pad));
            std::fs::write(
                alpha.join(format!("docs/meshwork/{id}-ask.md")),
                format!(
                    "---\nid: {id}\ntitle: {title}\nstatus: open\nto: beta\n\
                     created: {created}\n---\n\n## log\n- {created} created\n"
                ),
            )
            .unwrap();
            format!("alpha#{id}")
        })
        .collect()
}

/// mw-0a084qy: the inbox is either whole or a pointer. Every inbound ask
/// prints while it fits the budget; when it cannot, one line carries the
/// count, the oldest age and the exact `portfolio q` that lists them —
/// never a `… and N more` with nothing to run. `ready` names the verb in
/// its footnote and lifts the cap under `--all`.
#[test]
fn prime_inbox_lists_all_or_names_the_verb() {
    let (dir, portfolio) = portfolio_fixture();
    let alpha = dir.path().join("alpha");
    let beta = dir.path().join("beta");
    let at = |verb: &[&str]| {
        let mut c = meshwork(&beta);
        c.env("MESHWORK_PORTFOLIO", &portfolio)
            .env("MESHWORK_TODAY", "2026-09-07")
            .args(verb);
        stdout_of(&c.assert().success())
    };

    // Seven asks fit: every one is listed, nothing is elided.
    let gids = write_asks(&alpha, 7, "2026-08-", 0);
    let out = at(&["prime"]);
    for gid in &gids {
        assert!(out.contains(gid), "{gid} listed:\n{out}");
    }
    assert!(!out.contains("more addressed"), "{out}");
    assert!(out.contains("addressed to this repo (7):"), "{out}");

    // ready: the cap holds, the footnote names the verb; --all lifts it.
    let ready = at(&["ready"]);
    assert!(ready.contains("… and 2 more addressed"), "{ready}");
    assert!(ready.contains("portfolio q"), "{ready}");
    let all = at(&["ready", "--all"]);
    for gid in &gids {
        assert!(all.contains(gid), "{gid} under --all:\n{all}");
    }
    assert!(!all.contains("more addressed"), "{all}");

    // Forty long-titled asks cannot fit: the list collapses to one line
    // with the count, the oldest age and the statement to run.
    for gid in &gids {
        std::fs::remove_file(alpha.join(format!(
            "docs/meshwork/{}-ask.md",
            gid.trim_start_matches("alpha#")
        )))
        .unwrap();
    }
    let gids = write_asks(&alpha, 40, "2026-08-", 120);
    let out = at(&["prime"]);
    assert!(out.len() <= 6144, "budget: {} bytes", out.len());
    assert!(
        out.contains("addressed to this repo: 40 asks, oldest 37d"),
        "{out}"
    );
    assert!(
        out.contains("portfolio q \"SELECT gid, title FROM asks WHERE to_repo = 'beta' AND unanswered ORDER BY age_h DESC\""),
        "{out}"
    );
    assert!(
        gids.iter().all(|g| !out.contains(g.as_str())),
        "the collapsed inbox lists no ask: {out}"
    );
}

/// mw-d539ppk: the digest ends on the lines that say what to do next
/// with it — re-run it when the question changes, load the skill before
/// filing — and those lines survive the budget's truncation.
#[test]
fn prime_footer_names_skill() {
    let (_g, repo) = fixture_repo("alpha");
    let out = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    let last = out.lines().last().unwrap_or_default();
    assert!(
        last.contains("re-run prime") && last.contains("meshwork skill"),
        "{out}"
    );

    let (_g2, big) = git_repo("bulk");
    init_store(&big);
    let long = "very long title segment that pads the line ".repeat(4);
    for i in 0..60 {
        let id = add_task(&big, &format!("Doing {i:02} {long}"));
        meshwork(&big).args(["start", &id]).assert().success();
    }
    let out = stdout_of(&meshwork(&big).arg("prime").assert().success());
    assert!(out.len() <= 6144, "budget: {} bytes", out.len());
    assert!(out.contains("truncated"), "{out}");
    let tail: Vec<&str> = out.lines().rev().take(2).collect();
    assert!(
        tail[0].contains("meshwork skill") && tail[1].starts_with("rules:"),
        "the footer outlives the cut:\n{out}"
    );
}

/// MW-L6 (owner ruling 2026-09-20, R-B item B.6): the footer states the
/// four session rules in two lines inside the budget — a spoken
/// instruction becomes a task and the answer is an id; an ask is a `to:`
/// line in the author's own store; an owner-scoped field raises a
/// conflict, never an edit; a ruling counts only from this transcript —
/// in the binary, so every adopting repo's session start carries them.
#[test]
fn prime_footer_rules() {
    let (_g, repo) = fixture_repo("alpha");
    let out = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    let lines: Vec<&str> = out.lines().collect();
    let footer = &lines[lines.len() - 2..];
    assert!(footer[0].starts_with("rules:"), "{out}");
    let joined = footer.join("\n");
    for rule in [
        "becomes a task before the work starts",
        "the answer is an id",
        "to: line in your own store",
        "conflict, never an edit",
        "counts only from this transcript",
        "load the meshwork skill before filing",
    ] {
        assert!(joined.contains(rule), "{rule}:\n{joined}");
    }
    assert!(joined.len() <= 320, "two tight lines: {} bytes", joined.len());
    assert!(out.len() <= 6144, "budget: {} bytes", out.len());
}
