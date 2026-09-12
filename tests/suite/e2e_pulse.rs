// mw-bwwd75h (MW-S6/S7): the pulse block — the repo's `pulse` row rendered
// as weather lines at the top of `prime`, computed in Rust over the tasks
// prime already parsed, and held equal to `SELECT * FROM pulse` for the
// same store and clock. The SQL is the specification; a difference is a
// bug on one side.

const PULSE_STAMP: &str = "2026-08-10T12:00Z";

/// The block's five lines and the pulse columns each one carries — a line
/// whose every count is zero is omitted, so presence is checkable too.
const PULSE_LINES: &[(&str, &[&str])] = &[
    ("- flow ", &["filed_w", "filed_from_w", "done_w", "dropped_w"]),
    (
        "- queue:",
        &["open_n", "doing_n", "blocked_n", "ready_n", "doing_stale", "past_triage"],
    ),
    (
        "- graph:",
        &[
            "lanes_multi",
            "unlockers",
            "needs_behind_n",
            "blocked_foreign",
            "blocked_unresolved",
            "owed_foreign",
        ],
    ),
    (
        "- friction ",
        &["close_attempts_w", "reopens_w", "blocks_w", "thrash_n", "handoff_stale_n"],
    ),
];

/// Columns a single-store session computes; `open_age_med_d` and
/// `backlog_delta_w` ride along as derived values.
fn single_store_columns() -> Vec<&'static str> {
    let mut cols: Vec<&str> = PULSE_LINES.iter().flat_map(|(_, c)| c.iter().copied()).collect();
    cols.extend(["done_n", "dropped_n", "open_age_med_d", "backlog_delta_w"]);
    cols
}

const ASK_COLUMNS: &[&str] = &[
    "asks_in_open",
    "asks_in_oldest_d",
    "asks_out_open",
    "asks_out_oldest_d",
];

fn json_of(assert: &assert_cmd::assert::Assert) -> serde_json::Value {
    serde_json::from_str(&stdout_of(assert)).unwrap()
}

/// `SELECT * FROM pulse` through `verb`, the row for `repo` as column → cell.
fn pulse_view(
    mut cmd: Command,
    verb: &[&str],
    repo: &str,
) -> std::collections::BTreeMap<String, serde_json::Value> {
    let mut args: Vec<&str> = verb.to_vec();
    args.extend(["SELECT * FROM pulse", "--json"]);
    let v = json_of(&cmd.args(&args).assert().success());
    let columns: Vec<String> = v["data"]["columns"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap().to_string())
        .collect();
    let row = v["data"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r[0] == repo)
        .unwrap_or_else(|| panic!("no pulse row for {repo}: {v}"));
    columns
        .into_iter()
        .zip(row.as_array().unwrap().iter().cloned())
        .collect()
}

/// Numbers within rounding noise, everything else exactly.
fn same_cell(a: &serde_json::Value, b: &serde_json::Value) -> bool {
    match (a.as_f64(), b.as_f64()) {
        (Some(x), Some(y)) => (x - y).abs() < 1e-9,
        _ => a == b,
    }
}

fn assert_columns(
    label: &str,
    columns: &[&str],
    pulse: &serde_json::Value,
    view: &std::collections::BTreeMap<String, serde_json::Value>,
) {
    for col in columns {
        let ours = &pulse[*col];
        let theirs = &view[*col];
        assert!(
            same_cell(ours, theirs),
            "{label}.{col}: prime renders {ours} but the view says {theirs}"
        );
    }
}

fn is_zero(v: &serde_json::Value) -> bool {
    v.is_null() || v.as_f64() == Some(0.0)
}

