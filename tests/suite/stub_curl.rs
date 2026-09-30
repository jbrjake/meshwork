//! `stub_curl::` — the offline `curl` double (MW-J6) the plugin hook's
//! release fetch lands on. It records every invocation, replays a canned
//! body keyed by the URL's last path segment, exits as canned, and refuses
//! tests that didn't opt in — the same discipline as the `gh` stub.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn run_stub(args: &[&str], canned: &Path, calls: Option<&Path>) -> Output {
    let mut cmd = Command::new(repo_root().join("tests/bin/curl"));
    cmd.args(args)
        .env("CURL_STUB_CANNED", canned)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    match calls {
        Some(p) => {
            cmd.env("CURL_STUB_CALLS", p);
        }
        None => {
            cmd.env_remove("CURL_STUB_CALLS");
        }
    }
    cmd.output().expect("stub curl spawns")
}

const URL: &str = "https://github.com/jbrjake/meshwork/releases/download/v9.9.9/meshwork-v9.9.9-aarch64-apple-darwin.tar.gz";

#[test]
fn records_argv_and_writes_body_to_output_file() {
    let dir = tempfile::tempdir().unwrap();
    let canned = dir.path().join("canned");
    fs::create_dir_all(&canned).unwrap();
    fs::write(
        canned.join("meshwork-v9.9.9-aarch64-apple-darwin.tar.gz"),
        b"tarball bytes",
    )
    .unwrap();
    let calls = dir.path().join("curl.calls");
    let out_file = dir.path().join("release.tar.gz");
    let out = run_stub(
        &[
            "-fsSL",
            "--retry",
            "2",
            "-o",
            out_file.to_str().unwrap(),
            URL,
        ],
        &canned,
        Some(&calls),
    );
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    assert_eq!(
        fs::read(&out_file).unwrap(),
        b"tarball bytes",
        "body lands in -o"
    );
    assert!(out.stdout.is_empty(), "nothing on stdout when -o is given");
    let rec = fs::read_to_string(&calls).unwrap();
    assert_eq!(
        rec,
        format!("$ -fsSL --retry 2 -o {} {URL}\n", out_file.display())
    );
}

#[test]
fn canned_exit_fails_like_curl_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let canned = dir.path().join("canned");
    fs::create_dir_all(&canned).unwrap();
    fs::write(
        canned.join("meshwork-v9.9.9-aarch64-apple-darwin.tar.gz.exit"),
        "22\n",
    )
    .unwrap();
    let calls = dir.path().join("curl.calls");
    let out_file = dir.path().join("release.tar.gz");
    let out = run_stub(
        &["-fsSL", "-o", out_file.to_str().unwrap(), URL],
        &canned,
        Some(&calls),
    );
    assert_eq!(out.status.code(), Some(22));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("404"),
        "curl-shaped stderr"
    );
    assert!(!out_file.exists(), "a failed fetch leaves no partial file");
    // Still recorded — a failed call must leave evidence.
    assert!(fs::read_to_string(&calls).unwrap().starts_with("$ -fsSL"));
}

#[test]
fn unknown_url_fails_loudly() {
    let dir = tempfile::tempdir().unwrap();
    let canned = dir.path().join("canned");
    fs::create_dir_all(&canned).unwrap();
    let calls = dir.path().join("curl.calls");
    let out = run_stub(&["-fsSL", URL], &canned, Some(&calls));
    assert_eq!(out.status.code(), Some(64));
    assert!(String::from_utf8_lossy(&out.stderr).contains("no canned body"));
}

#[test]
fn refuses_without_calls_env() {
    let dir = tempfile::tempdir().unwrap();
    let out = run_stub(&["-fsSL", URL], dir.path(), None);
    assert_eq!(
        out.status.code(),
        Some(66),
        "a curl call from a test that never opted in is itself a bug (MW-J6)"
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("CURL_STUB_CALLS"));
}

#[test]
fn harness_path_resolves_stub() {
    // The harness contract (gate §3 does the same): tests/bin prepended to
    // PATH makes `curl` resolve to the stub, never the real binary.
    let path = format!(
        "{}:{}",
        repo_root().join("tests/bin").display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out = Command::new("sh")
        .args(["-c", "command -v curl"])
        .env("PATH", path)
        .output()
        .unwrap();
    let resolved = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        resolved.trim(),
        repo_root().join("tests/bin/curl").to_string_lossy(),
        "curl must resolve to the stub"
    );
}
