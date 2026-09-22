// MW-L2 / MW-L5 (owner ruling 2026-09-20, R-B items B.2 and B.5): an ask
// stays owed until a DONE task answers it and renders its live answer as
// `answered-by <gid> (<status>)`; a sender's own `to:` tasks leave its
// ready/next for an `asks out` line that carries the same state. Nothing
// here transmits: every surface is a read-time join over the union.

fn write_ask(repo: &Path, id: &str, to: &str, created: &str) {
    std::fs::write(
        repo.join(format!("docs/meshwork/{id}-ask.md")),
        format!(
            "---\nid: {id}\ntitle: Ask {id} of {to}\nstatus: open\nto: {to}\n\
             created: {created}\n---\n\n## log\n- {created} created\n"
        ),
    )
    .unwrap();
}

/// File an open answer to `ask_gid` in `repo` through the batch path;
/// returns the new id.
fn file_answer(repo: &Path, portfolio: &Path, ask_gid: &str) -> String {
    let out = stdout_of(
        &meshwork(repo)
            .env("MESHWORK_PORTFOLIO", portfolio)
            .args(["add", "--batch", "-"])
            .write_stdin(format!(
                "---\ntitle: Answer {ask_gid}\nanswers: {ask_gid}\nverify: \"true\"\n---\n"
            ))
            .assert()
            .success(),
    );
    out.lines()
        .find_map(|l| l.split_whitespace().next())
        .unwrap()
        .to_string()
}

fn at(repo: &Path, portfolio: &Path, args: &[&str]) -> String {
    stdout_of(
        &meshwork(repo)
            .env("MESHWORK_PORTFOLIO", portfolio)
            .env("MESHWORK_TODAY", "2026-09-07")
            .args(args)
            .assert()
            .success(),
    )
}

/// MW-L2: an open answer is an intent, not an answer — the ask stays in
/// the addressee's ready and prime, carrying `answered-by <gid> (<status>)`
/// as the answer moves through open and doing; it leaves only when the
/// answer is done.
#[test]
fn ask_surfaced_until_answer_done() {
    let (dir, portfolio) = portfolio_fixture();
    let alpha = dir.path().join("alpha");
    let beta = dir.path().join("beta");
    write_ask(&alpha, "az-a5k020", "beta", "2026-08-17");

    let ready = at(&beta, &portfolio, &["ready"]);
    assert!(ready.contains("alpha#az-a5k020") && !ready.contains("answered-by"), "{ready}");

    let answer = file_answer(&beta, &portfolio, "alpha#az-a5k020");
    let ready = at(&beta, &portfolio, &["ready"]);
    assert!(
        ready.contains(&format!("alpha#az-a5k020  Ask az-a5k020 of beta  (21d)  answered-by beta#{answer} (open)")),
        "an open answer keeps the ask surfaced, with its state:\n{ready}"
    );
    let prime = at(&beta, &portfolio, &["prime"]);
    assert!(
        prime.contains(&format!("alpha#az-a5k020 Ask az-a5k020 of beta (21d) \u{b7} answered-by beta#{answer} (open)")),
        "{prime}"
    );
    assert!(
        prime.lines().next().unwrap_or_default().contains("1 ask unanswered"),
        "the headline still counts it owed:\n{prime}"
    );
    let v: serde_json::Value = serde_json::from_str(&at(&beta, &portfolio, &["ready", "--json"])).unwrap();
    assert_eq!(v["data"]["addressed"][0]["answered_by"]["gid"], format!("beta#{answer}"));
    assert_eq!(v["data"]["addressed"][0]["answered_by"]["status"], "open");

    at(&beta, &portfolio, &["start", &answer]);
    let ready = at(&beta, &portfolio, &["ready"]);
    assert!(ready.contains(&format!("answered-by beta#{answer} (doing)")), "{ready}");

    at(&beta, &portfolio, &["close", &answer]);
    let ready = at(&beta, &portfolio, &["ready"]);
    assert!(!ready.contains("az-a5k020"), "a done answer retires the ask:\n{ready}");
    let prime = at(&beta, &portfolio, &["prime"]);
    assert!(
        !prime.contains("addressed to this repo") && !prime.contains("unanswered"),
        "{prime}"
    );
}

