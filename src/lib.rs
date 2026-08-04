//! meshwork — task graph as markdown-in-git, queried with SQL, no database.
//!
//! Library crate: the binary in `main.rs` is a thin CLI shell; everything
//! testable lives here. Normative references: REQUIREMENTS-meshwork.md
//! (`MW-*`) and DESIGN-meshwork.md (`§*`).

pub mod id;
pub mod parse;
