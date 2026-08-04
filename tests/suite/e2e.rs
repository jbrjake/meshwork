//! E2E scenario tests — drive the real binary (DESIGN §13). Grows with M0+.

use assert_cmd::Command;

/// Bootstrap stub: until M0 verbs land, the binary must point at the plan
/// and exit 2 — never pretend to work. Replaced when `init` (PLAN 0.4) lands.
#[test]
fn binary_bootstrap_stub_exits_2() {
    Command::cargo_bin("meshwork")
        .expect("meshwork binary builds")
        .assert()
        .code(2);
}