/// MW-L2: declining to answer leaves the ask owed — a dropped answer
/// re-surfaces it, with no `answered-by`.
#[test]
fn ask_resurfaces_on_dropped_answer() {
    let (dir, portfolio) = portfolio_fixture();
    let alpha = dir.path().join("alpha");
    let beta = dir.path().join("beta");
    write_ask(&alpha, "az-a5k021", "beta", "2026-08-17");
    let answer = file_answer(&beta, &portfolio, "alpha#az-a5k021");
    assert!(at(&beta, &portfolio, &["ready"]).contains("answered-by"));

    at(&beta, &portfolio, &["drop", &answer]);
    let ready = at(&beta, &portfolio, &["ready"]);
    assert!(
        ready.contains("alpha#az-a5k021") && !ready.contains("answered-by"),
        "a dropped answer is no answer:\n{ready}"
    );
    let v: serde_json::Value = serde_json::from_str(&at(&beta, &portfolio, &["ready", "--json"])).unwrap();
    assert!(v["data"]["addressed"][0]["answered_by"].is_null(), "{v}");
}

/// MW-L5: a task carrying `to:` is owed by someone else, not worked here —
/// it leaves its own repo's `ready`, prime's next and also-ready, and the
/// portfolio's ready/next, and lists under `asks out` with its addressee
/// and age. The line needs no registry: the asks are local files.
#[test]
fn outbound_asks_out_of_ready() {
    let (dir, portfolio) = portfolio_fixture();
    let beta = dir.path().join("beta");
    // seq 1 would put it first in ready if asks were still work.
    std::fs::write(
        beta.join("docs/meshwork/bz-a5k030-ask.md"),
        "---\nid: bz-a5k030\ntitle: Ask alpha for the landing\nstatus: open\nto: alpha\n\
         seq: 1\ncreated: 2026-08-30\n---\n\n## log\n- 2026-08-30 created\n",
    )
    .unwrap();

    let ready = at(&beta, &portfolio, &["ready"]);
    let (rows, tail) = ready.split_once("asks out (1):\n").expect(&ready);
    assert!(!rows.contains("bz-a5k030"), "not in the worklist:\n{ready}");
    assert!(
        tail.starts_with("bz-a5k030  \u{2192} alpha  Ask alpha for the landing  (8d)\n"),
        "{ready}"
    );
    let v: serde_json::Value = serde_json::from_str(&at(&beta, &portfolio, &["ready", "--json"])).unwrap();
    assert!(v["data"]["rows"].as_array().unwrap().iter().all(|r| r["id"] != "bz-a5k030"), "{v}");
    assert_eq!(v["data"]["asks_out"][0]["id"], "bz-a5k030");
    assert_eq!(v["data"]["asks_out"][0]["to"], "alpha");
    assert_eq!(v["data"]["asks_out"][0]["age_days"], 8);
    assert!(v["data"]["asks_out"][0]["answered_by"].is_null());

    let prime = at(&beta, &portfolio, &["prime"]);
    let next = prime.lines().find(|l| l.starts_with("next \u{2192}")).unwrap_or_default();
    assert!(!next.contains("bz-a5k030"), "{prime}");
    assert!(
        prime.contains("asks out (1):\n- bz-a5k030 \u{2192} alpha (8d) Ask alpha for the landing"),
        "{prime}"
    );
    let v: serde_json::Value = serde_json::from_str(&at(&beta, &portfolio, &["prime", "--json"])).unwrap();
    assert_eq!(v["data"]["asks_out"][0]["id"], "bz-a5k030");
    assert!(v["data"]["ready"].as_array().unwrap().iter().all(|r| r["id"] != "bz-a5k030"));

    for verb in [&["portfolio", "ready"][..], &["portfolio", "next"][..]] {
        let out = at(dir.path(), &portfolio, verb);
        assert!(!out.contains("bz-a5k030"), "{verb:?}:\n{out}");
    }

    // Hermetic: no registry, same exclusion, same line — minus any answer.
    let plain = stdout_of(&meshwork(&beta).env("MESHWORK_TODAY", "2026-09-07").arg("ready").assert().success());
    let (rows, tail) = plain.split_once("asks out (1):\n").expect(&plain);
    assert!(!rows.contains("bz-a5k030") && tail.starts_with("bz-a5k030  \u{2192} alpha"), "{plain}");
}

