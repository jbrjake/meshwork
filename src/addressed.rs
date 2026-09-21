//! Addressed tasks (mw-hfvtx0s): the read-time join. An ask is a task in
//! its author's store carrying `to: <repo>` (or `to: repo#id`); the
//! addressee's prime/ready surface it by scanning the portfolio union at
//! read time. It stops surfacing when a DONE task in the union `answers:`
//! its gid, or when the ask itself goes terminal; while the answering task
//! is merely live the ask stays owed and carries `answered-by <gid>
//! (<status>)` — an intent to answer is not an answer (MW-L2). On the
//! sending side a `to:` task is owed by someone else, not worked here: it
//! leaves its own repo's worklist for an `asks out` line (MW-L5). No
//! broker, no transport, no write into another repo's store — the merge
//! stays the trust unit (mw-egksvhm), and delivery is derived, never pushed.
//!
//! This is the one sanctioned full-union read inside a single-repo verb
//! (owner lane 2026-08-17): dep resolution keeps its direct-file-lookup
//! discipline (mw-k7r5), but incoming asks are unknowable in advance — a
//! scan is the mechanism, and the union is ~30ms cold (MW-C4 §portfolio).
//! No registry (or an unreadable one) means no join: quiet empty, exit 0.

use crate::parse::{ParsedTask, Status};
use crate::store::RepoStore;
use std::collections::BTreeMap;

/// The task that answers an ask — the one the `asks` view names as
/// `answer_gid`: a done answer over a live one, then the smallest gid. A
/// dropped answer is no answer at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    /// `repo#id` of the answering task.
    pub gid: String,
    /// Its current status — `done` retires the ask; anything else is intent.
    pub status: Status,
}

impl Answer {
    /// `answered-by <gid> (<status>)` — the same words on every surface.
    #[must_use]
    pub fn render(&self) -> String {
        format!("answered-by {} ({})", self.gid, self.status.as_str())
    }

    /// True when this answer retires the ask.
    #[must_use]
    pub fn is_done(&self) -> bool {
        self.status == Status::Done
    }
}

/// Every ask gid → its answer, over the loaded stores.
#[must_use]
pub fn answers_of(stores: &[RepoStore]) -> BTreeMap<String, Answer> {
    let mut out: BTreeMap<String, Answer> = BTreeMap::new();
    for store in stores {
        for entry in &store.entries {
            let ParsedTask::Valid(t) = &entry.parsed else {
                continue;
            };
            if t.status == Status::Dropped {
                continue;
            }
            let Some(a) = t.answers.as_deref() else {
                continue;
            };
            let ask = crate::tables::qualify_ref(store, a);
            let cand = Answer {
                gid: store.gid(&t.id),
                status: t.status,
            };
            let replace = match out.get(&ask) {
                None => true,
                Some(cur) => {
                    (cand.is_done() && !cur.is_done())
                        || (cand.is_done() == cur.is_done() && cand.gid < cur.gid)
                }
            };
            if replace {
                out.insert(ask, cand);
            }
        }
    }
    out
}

/// Whole days from `created` to `today`; None when absent or unparseable.
/// Clamped at zero — a future stamp is not a negative wait.
fn age_of(created: Option<&str>, today: &str) -> Option<i64> {
    crate::clock::days_between(created?, today).map(|d| d.max(0))
}

/// ` (Dd)` for a row, or nothing when the age is unknown.
fn age_suffix_of(created: Option<&str>, today: &str) -> String {
    age_of(created, today).map_or(String::new(), |d| format!(" ({d}d)"))
}

/// One incoming ask, ready to render.
pub struct Ask {
    /// The ask's home identity — `repo#id` in its author's store.
    pub gid: String,
    /// The ask's title, verbatim.
    pub title: String,
    /// The ask's `created` stamp, as written; its age derives from this
    /// plus the absence of a done answer (mw-r6g9bhe).
    pub created: Option<String>,
    /// The live task answering it, when one exists — rendered, never a
    /// retirement.
    pub answer: Option<Answer>,
}

