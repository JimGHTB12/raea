// What node.js's fs and child_process report that Unix and Windows keep differently.

/// fs.stat's mode and ctimeMs (the inode change time).
#[cfg(unix)]
pub(super) fn mode_ctime(m: &std::fs::Metadata) -> (u32, f64) {
    use std::os::unix::fs::MetadataExt;
    (m.mode(), m.ctime() as f64 * 1e3 + m.ctime_nsec() as f64 / 1e6)
}

/// Windows has no mode bits or ctime: Node's own stat reports a directory as 0o40666 and a file as 0o100666 (0o444
/// read-only), and ctimeMs as the last write.
#[cfg(windows)]
pub(super) fn mode_ctime(m: &std::fs::Metadata) -> (u32, f64) {
    let perm = if m.permissions().readonly() { 0o444 } else { 0o666 };
    ((if m.is_dir() { 0o40000 } else { 0o100000 }) | perm,super::millis(m.modified()))
}

/// The signal that ended a child, as Node names it (null when it exited with a code).
#[cfg(unix)]
pub(super) fn signal(st: &std::process::ExitStatus) -> Option<String> {
    std::os::unix::process::ExitStatusExt::signal(st).map(signal_name)
}

/// Windows has no signals: a child always exits with a code.
#[cfg(windows)]
pub(super) fn signal(_: &std::process::ExitStatus) -> Option<String> {
    None
}

#[cfg(unix)]
fn signal_name(n: i32) -> String {
    match n {
        libc::SIGTERM => "SIGTERM".into(),
        libc::SIGKILL => "SIGKILL".into(),
        libc::SIGINT => "SIGINT".into(),
        libc::SIGSEGV => "SIGSEGV".into(),
        libc::SIGABRT => "SIGABRT".into(),
        n => format!("SIG{n}"),
    }
}
