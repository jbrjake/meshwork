//! The `graph` view computed in Rust (FORMAT.md §Views, `graph`): one row
//! per valid task with its structure over live `needs` — what it unlocks,
//! the place it inherits from its dependents, its lane. The gated verbs
//! (`why` today; `ready` and `prime` once bands order by `unlock`) cannot
//! pay to plan fifty view bodies, so they compute the same relation here
//! from the loaded stores. The published SQL stays the specification:
//! `graph::graph_rust_matches_view` holds the two equal, column by
//! column, on every fixture store — a difference is a bug on one side.
//!
//! Inputs mirror the SQL session's exactly (`tables::session_for`): the
//! loaded stores, plus the registry-resolved foreign thin rows the caller
//! injects into `tasks`. A store minting one gid twice is damage the SQL
//! answers with multiplied joins and this reader with whichever file it
//! keeps — `lint`'s finding, not the view's.

use crate::parse::ParsedTask;
use crate::registry::ForeignTask;
use crate::store::RepoStore;
use crate::tables::qualify_ref;
use std::collections::{BTreeMap, BTreeSet};

/// `place` of a task with no `seq`: the floor every unranked task shares.
pub const UNRANKED: i64 = 999_999;

/// The closure bound: a chain is walked at most this many steps, as the
/// published SQL does — a cycle (lint-fenced, never impossible) would
/// otherwise never terminate.
const DEPTH_BOUND: i64 = 64;

/// One row of the `graph` view, fields in the view's column order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphRow {
    /// `repo#id`.
    pub gid: String,
    /// The store's registry name.
    pub repo: String,
    /// Bare task id.
    pub id: String,
    /// Status as the `tasks` table carries it.
    pub status: String,
    /// The per-repo order weight, as written.
    pub seq: Option<i64>,
    /// Live tasks that transitively need this one.
    pub unlock: i64,
    /// The longest live `needs` chain below it, in edges.
    pub depth: i64,
    /// `seq`, or [`UNRANKED`].
    pub place: i64,
    /// The smallest `place` among live same-repo transitive dependents,
    /// else its own.
    pub inherit: i64,
    /// `inherit < place`: a prerequisite placed later than work waiting
    /// on it.
    pub needs_behind: bool,
    /// The connected component over live `needs` edges, labelled by its
    /// smallest gid; `None` for a task on no live edge.
    pub lane: Option<String>,
    /// Members of the lane; 1 off the live graph.
    pub lane_size: i64,
    /// Unmet `needs`: target missing from the loaded set or non-terminal.
    pub needs_open: i64,
    /// Targets absent from the loaded set.
    pub needs_unresolved: i64,
    /// Unmet targets in another repo.
    pub needs_open_foreign: i64,
    /// Live tasks needing this one.
    pub needed_by_open: i64,
    /// Those in another repo.
    pub needed_by_foreign: i64,
    /// Children (`parent` edges in) that are open, doing or blocked.
    pub live_children: i64,
    /// Tasks carrying `discovered-from` this one.
    pub discovered: i64,
    /// The ready predicate: open, no unmet needs, no live children.
    pub ready: bool,
}

/// A `tasks` row as the view sees it.
struct Node {
    repo: String,
    id: String,
    status: String,
    seq: Option<i64>,
}

enum Kind {
    Needs,
    Parent,
    DiscoveredFrom,
}

struct Edge {
    src: String,
    dst: String,
    kind: Kind,
}

fn live(status: &str) -> bool {
    matches!(status, "open" | "doing" | "blocked")
}

fn terminal(status: &str) -> bool {
    matches!(status, "done" | "dropped")
}

