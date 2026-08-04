//! meshwork test suite — single integration binary so test paths match
//! TRACE.md names (`fixtures::corpus_covers_features`, `e2e::init_layout`, …).
//! Tiers per DESIGN §13: fixtures = corpus honesty, e2e = real binary in a
//! tempdir. Unit tests live in src/ modules. Everything offline (MW-J6).

mod e2e;
mod fixtures;
mod stub_gh;
