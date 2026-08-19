//! meshwork — thin binary shell; everything lives in the library
//! (`meshwork::cli` dispatches per DESIGN §6).

fn main() {
    // Die quietly when the reader closes the pipe (`ready | head`), like
    // any unix filter: Rust masks SIGPIPE at startup, which would turn
    // the EPIPE into a panic mid-listing. Restore the default before any
    // output exists.
    #[cfg(unix)]
    // SAFETY: single-threaded here — no other code is running, no signal
    // handler is being replaced beyond the startup mask.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
    std::process::exit(meshwork::cli::run());
}
