//! meshwork — task graph as markdown-in-git, queried with SQL, no database.
//!
//! Library crate: the binary in `main.rs` is a thin CLI shell; everything
//! testable lives here. Normative references: REQUIREMENTS-meshwork.md
//! (`MW-*`) and DESIGN-meshwork.md (`§*`).

pub mod cli;
pub mod clock;
pub mod docs;
pub mod edit;
pub mod id;
pub mod lint;
pub mod parse;
pub mod registry;
pub mod registry_hygiene;
pub mod store;
pub mod tables;
pub mod trust;
pub mod verify_dsl;
pub mod write;
