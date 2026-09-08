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

/// An unanswered ask's silence is a number every session sees: prime's
/// headline carries the unanswered count and the oldest age, derived
/// from `created` plus the absence of `answers`; each inbox row carries
/// its own age, in ready and prime alike.
#[test]
fn prime_asks_age_line() {
    let (dir, portfolio) = portfolio_fixture();
    let alpha = dir.path().join("alpha");
    let beta = dir.path().join("beta");
    for (id, created) in [("az-a5k010", "2026-08-17"), ("az-a5k011", "2026-09-01T09:30Z")] {
        std::fs::write(
            alpha.join(format!("docs/meshwork/{id}-ask-{created}.md")),
            format!(
                "---\nid: {id}\ntitle: Ask from {created}\nstatus: open\nto: beta\n\
                 created: {created}\n---\n\n## log\n- {created} created\n"
            ),
        )
        .unwrap();
    }
    let prime = |json: bool| {
        let mut cmd = meshwork(&beta);
        cmd.env("MESHWORK_PORTFOLIO", &portfolio)
            .env("MESHWORK_TODAY", "2026-09-07")
            .arg("prime");
        if json {
            cmd.arg("--json");
        }
        stdout_of(&cmd.assert().success())
    };

    let text = prime(false);
    let headline = text.lines().next().unwrap_or_default();
    assert!(
        headline.contains("2 asks unanswered, oldest 21d"),
        "headline carries the count and the oldest age: {headline}"
    );
    assert!(text.contains("alpha#az-a5k010 Ask from 2026-08-17 (21d)"), "{text}");
    assert!(text.contains("alpha#az-a5k011 Ask from 2026-09-01T09:30Z (6d)"), "{text}");

    let v: serde_json::Value = serde_json::from_str(&prime(true)).unwrap();
    assert_eq!(v["data"]["asks"]["unanswered"], 2, "{v}");
    assert_eq!(v["data"]["asks"]["oldest_days"], 21, "{v}");
    let addressed = v["data"]["addressed"].as_array().unwrap();
    assert_eq!(addressed[0]["gid"], "alpha#az-a5k010");
    assert_eq!(addressed[0]["age_days"], 21);
    assert_eq!(addressed[0]["created"], "2026-08-17");

    // ready carries the age too.
    let ready = stdout_of(
        &meshwork(&beta)
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .env("MESHWORK_TODAY", "2026-09-07")
            .arg("ready")
            .assert()
            .success(),
    );
    assert!(ready.contains("alpha#az-a5k010  Ask from 2026-08-17  (21d)"), "{ready}");

    // No inbox, no asks tail — the headline stays as it was.
    let quiet = stdout_of(&meshwork(&beta).arg("prime").assert().success());
    assert!(!quiet.lines().next().unwrap_or_default().contains("unanswered"), "{quiet}");
}

/// The cheap half of the same failure class: an ask whose `to:` names no
/// registered repo can never surface anywhere. add --batch says so at
/// file time — a warning, not a refusal (the registry may lag the repo).
#[test]
fn batch_warns_unresolvable_to() {
    let (dir, portfolio) = portfolio_fixture();
    let beta = dir.path().join("beta");
    let out = meshwork(&beta)
        .env("MESHWORK_PORTFOLIO", &portfolio)
        .args(["add", "--batch", "-"])
        .write_stdin("---\ntitle: Ask nobody\nto: nowhere\nverify: \"true\"\n---\n")
        .assert()
        .success();
    let err = stderr_of(&out);
    assert!(
        err.contains("to: nowhere") && err.contains("no registered repo"),
        "{err}"
    );
    // A resolvable addressee is quiet.
    let out = meshwork(&beta)
        .env("MESHWORK_PORTFOLIO", &portfolio)
        .args(["add", "--batch", "-"])
        .write_stdin("---\ntitle: Ask alpha\nto: alpha#az-x9b2\nverify: \"true\"\n---\n")
        .assert()
        .success();
    assert!(!stderr_of(&out).contains("no registered repo"), "{}", stderr_of(&out));
}
