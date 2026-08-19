// e2e part-file: broken-pipe behavior (mw-n0kfw5b). Included by e2e.rs.

/// mw-n0kfw5b: `ready | head` must die quietly when the reader closes the
/// pipe, like any unix filter — termination by SIGPIPE itself, nothing on
/// stderr. Rust masks SIGPIPE at startup, which turns the EPIPE into a
/// panic unless the binary restores the default disposition.
#[test]
#[cfg(unix)]
fn sigpipe_quiet() {
    use std::os::unix::process::ExitStatusExt as _;

    let (_g, repo) = git_repo("sigpipe");
    init_store(&repo);
    for i in 0..5 {
        add_task(&repo, &format!("Filler task {i}"));
    }

    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_meshwork"))
        .arg("ready")
        .current_dir(&repo)
        .env("HOME", &repo)
        .env("MESHWORK_TRUST", "1")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    // The reader is gone before the child's first write: every write from
    // here on is EPIPE, so the kernel delivers SIGPIPE.
    drop(child.stdout.take());
    let out = child.wait_with_output().unwrap();

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!stderr.contains("panic"), "broken pipe panicked:\n{stderr}");
    assert_eq!(
        out.status.signal(),
        Some(13),
        "want death by SIGPIPE, got {:?}",
        out.status
    );
}
