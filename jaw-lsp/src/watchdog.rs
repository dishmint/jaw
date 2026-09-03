//! Parent-process liveness watchdog.
//!
//! The LSP `initialize` request carries the client's `processId`. When a
//! client crashes or is killed without sending `shutdown`/`exit`, the server
//! can be orphaned. The usual signal — EOF on stdin — is reliable in the
//! common case, but if the server's stdin was inherited and kept open
//! elsewhere the read never returns and the process leaks. Monitoring the
//! parent PID and self-terminating when it disappears is the standard belt-
//! and-suspenders fix (rust-analyzer and most servers do the same).
//!
//! Note: PID-reuse is a known, accepted limitation of PID-based monitoring —
//! in the small window between the parent dying and the next poll, its PID
//! could be recycled. The poll interval keeps that window short.

#[cfg(unix)]
const POLL_INTERVAL_SECS: u64 = 10;

/// Spawn a background thread that exits the process once `parent_pid` is gone.
#[cfg(unix)]
pub fn spawn(parent_pid: i32) {
    use std::thread;
    use std::time::Duration;

    // A non-positive PID would make `kill` address a process group / every
    // process; never monitor those.
    if parent_pid <= 1 {
        return;
    }

    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(POLL_INTERVAL_SECS));

        // kill(pid, 0) sends no signal; it only checks existence/permission.
        // rc == 0            -> alive
        // errno == EPERM     -> exists but not ours to signal -> alive
        // errno == ESRCH etc -> gone
        let rc = unsafe { libc::kill(parent_pid, 0) };
        if rc == 0 {
            continue;
        }
        if std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM) {
            continue;
        }

        eprintln!("jaw-lsp: parent process {parent_pid} is gone; exiting");
        std::process::exit(0);
    });
}

/// No-op on platforms without the POSIX `kill` probe.
#[cfg(not(unix))]
pub fn spawn(_parent_pid: i32) {}