/// Project the stores and foreign rows the way `tables` does: every entry
/// a `tasks` row (invalid files under status `invalid`), the three edge
/// kinds the view reads from valid tasks, targets qualified to gids.
fn project(stores: &[RepoStore], foreign: &[ForeignTask]) -> (BTreeMap<String, Node>, Vec<Edge>) {
    let mut tasks = BTreeMap::new();
    let mut edges = Vec::new();
    for store in stores {
        for entry in &store.entries {
            match &entry.parsed {
                ParsedTask::Valid(t) => {
                    let gid = store.gid(&t.id);
                    let mut push = |target: &str, kind: Kind| {
                        edges.push(Edge {
                            src: gid.clone(),
                            dst: qualify_ref(store, target),
                            kind,
                        });
                    };
                    for n in &t.needs {
                        push(n, Kind::Needs);
                    }
                    if let Some(p) = &t.parent {
                        push(p, Kind::Parent);
                    }
                    if let Some(d) = &t.discovered_from {
                        push(d, Kind::DiscoveredFrom);
                    }
                    tasks.insert(
                        gid,
                        Node {
                            repo: store.repo.clone(),
                            id: t.id.clone(),
                            status: t.status.as_str().to_string(),
                            seq: t.seq,
                        },
                    );
                }
                ParsedTask::Invalid(inv) => {
                    tasks.insert(
                        store.gid(&inv.id),
                        Node {
                            repo: store.repo.clone(),
                            id: inv.id.clone(),
                            status: "invalid".to_string(),
                            seq: None,
                        },
                    );
                }
            }
        }
    }
    for f in foreign {
        tasks.insert(
            f.gid.clone(),
            Node {
                repo: f.repo.clone(),
                id: f.id.clone(),
                status: f.status.clone(),
                seq: None,
            },
        );
    }
    (tasks, edges)
}

/// Per-prerequisite closure over live `needs`: `(unlock, depth, inherit)`
/// for every task something live needs. Level by level, as the recursive
/// SQL walks — each round the dependents of the last, bounded — so a
/// cycle reports the bound, and a diamond counts its far node once.
fn closure<'a>(
    tasks: &'a BTreeMap<String, Node>,
    dependents: &BTreeMap<&'a str, BTreeSet<&'a str>>,
) -> BTreeMap<&'a str, (i64, i64, Option<i64>)> {
    let place = |gid: &str| tasks[gid].seq.unwrap_or(UNRANKED);
    let mut out = BTreeMap::new();
    for (pre, direct) in dependents {
        let repo = &tasks[*pre].repo;
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        let mut frontier: BTreeSet<&str> = direct.clone();
        let mut depth = 0;
        while !frontier.is_empty() && depth < DEPTH_BOUND {
            depth += 1;
            seen.extend(frontier.iter().copied());
            frontier = frontier
                .iter()
                .filter_map(|n| dependents.get(n))
                .flat_map(|s| s.iter().copied())
                .collect();
        }
        let inherit = seen
            .iter()
            .filter(|g| tasks[**g].repo == *repo)
            .map(|g| place(g))
            .min();
        out.insert(
            *pre,
            (
                i64::try_from(seen.len()).unwrap_or(i64::MAX),
                depth,
                inherit,
            ),
        );
    }
    out
}

/// Connected components of the undirected live-`needs` graph, each
/// labelled by its smallest gid: `gid → (lane, size)`.
fn lanes<'a>(adjacent: &BTreeMap<&'a str, BTreeSet<&'a str>>) -> BTreeMap<&'a str, (&'a str, i64)> {
    let mut out = BTreeMap::new();
    let mut assigned: BTreeSet<&str> = BTreeSet::new();
    for start in adjacent.keys() {
        if assigned.contains(start) {
            continue;
        }
        let mut members: BTreeSet<&str> = BTreeSet::new();
        let mut stack = vec![*start];
        while let Some(n) = stack.pop() {
            if members.insert(n) {
                stack.extend(adjacent[n].iter().copied());
            }
        }
        let label = *members.iter().next().unwrap_or(start);
        let size = i64::try_from(members.len()).unwrap_or(i64::MAX);
        for m in &members {
            out.insert(*m, (label, size));
        }
        assigned.extend(members);
    }
    out
}

