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
    /// The ask's `created` stamp, as written; its age derives from this
    /// plus the absence of an answer (mw-r6g9bhe).
    pub created: Option<String>,
}

impl Ask {
    /// Whole days the ask has waited as of `today`; None when `created`
    /// is absent or unparseable. Clamped at zero — a future stamp is not
    /// a negative wait.
    #[must_use]
    pub fn age_days(&self, today: &str) -> Option<i64> {
        let created = self.created.as_deref()?;
        crate::clock::days_between(created, today).map(|d| d.max(0))
    }
}

/// The longest wait in the inbox — the headline's number.
#[must_use]
pub fn oldest_age_days(asks: &[Ask], today: &str) -> Option<i64> {
    asks.iter().filter_map(|a| a.age_days(today)).max()
}

/// `N ask(s) unanswered, oldest Dd` — the headline tail; None when the
/// inbox is empty.
#[must_use]
pub fn headline_tail(asks: &[Ask], today: &str) -> Option<String> {
    if asks.is_empty() {
        return None;
    }
    let s = if asks.len() == 1 { "" } else { "s" };
    let oldest = oldest_age_days(asks, today).map_or(String::new(), |d| format!(", oldest {d}d"));
    Some(format!("{} ask{s} unanswered{oldest}", asks.len()))
}

/// ` (Dd)` for an inbox row, or nothing when the age is unknown.
#[must_use]
pub fn age_suffix(ask: &Ask, today: &str) -> String {
    ask.age_days(today)
        .map_or(String::new(), |d| format!(" ({d}d)"))
}

/// The portfolio union as the read-time join sees it: every store the
/// registry resolves, or `None` when there is no registry to resolve
/// (quiet by rule — absent repos simply contribute nothing).
#[must_use]
pub fn union() -> Option<Vec<crate::store::RepoStore>> {
    let registry = crate::registry::quiet_load().ok()??;
    let (stores, _skipped) = crate::registry::load_stores(&registry).ok()?;
    Some(stores)
}

/// The asks addressed to `me` that no live task answers, oldest first.
/// Best-effort by design: absent repos simply contribute nothing.
#[must_use]
pub fn inbox(me: &str) -> Vec<Ask> {
    union().map_or_else(Vec::new, |stores| inbox_of(&stores, me))
}

/// [`inbox`] over an already-loaded union — one read serves every
/// consumer in a digest.
#[must_use]
pub fn inbox_of(stores: &[crate::store::RepoStore], me: &str) -> Vec<Ask> {
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
    for store in stores {
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
                    created: t.created.clone(),
                },
            ));
        }
    }
    asks.sort_by(|a, b| (&a.0, &a.1.gid).cmp(&(&b.0, &b.1.gid)));
    asks.into_iter().map(|(_, a)| a).collect()
}
