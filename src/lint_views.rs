//! Lint findings the derived projection makes cheap (MW-S9, mw-axhze9m;
//! MW-R6b, mw-073zekp): each one query over the views the session
//! registers — `mentions`, `lineage`, `facts`, and `graph` for its
//! placement helpers — warnings all, report only. Lint has no perf gate,
//! so it reads the views themselves rather than keeping Rust twins; the
//! views are the specification and this file only asks them questions.

use crate::lint::{finding, Finding, Severity};
use crate::store::RepoStore;
use crate::views::query_blocking;
use datafusion::prelude::SessionContext;
use std::collections::BTreeMap;

/// The spawn-chain bound of the `lineage` view: a depth that reaches it
/// is a `discovered-from` cycle, never a real chain.
const SPAWN_BOUND: i64 = 64;

/// Failed closes on one live task before lint says so.
const CLOSE_ATTEMPTS: i64 = 2;

/// The `place` of a task with no `seq`, as the `graph` view spells it.
const UNRANKED: &str = "999999";

/// A gid as the store writes ids: its own `repo#` stripped.
type Bare<'a> = &'a dyn Fn(&str) -> String;

/// The six view-backed findings over the store. A projection that cannot
/// be read is itself one warning, never silence.
pub(crate) fn check(store: &RepoStore, out: &mut Vec<Finding>) {
    match query(store) {
        Ok(found) => out.extend(found),
        Err(why) => out.push(finding(
            Severity::Warning,
            "views-unavailable",
            "store",
            format!(
                "the derived projection could not be read ({why}) — view-backed findings skipped"
            ),
        )),
    }
}

fn query(store: &RepoStore) -> Result<Vec<Finding>, String> {
    let ctx =
        crate::tables::session_for(std::slice::from_ref(store), &[]).map_err(|e| e.to_string())?;
    let clock = crate::views::Clock::resolve(store.config.window_days())?;
    crate::views::register_blocking(&ctx, &clock, "mentions lineage facts graph")?;
    let prefix = format!("{}#", store.repo);
    let bare = |gid: &str| gid.strip_prefix(&prefix).unwrap_or(gid).to_string();
    let mut out = Vec::new();
    out.extend(handoff_cites_closed(&ctx, &bare)?);
    out.extend(implicit_edges(&ctx, &bare)?);
    out.extend(discovered_cycles(&ctx, &bare)?);
    out.extend(seq_collisions(&ctx, &bare)?);
    out.extend(close_attempts(&ctx, &bare)?);
    out.extend(needs_behind(&ctx, &bare)?);
    Ok(out)
}

/// A prerequisite placed later than live work that transitively needs
/// it — the `graph` view's `needs_behind`, spelled out with the dependent
/// that puts it there. One finding per prerequisite, naming its
/// best-placed dependent; the helpers behind `graph` hold the pair, the
/// view only the verdict. Same-repo only, as `inherit` is.
fn needs_behind(ctx: &SessionContext, bare: Bare) -> Result<Vec<Finding>, String> {
    let rows = query_blocking(
        ctx,
        "SELECT p.gid, p.place, l.gid, l.place FROM g_up u \
         JOIN g_live p ON p.gid = u.pre \
         JOIN g_live l ON l.gid = u.dep AND l.repo = p.repo \
         WHERE l.place < p.place ORDER BY p.gid, l.place, l.gid",
    )?;
    let mut by_pre: BTreeMap<String, (String, String, String)> = BTreeMap::new();
    for r in &rows {
        by_pre
            .entry(bare(&r[0]))
            .or_insert_with(|| (r[1].clone(), bare(&r[2]), r[3].clone()));
    }
    Ok(by_pre
        .into_iter()
        .map(|(pre, (place, dep, dep_place))| {
            let placed = if place == UNRANKED {
                "unranked".to_string()
            } else {
                format!("at seq {place}")
            };
            finding(
                Severity::Warning,
                "needs-behind",
                &pre,
                format!(
                    "{placed}, behind {dep} at seq {dep_place} which needs it — report only; \
                     the fix is a rank for this one or a park for that one, and only the owner \
                     knows which"
                ),
            )
        })
        .collect())
}