/// The view over these stores and foreign rows, one row per valid task of
/// a loaded store, sorted by gid.
#[must_use]
pub fn compute(stores: &[RepoStore], foreign: &[ForeignTask]) -> Vec<GraphRow> {
    let (tasks, edges) = project(stores, foreign);
    let repos: BTreeSet<&str> = stores.iter().map(|s| s.repo.as_str()).collect();
    let is_live = |gid: &str| tasks.get(gid).is_some_and(|n| live(&n.status));

    // The live graph: `needs` edges with both ends live.
    let mut dependents: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut adjacent: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for e in &edges {
        if matches!(e.kind, Kind::Needs) && is_live(&e.src) && is_live(&e.dst) {
            dependents.entry(&e.dst).or_default().insert(&e.src);
            adjacent.entry(&e.src).or_default().insert(&e.dst);
            adjacent.entry(&e.dst).or_default().insert(&e.src);
        }
    }
    let closed = closure(&tasks, &dependents);
    let laned = lanes(&adjacent);

    // Degree counts over every edge, live or not, as the view's joins do.
    let mut needs_open: BTreeMap<&str, (i64, i64, i64)> = BTreeMap::new();
    let mut needed_by: BTreeMap<&str, (i64, i64)> = BTreeMap::new();
    let mut live_children: BTreeMap<&str, i64> = BTreeMap::new();
    let mut discovered: BTreeMap<&str, i64> = BTreeMap::new();
    for e in &edges {
        let src = &tasks[&e.src];
        let dst = tasks.get(&e.dst);
        match e.kind {
            Kind::Needs => {
                let n = needs_open.entry(&e.src).or_default();
                match dst {
                    None => {
                        n.0 += 1;
                        n.1 += 1;
                    }
                    Some(d) if !terminal(&d.status) => {
                        n.0 += 1;
                        if d.repo != src.repo {
                            n.2 += 1;
                        }
                    }
                    Some(_) => {}
                }
                if let Some(d) = dst.filter(|_| live(&src.status)) {
                    let b = needed_by.entry(&e.dst).or_default();
                    b.0 += 1;
                    if d.repo != src.repo {
                        b.1 += 1;
                    }
                }
            }
            Kind::Parent => {
                if live(&src.status) {
                    *live_children.entry(&e.dst).or_default() += 1;
                }
            }
            Kind::DiscoveredFrom => *discovered.entry(&e.dst).or_default() += 1,
        }
    }

    tasks
        .iter()
        .filter(|(_, n)| n.status != "invalid" && repos.contains(n.repo.as_str()))
        .map(|(gid, n)| {
            let place = n.seq.unwrap_or(UNRANKED);
            let (unlock, depth, inherited) = closed.get(gid.as_str()).copied().unwrap_or_default();
            let inherit = inherited.unwrap_or(place);
            let (lane, lane_size) = laned
                .get(gid.as_str())
                .map_or((None, 1), |(l, s)| (Some((*l).to_string()), *s));
            let (needs_open, needs_unresolved, needs_open_foreign) =
                needs_open.get(gid.as_str()).copied().unwrap_or_default();
            let (needed_by_open, needed_by_foreign) =
                needed_by.get(gid.as_str()).copied().unwrap_or_default();
            let live_children = live_children.get(gid.as_str()).copied().unwrap_or_default();
            GraphRow {
                gid: gid.clone(),
                repo: n.repo.clone(),
                id: n.id.clone(),
                status: n.status.clone(),
                seq: n.seq,
                unlock,
                depth,
                place,
                inherit,
                needs_behind: inherit < place,
                lane,
                lane_size,
                needs_open,
                needs_unresolved,
                needs_open_foreign,
                needed_by_open,
                needed_by_foreign,
                live_children,
                discovered: discovered.get(gid.as_str()).copied().unwrap_or_default(),
                ready: n.status == "open" && needs_open == 0 && live_children == 0,
            }
        })
        .collect()
}
