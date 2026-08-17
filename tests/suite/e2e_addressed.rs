// mw-hfvtx0s: addressed tasks — `to:` / `answers:` as a read-time join
// across the portfolio union, no broker. An ask lives in the SENDING
// repo's store; the addressee's prime/ready surface it by scanning the
// union; it drops out when a non-dropped task anywhere answers it. No
// transport, no write into the other repo's store, no CLI change.

/// The conformance scenario the task names: a `to:`-addressed task
/// appears in the addressee's prime/ready view and drops out when
/// answered; a dropped answer un-answers.
#[test]
fn addressed_task_surfaces_and_drops_when_answered() {
    let (dir, portfolio) = portfolio_fixture();
    let alpha = dir.path().join("alpha");
    let beta = dir.path().join("beta");

    // The ask: a task in alpha addressed to beta, written as a plain
    // task file — the format is the API (FORMAT.md), no authoring verb.
    std::fs::write(
        alpha.join("docs/meshwork/az-a5k001-answer-the-landing-question.md"),
        "---\nid: az-a5k001\ntitle: Answer the landing question\nstatus: open\n\
         to: beta\ncreated: 2026-08-17\n---\n\n## log\n- 2026-08-17 created\n",
    )
    .unwrap();

    let ready_beta = || {
        stdout_of(
            &meshwork(&beta)
                .env("MESHWORK_PORTFOLIO", &portfolio)
                .arg("ready")
                .assert()
                .success(),
        )
    };

    // Surfaces in the addressee's ready, labeled, with its home gid.
    let out = ready_beta();
    assert!(out.contains("addressed to this repo"), "{out}");
    assert!(out.contains("alpha#az-a5k001"), "{out}");
    assert!(out.contains("Answer the landing question"), "{out}");

    // …and in prime.
    let prime = stdout_of(
        &meshwork(&beta)
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .arg("prime")
            .assert()
            .success(),
    );
    assert!(prime.contains("alpha#az-a5k001"), "{prime}");

    // Hermetic (no registry): the join never happens, nothing leaks.
    let plain = stdout_of(&meshwork(&beta).arg("ready").assert().success());
    assert!(!plain.contains("az-a5k001"), "{plain}");

    // The answer: a beta task carrying `answers:` — authored through
    // add --batch to prove the batch path accepts the key.
    let batch = stdout_of(
        &meshwork(&beta)
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .args(["add", "--batch", "-"])
            .write_stdin(
                "---\ntitle: Land the answer\nanswers: alpha#az-a5k001\nverify: \"true\"\n---\n",
            )
            .assert()
            .success(),
    );
    let answer_id = batch
        .lines()
        .find_map(|l| l.split_whitespace().next())
        .unwrap()
        .to_string();

    // Answered → the ask drops out of the addressee's view.
    let out = ready_beta();
    assert!(!out.contains("az-a5k001"), "answered ask must drop: {out}");

    // A dropped answer un-answers — the ask resurfaces.
    meshwork(&beta)
        .env("MESHWORK_PORTFOLIO", &portfolio)
        .args(["drop", &answer_id])
        .assert()
        .success();
    let out = ready_beta();
    assert!(out.contains("alpha#az-a5k001"), "dropped answer un-answers: {out}");
}

/// The projection contract: `to:` lands in the `addressed_to` column,
/// `answers:` becomes an `answers` edge — both SQL-visible (DESIGN §4).
#[test]
fn addressed_keys_are_sql_visible() {
    let (dir, portfolio) = portfolio_fixture();
    let beta = dir.path().join("beta");

    std::fs::write(
        beta.join("docs/meshwork/bz-a5k002-ask-alpha-something.md"),
        "---\nid: bz-a5k002\ntitle: Ask alpha something\nstatus: open\n\
         to: alpha\ncreated: 2026-08-17\n---\n\n## log\n- 2026-08-17 created\n",
    )
    .unwrap();
    std::fs::write(
        beta.join("docs/meshwork/bz-a5k003-answer-an-alpha-ask.md"),
        "---\nid: bz-a5k003\ntitle: Answer an alpha ask\nstatus: open\n\
         answers: alpha#az-n33d\ncreated: 2026-08-17\n---\n\n## log\n- 2026-08-17 created\n",
    )
    .unwrap();

    let q = |sql: &str| {
        stdout_of(
            &meshwork(&beta)
                .env("MESHWORK_PORTFOLIO", &portfolio)
                .args(["q", sql])
                .assert()
                .success(),
        )
    };
    assert!(
        q("SELECT addressed_to FROM tasks WHERE id='bz-a5k002'").contains("alpha"),
        "to: projects into addressed_to"
    );
    assert!(
        q("SELECT dst_gid FROM edges WHERE kind='answers' AND src_gid='beta#bz-a5k003'")
            .contains("alpha#az-n33d"),
        "answers: projects as an answers edge"
    );

    // show renders both keys.
    let shown = stdout_of(
        &meshwork(&beta)
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .args(["show", "bz-a5k003"])
            .assert()
            .success(),
    );
    assert!(shown.contains("answers: alpha#az-n33d"), "{shown}");

    // An answers edge never gates ready — it is not a dep (MW-B6 predicate
    // filters kind='needs' only): the answering task stays actionable.
    let ready = stdout_of(
        &meshwork(&beta)
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .arg("ready")
            .assert()
            .success(),
    );
    assert!(ready.contains("bz-a5k003"), "{ready}");
}

/// A `to:` ask that is closed (or dropped) on the asking side stops
/// surfacing at the addressee — terminal asks are not incoming work.
#[test]
fn addressed_terminal_ask_stops_surfacing() {
    let (dir, portfolio) = portfolio_fixture();
    let alpha = dir.path().join("alpha");
    let beta = dir.path().join("beta");

    std::fs::write(
        alpha.join("docs/meshwork/az-a5k004-stale-ask.md"),
        "---\nid: az-a5k004\ntitle: Stale ask\nstatus: done\n\
         to: beta\ncreated: 2026-08-17\n---\n\n## log\n- 2026-08-17 created\n",
    )
    .unwrap();

    let out = stdout_of(
        &meshwork(&beta)
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .arg("ready")
            .assert()
            .success(),
    );
    assert!(!out.contains("az-a5k004"), "terminal ask must not surface: {out}");
}