/// MW-S6/S7: every fixture store, column by column; the lines carry the
/// same numbers, sit at the top of weather, and vanish when all-zero.
#[test]
fn prime_pulse_matches_view() {
    for name in ["alpha", "beta", "conformance", "alpha-broken"] {
        let (_g, repo) = fixture_repo(name);
        if name == "alpha-broken" {
            // Two files minting one gid is store damage the SQL answers
            // with doubled rows and a reader with whichever it keeps —
            // lint's finding (graph::graph_rust_matches_view says the same).
            std::fs::remove_file(repo.join("docs/meshwork/ax-dup1-duplicate-id-second.md")).unwrap();
        }
        let prime = json_of(
            &meshwork(&repo)
                .env("MESHWORK_TODAY", PULSE_STAMP)
                .args(["prime", "--json"])
                .assert()
                .success(),
        );
        let pulse = &prime["data"]["pulse"];
        assert!(pulse.is_object(), "{name}: prime --json carries data.pulse: {prime}");
        let mut q = meshwork(&repo);
        q.env("MESHWORK_TODAY", PULSE_STAMP);
        let view = pulse_view(q, &["q"], name);
        assert_columns(name, &single_store_columns(), pulse, &view);

        let text = stdout_of(
            &meshwork(&repo)
                .env("MESHWORK_TODAY", PULSE_STAMP)
                .arg("prime")
                .assert()
                .success(),
        );
        for (prefix, cols) in PULSE_LINES {
            let expected = cols.iter().any(|c| !is_zero(&view[*c]));
            let present = text.lines().any(|l| l.starts_with(prefix));
            assert_eq!(present, expected, "{name}: `{prefix}` line presence:\n{text}");
        }
        if let Some(at) = text.find("weather:\n") {
            let first = text[at + "weather:\n".len()..].lines().next().unwrap_or("");
            assert!(
                PULSE_LINES.iter().any(|(p, _)| first.starts_with(p)),
                "{name}: the pulse leads the weather: {first}\n{text}"
            );
        }
    }

    // The numbers in the text are the view's — conformance, every line live.
    let (_g, repo) = fixture_repo("conformance");
    let mut q = meshwork(&repo);
    q.env("MESHWORK_TODAY", PULSE_STAMP);
    let view = pulse_view(q, &["q"], "conformance");
    let text = stdout_of(
        &meshwork(&repo)
            .env("MESHWORK_TODAY", PULSE_STAMP)
            .arg("prime")
            .assert()
            .success(),
    );
    assert!(
        text.contains(&format!("ready {} of {} open", view["ready_n"], view["open_n"])),
        "{text}"
    );
    assert!(
        text.contains(&format!("filed {} ({} from other tasks)", view["filed_w"], view["filed_from_w"])),
        "{text}"
    );
    assert!(text.contains("open age med 5.1d"), "one decimal, like the view: {text}");
}

/// The asks line rides the union read (MW-S6): each half equals the
/// portfolio-scope pulse for the repo, by the view's rule — a done answer
/// answers; an open one is an intent and the ask stays owed.
#[test]
fn prime_pulse_asks_match_union() {
    let dir = tempfile::tempdir().unwrap();
    let linked = fixtures_root().join("conformance/linked");
    for name in ["left", "right"] {
        let repo = dir.path().join(name);
        copy_dir(&linked.join(name), &repo);
        git(&repo, &["init", "-q"]);
    }
    let portfolio = dir.path().join("portfolio");
    copy_dir(&linked.join("portfolio"), &portfolio);
    std::fs::write(
        portfolio.join("repos.local.toml"),
        format!(
            "[paths]\nleft = \"{}\"\nright = \"{}\"\n",
            dir.path().join("left").display(),
            dir.path().join("right").display()
        ),
    )
    .unwrap();
    let at = |name: &str| {
        let mut c = meshwork(&dir.path().join(name));
        c.env("MESHWORK_PORTFOLIO", &portfolio);
        c.env("MESHWORK_TODAY", PULSE_STAMP);
        c
    };

    for name in ["left", "right"] {
        let prime = json_of(&at(name).args(["prime", "--json"]).assert().success());
        let pulse = &prime["data"]["pulse"];
        let union = pulse_view(at(name), &["portfolio", "q"], name);
        assert_columns(&format!("{name} (union)"), ASK_COLUMNS, pulse, &union);
        let local = pulse_view(at(name), &["q"], name);
        assert_columns(name, &single_store_columns(), pulse, &local);
    }
    let left = stdout_of(&at("left").arg("prime").assert().success());
    assert!(left.contains("owed by me 2 (oldest 9.1d)"), "{left}");
    assert!(!left.contains("owed to me"), "nothing owed to left: {left}");
    let right = stdout_of(&at("right").arg("prime").assert().success());
    assert!(right.contains("owed to me 2 (oldest 9.1d)"), "{right}");
}

/// MW-S6: the next block names the closed tasks its handoff still cites —
/// the one place a session is about to trust prose of unknown age.
#[test]
fn prime_next_cites_closed() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let shipped = add_task(&repo, "Shipped already");
    meshwork(&repo)
        .args(["close", &shipped, "--waive", "fixture"])
        .assert()
        .success();
    let next = add_task(&repo, "Up next");
    let voice = format!("Start from {shipped} — it landed the seam; keep going from there.");
    meshwork(&repo)
        .args(["set", &next, "--handoff", &voice])
        .assert()
        .success();

    let out = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    assert!(
        out.contains(&format!("cites 1 closed task ({shipped})")),
        "{out}"
    );
    // The JSON names the mention's gid — a foreign closed task stays distinguishable.
    let v = json_of(&meshwork(&repo).args(["prime", "--json"]).assert().success());
    assert_eq!(
        v["data"]["next"]["cites"],
        serde_json::json!([format!("work#{shipped}")]),
        "{v}"
    );

    // A handoff citing only live work says nothing.
    let live = add_task(&repo, "Still open");
    meshwork(&repo)
        .args(["set", &next, "--handoff", &format!("Pair with {live}.")])
        .assert()
        .success();
    let out = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    assert!(!out.contains("cites"), "{out}");
}
