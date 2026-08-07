// mw-xvtf5jx: a comment's spec-level identity is
// SHA-256(date NUL author NUL text), lowercase hex — the formula the
// mirror's idempotency markers abbreviate (first 8 chars, DESIGN §8) and
// any UI/replication layer dedups by. Exposed as the `hash` column on the
// comments table. Minute stamps (mw-zp1h12d) are what make it trustworthy:
// same-day identical text no longer collides.

/// The formula is pinned byte-for-byte (a silent change would desync
/// every consumer): identical tuples hash identically, one minute of
/// stamp difference re-keys.
#[test]
fn comment_identity() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "commented");
    let comment = |stamp: &str| {
        meshwork(&repo)
            .env("MESHWORK_TODAY", stamp)
            .args(["comment", &id, "--as", "maya", "hello"])
            .assert()
            .success();
    };
    comment("2026-08-05T01:00Z");
    comment("2026-08-05T01:01Z"); // same author+text, one minute later
    comment("2026-08-05T01:00Z"); // the exact first tuple again

    let js = stdout_of(
        &meshwork(&repo)
            .args(["q", "SELECT ord, hash FROM comments ORDER BY ord", "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    let hashes: Vec<&str> = v["data"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r[1].as_str().unwrap())
        .collect();
    assert_eq!(hashes.len(), 3);

    // Pinned: sha256("2026-08-05T01:00Z\0maya\0hello").
    assert_eq!(
        hashes[0],
        "4fe39928c01770e4cd0579c1580ac093769339536bcf95ab0084e9f899b46a2f",
        "the formula is frozen — did the tuple encoding change?"
    );
    assert_eq!(hashes[0], hashes[2], "identity IS the tuple");
    assert_ne!(hashes[0], hashes[1], "minute stamps re-key (mw-zp1h12d)");
    assert!(
        hashes.iter().all(|h| h.len() == 64
            && h.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())),
        "lowercase hex sha256: {hashes:?}"
    );
}