/// Prose about to be trusted past its date: a live task's handoff naming
/// a closed one.
fn handoff_cites_closed(ctx: &SessionContext, bare: Bare) -> Result<Vec<Finding>, String> {
    let rows = query_blocking(
        ctx,
        "SELECT m.src_gid, m.ref_gid FROM mentions m JOIN facts f ON f.gid = m.src_gid \
         WHERE m.field = 'handoff' AND m.ref_status IN ('done','dropped') AND f.live \
         ORDER BY m.src_gid, m.ref_gid",
    )?;
    let mut by_src: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for r in &rows {
        by_src.entry(bare(&r[0])).or_default().push(bare(&r[1]));
    }
    Ok(by_src
        .into_iter()
        .map(|(src, refs)| {
            finding(
                Severity::Warning,
                "handoff-cites-closed",
                &src,
                format!(
                    "handoff names closed {} — the voice belongs to what is up next; \
                     refresh or drop it",
                    refs.join(", ")
                ),
            )
        })
        .collect())
}

/// A live task naming live work with no edge behind the mention.
fn implicit_edges(ctx: &SessionContext, bare: Bare) -> Result<Vec<Finding>, String> {
    let rows = query_blocking(
        ctx,
        "SELECT m.src_gid, m.ref_gid, m.field FROM mentions m \
         JOIN facts a ON a.gid = m.src_gid JOIN facts b ON b.gid = m.ref_gid \
         WHERE NOT m.edge_backed AND a.live AND b.live ORDER BY m.src_gid, m.ref_gid, m.field",
    )?;
    let mut by_pair: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for r in &rows {
        by_pair
            .entry((bare(&r[0]), bare(&r[1])))
            .or_default()
            .push(r[2].clone());
    }
    Ok(by_pair
        .into_iter()
        .map(|((src, dst), fields)| {
            finding(
                Severity::Warning,
                "implicit-edge",
                &src,
                format!(
                    "names {dst} in {} with no edge between them — `dep add {src} --needs {dst}` \
                     (or relates:) if it is one; report only",
                    fields.join(", ")
                ),
            )
        })
        .collect())
}

/// The spawn walk hit its bound: a `discovered-from` cycle.
fn discovered_cycles(ctx: &SessionContext, bare: Bare) -> Result<Vec<Finding>, String> {
    Ok(query_blocking(
        ctx,
        &format!("SELECT gid FROM lineage WHERE spawn_depth >= {SPAWN_BOUND} ORDER BY gid"),
    )?
    .iter()
    .map(|r| {
        finding(
            Severity::Warning,
            "discovered-cycle",
            &bare(&r[0]),
            "discovered-from chain from here never ends — a cycle; break it".to_string(),
        )
    })
    .collect())
}

/// Two live tasks on one seq: an order nobody chose.
fn seq_collisions(ctx: &SessionContext, bare: Bare) -> Result<Vec<Finding>, String> {
    let rows = query_blocking(
        ctx,
        "SELECT seq, gid FROM facts WHERE live AND seq IS NOT NULL ORDER BY seq, gid",
    )?;
    let mut by_seq: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for r in &rows {
        by_seq.entry(r[0].clone()).or_default().push(bare(&r[1]));
    }
    Ok(by_seq
        .into_iter()
        .filter(|(_, ids)| ids.len() > 1)
        .map(|(seq, ids)| {
            finding(
                Severity::Warning,
                "seq-collision",
                &ids[0],
                format!(
                    "seq {seq} is shared with {} — two live tasks cannot both come first; \
                     renumber one",
                    ids[1..].join(", ")
                ),
            )
        })
        .collect())
}

/// Repeated failed closes on live work.
fn close_attempts(ctx: &SessionContext, bare: Bare) -> Result<Vec<Finding>, String> {
    Ok(query_blocking(
        ctx,
        &format!(
            "SELECT gid, close_attempts FROM facts WHERE live AND close_attempts >= \
             {CLOSE_ATTEMPTS} ORDER BY gid"
        ),
    )?
    .iter()
    .map(|r| {
        finding(
            Severity::Warning,
            "close-attempts",
            &bare(&r[0]),
            format!(
                "{} failed closes — the verify or the work is wrong; look before the next attempt",
                r[1]
            ),
        )
    })
    .collect())
}