impl Ask {
    /// Whole days the ask has waited as of `today`.
    #[must_use]
    pub fn age_days(&self, today: &str) -> Option<i64> {
        age_of(self.created.as_deref(), today)
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

/// The statement that lists the asks addressed to `me`, oldest first —
/// what a digest points at when it cannot spend the bytes to list them
/// itself (the `asks` view over the union).
#[must_use]
pub fn list_statement(me: &str) -> String {
    format!(
        "portfolio q \"SELECT gid, title FROM asks WHERE to_repo = '{me}' AND unanswered \
         ORDER BY age_h DESC\""
    )
}

/// ` (Dd)` for an inbox row, or nothing when the age is unknown.
#[must_use]
pub fn age_suffix(ask: &Ask, today: &str) -> String {
    age_suffix_of(ask.created.as_deref(), today)
}

/// `<sep>answered-by <gid> (<status>)` for a row, or nothing.
#[must_use]
pub fn answered_suffix(answer: Option<&Answer>, sep: &str) -> String {
    answer.map_or(String::new(), |a| format!("{sep}{}", a.render()))
}

/// The portfolio union as the read-time join sees it: every store the
/// registry resolves, or `None` when there is no registry to resolve
/// (quiet by rule — absent repos simply contribute nothing).
#[must_use]
pub fn union() -> Option<Vec<RepoStore>> {
    let registry = crate::registry::quiet_load().ok()??;
    let (stores, _skipped) = crate::registry::load_stores(&registry).ok()?;
    Some(stores)
}

/// The asks addressed to `me` that no done task answers, oldest first.
/// Best-effort by design: absent repos simply contribute nothing.
#[must_use]
pub fn inbox(me: &str) -> Vec<Ask> {
    union().map_or_else(Vec::new, |stores| inbox_of(&stores, me))
}

/// [`inbox`] over an already-loaded union — one read serves every
/// consumer in a digest.
#[must_use]
pub fn inbox_of(stores: &[RepoStore], me: &str) -> Vec<Ask> {
    let answers = answers_of(stores);
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
            let answer = answers.get(&gid).cloned();
            if answer.as_ref().is_some_and(Answer::is_done) {
                continue; // retired: a done task answered it
            }
            asks.push((
                t.created.clone().unwrap_or_default(),
                Ask {
                    gid,
                    title: t.title.clone(),
                    created: t.created.clone(),
                    answer,
                },
            ));
        }
    }
    asks.sort_by(|a, b| (&a.0, &a.1.gid).cmp(&(&b.0, &b.1.gid)));
    asks.into_iter().map(|(_, a)| a).collect()
}

/// One of this store's own asks — owed by its addressee, not worked here.
pub struct Outbound {
    /// The ask's local id.
    pub id: String,
    /// Its title, verbatim.
    pub title: String,
    /// The `to:` value as written (`repo` or `repo#id`).
    pub to: String,
    /// Its `created` stamp, as written.
    pub created: Option<String>,
    /// The task answering it, done or live, when the loaded set holds one.
    pub answer: Option<Answer>,
}

impl Outbound {
    /// Whole days since the ask was filed as of `today`.
    #[must_use]
    pub fn age_days(&self, today: &str) -> Option<i64> {
        age_of(self.created.as_deref(), today)
    }

    /// ` (Dd)` or nothing.
    #[must_use]
    pub fn age_suffix(&self, today: &str) -> String {
        age_suffix_of(self.created.as_deref(), today)
    }
}

/// The store's own non-terminal `to:` tasks, oldest first, each with its
/// answer's state when the union holds one — or the store alone, when
/// there is no registry: the asks are local files and need no join.
#[must_use]
pub fn outbound(store: &RepoStore, union: Option<&[RepoStore]>) -> Vec<Outbound> {
    let answers = match union {
        Some(u) => answers_of(u),
        None => answers_of(std::slice::from_ref(store)),
    };
    let mut out: Vec<Outbound> = store
        .entries
        .iter()
        .filter_map(|e| match &e.parsed {
            ParsedTask::Valid(t) if !matches!(t.status, Status::Done | Status::Dropped) => {
                t.to.as_ref().map(|to| Outbound {
                    id: t.id.clone(),
                    title: t.title.clone(),
                    to: to.clone(),
                    created: t.created.clone(),
                    answer: answers.get(&store.gid(&t.id)).cloned(),
                })
            }
            _ => None,
        })
        .collect();
    out.sort_by(|a, b| {
        (a.created.as_deref().unwrap_or(""), &a.id)
            .cmp(&(b.created.as_deref().unwrap_or(""), &b.id))
    });
    out
}

/// The answer to the ask `gid` as the union sees it — or as the store at
/// `root` sees it when no registry loads. For `show` on an ask.
#[must_use]
pub fn answer_for(root: &std::path::Path, gid: &str) -> Option<Answer> {
    let mut answers = match union() {
        Some(stores) => answers_of(&stores),
        None => answers_of(std::slice::from_ref(&crate::store::load_repo(root).ok()?)),
    };
    answers.remove(gid)
}