/// MW-L5: the `asks out` row and `show` carry the answering task and its
/// status once one exists — the sender sees the answer move without
/// leaving its own store.
#[test]
fn asks_out_line_answered_state() {
    let (dir, portfolio) = portfolio_fixture();
    let alpha = dir.path().join("alpha");
    let beta = dir.path().join("beta");
    write_ask(&beta, "bz-a5k031", "alpha", "2026-08-30");
    let prime = at(&beta, &portfolio, &["prime"]);
    assert!(prime.contains("owed by me 1"), "{prime}");

    let answer = file_answer(&alpha, &portfolio, "beta#bz-a5k031");
    let ready = at(&beta, &portfolio, &["ready"]);
    assert!(
        ready.contains(&format!("bz-a5k031  \u{2192} alpha  Ask bz-a5k031 of alpha  (8d)  answered-by alpha#{answer} (open)")),
        "{ready}"
    );
    let shown = at(&beta, &portfolio, &["show", "bz-a5k031"]);
    assert!(
        shown.contains(&format!("ask: \u{2192} alpha (8d) \u{b7} answered-by alpha#{answer} (open)")),
        "{shown}"
    );
    let v: serde_json::Value = serde_json::from_str(&at(&beta, &portfolio, &["show", "bz-a5k031", "--json"])).unwrap();
    assert_eq!(v["data"]["answered_by"]["gid"], format!("alpha#{answer}"));
    assert_eq!(v["data"]["answered_by"]["status"], "open");

    at(&alpha, &portfolio, &["close", &answer]);
    let ready = at(&beta, &portfolio, &["ready"]);
    assert!(ready.contains(&format!("answered-by alpha#{answer} (done)")), "{ready}");
    let prime = at(&beta, &portfolio, &["prime"]);
    assert!(!prime.contains("owed by me"), "a done answer is no longer owed:\n{prime}");
    assert!(prime.contains(&format!("answered-by alpha#{answer} (done)")), "{prime}");
}

/// MW-M2: `asks` is the inbox in full — every inbound ask past `ready`'s
/// cap, every outbound one, each in `ready`'s spelling with its age and
/// its answer's state — and the verb the capped surfaces name. The
/// did-you-mean for `inbox`/`addressed` points at it.
#[test]
fn asks_verb_in_and_out() {
    let (dir, portfolio) = portfolio_fixture();
    let alpha = dir.path().join("alpha");
    let beta = dir.path().join("beta");
    let gids = write_asks(&alpha, 7, "2026-08-", 0);
    write_ask(&beta, "bz-a5k040", "alpha", "2026-08-30");
    let answer = file_answer(&alpha, &portfolio, "beta#bz-a5k040");

    let out = at(&beta, &portfolio, &["asks"]);
    assert!(out.starts_with("asks in (7):\n"), "{out}");
    for gid in &gids {
        assert!(out.contains(gid), "{gid} uncapped:\n{out}");
    }
    assert!(
        out.contains(&format!(
            "asks out (1):\n  bz-a5k040  \u{2192} alpha  Ask bz-a5k040 of alpha  (8d)  \
             answered-by alpha#{answer} (open)\n"
        )),
        "{out}"
    );
    let ready = at(&beta, &portfolio, &["ready"]);
    assert!(ready.contains("… and 2 more addressed (--all, or meshwork asks)"), "{ready}");

    let v: serde_json::Value = serde_json::from_str(&at(&beta, &portfolio, &["asks", "--json"])).unwrap();
    assert_eq!(v["verb"], "asks");
    assert_eq!(v["data"]["registry"], true);
    assert_eq!(v["data"]["in"].as_array().map(Vec::len), Some(7), "{v}");
    assert_eq!(v["data"]["in"][0]["gid"], gids[0], "oldest first: {v}");
    assert!(v["data"]["in"][0]["age_days"].as_i64().unwrap() > 0, "{v}");
    assert_eq!(v["data"]["out"][0]["id"], "bz-a5k040");
    assert_eq!(v["data"]["out"][0]["to"], "alpha");
    assert_eq!(v["data"]["out"][0]["age_days"], 8);
    assert_eq!(v["data"]["out"][0]["answered_by"]["gid"], format!("alpha#{answer}"));
    assert_eq!(v["data"]["out"][0]["answered_by"]["status"], "open");

    // A done answer retires the inbound ask and settles the outbound one.
    at(&alpha, &portfolio, &["close", &answer]);
    let out = at(&beta, &portfolio, &["asks"]);
    assert!(out.contains(&format!("answered-by alpha#{answer} (done)")), "{out}");

    // Without a registry the inbox is empty and says why; the outbound
    // side needs no join.
    let alone = stdout_of(
        &meshwork(&beta)
            .env("HOME", dir.path())
            .env_remove("MESHWORK_PORTFOLIO")
            .env("MESHWORK_TODAY", "2026-09-07")
            .arg("asks")
            .assert()
            .success(),
    );
    assert!(alone.contains("asks in (0):\n  (none \u{2014} no registry"), "{alone}");
    assert!(alone.contains("asks out (1):\n  bz-a5k040"), "{alone}");

    let err = stderr_of(&meshwork(&beta).arg("inbox").assert().failure());
    assert!(err.contains("did you mean `asks`"), "{err}");
}
