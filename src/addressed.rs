//! Addressed tasks (mw-hfvtx0s): the read-time join. An ask is a task in
//! its author's store carrying `to: <repo>` (or `to: repo#id`); the
//! addressee's prime/ready surface it by scanning the portfolio union at
//! read time. It stops surfacing when any non-dropped task in the union
//! `answers:` its gid, or when the ask itself goes terminal. No broker, no
//! transport, no write into another repo's store — the merge stays the
//! trust unit (mw-egksvhm), and delivery is derived, never pushed.
//!
//! This is the one sanctioned full-union read inside a single-repo verb
//! (owner lane 2026-08-17): dep resolution keeps its direct-file-lookup
//! discipline (mw-k7r5), but incoming asks are unknowable in advance — a
//! scan is the mechanism, and the union is ~30ms cold (MW-C4 §portfolio).
//! No registry (or an unreadable one) means no join: quiet empty, exit 0.

use crate::parse::{ParsedTask, Status};

/// One incoming ask, ready to render.
pub struct Ask {
    /// The ask's home identity — `repo#id` in its author's store.
    pub gid: String,
    /// The ask's title, verbatim.
    pub title: String,
}

/// The asks addressed to `me` that no live task answers, oldest first.
/// Best-effort by design: absent repos simply contribute nothing.
#[must_use]
pub fn inbox(me: &str) -> Vec<Ask> {
    let Ok(Some(registry)) = crate::registry::quiet_load() else {
        return Vec::new();
    };
    let Ok((stores, _skipped)) = crate::registry::load_stores(&registry) else {
        return Vec::new();
    };

    // Every ask gid answered by a non-dropped task, anywhere in the union.
    let answered: std::collections::BTreeSet<String> = stores
        .iter()
        .flat_map(|s| {
            s.entries.iter().filter_map(|e| match &e.parsed {
                ParsedTask::Valid(t) if t.status != Status::Dropped => t
                    .answers
                    .as_deref()
                    .map(|a| crate::tables::qualify_ref(s, a)),
                _ => None,
            })
        })
        .collect();

    let mut asks: Vec<(String, Ask)> = Vec::new();
    for store in &stores {
        for entry in &store.entries {
            let ParsedTask::Valid(t) = &entry.parsed else {
                continue;
            };
            // Addressee = the repo segment of `to:` (`repo` or `repo#id`).
            let addressee = t.to.as_deref().map(|v| v.split('#').next().unwrap_or(v));
            if addressee != Some(me) {
                continue;
            }
            if matches!(t.status, Status::Done | Status::Dropped) {
                continue; // a terminal ask is not incoming work
            }
            let gid = store.gid(&t.id);
            if answered.contains(&gid) {
                continue;
            }
            asks.push((
                t.created.clone().unwrap_or_default(),
                Ask {
                    gid,
                    title: t.title.clone(),
                },
            ));
        }
    }
    asks.sort_by(|a, b| (&a.0, &a.1.gid).cmp(&(&b.0, &b.1.gid)));
    asks.into_iter().map(|(_, a)| a).collect()
}
