//! Whether a process still runs, asked of the system itself: a `kill` found on PATH can't be trusted (on Windows under
//! Git Bash it is MSYS's, which knows no Windows PID and called every live render dead, #29).

/// Whether process `pid` runs: Some(false) only when there's no such process, None when the system won't say (so a
/// caller deciding whether to delete what it left keeps it).
#[cfg(unix)]
pub(crate) fn running(pid: u32) -> Option<bool> {
    // 0 and anything past i32::MAX name process groups, not one process
    let p = i32::try_from(pid).ok().filter(|p| *p > 0)?;
    // signal 0 sends nothing: it only checks that the process exists and could be signalled
    if unsafe { libc::kill(p, 0) } == 0 {
        return Some(true);
    }
    match std::io::Error::last_os_error().raw_os_error() {
        Some(libc::ESRCH) => Some(false),
        // another user's process
        Some(libc::EPERM) => Some(true),
        _ => None,
    }
}

/// Whether process `pid` runs: Some(false) only when there's no such process, None when the system won't say (so a
/// caller deciding whether to delete what it left keeps it).
#[cfg(windows)]
pub(crate) fn running(pid: u32) -> Option<bool> {
    use windows_sys::Win32::Foundation::{CloseHandle, ERROR_INVALID_PARAMETER, GetLastError, STILL_ACTIVE};
    use windows_sys::Win32::System::Threading::{GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if h.is_null() {
            // no such process; access denied (another user's, a protected one) says nothing
            return (GetLastError() == ERROR_INVALID_PARAMETER).then_some(false);
        }
        let mut code = 0u32;
        let ok = GetExitCodeProcess(h, &mut code) != 0;
        CloseHandle(h);
        ok.then_some(code == STILL_ACTIVE as u32)
    }
}

#[cfg(not(any(unix, windows)))]
pub(crate) fn running(_pid: u32) -> Option<bool> {
    None
}

#[cfg(test)]
mod tests {
    use super::running;

    /// A child that waits until its stdin closes, on any system.
    fn child() -> std::process::Child {
        #[cfg(windows)]
        let mut c = std::process::Command::new("cmd");
        #[cfg(windows)]
        c.args(["/c", "more"]);
        #[cfg(not(windows))]
        let mut c = std::process::Command::new("cat");
        c.stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::null()).spawn().unwrap()
    }

    #[test]
    fn a_process_runs_until_it_ends() {
        assert_eq!(running(std::process::id()), Some(true));
        let mut c = child();
        let pid = c.id();
        assert_eq!(running(pid), Some(true));
        drop(c.stdin.take());
        c.wait().unwrap();
        // reaped, so its PID names nothing (until the system hands it out again, which takes far longer than this)
        assert_eq!(running(pid), Some(false));
    }

    /// The sweep each render starts with (cache.rs) removes a work folder only once its render's process is gone
    #[test]
    fn the_sweep_keeps_a_running_renders_folder() {
        let dir = std::env::temp_dir().join(format!("raea-pid-sweep-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut live = child();
        let mut ended = child();
        drop(ended.stdin.take());
        ended.wait().unwrap();
        let keep = [format!("raea-cli-b{}", live.id()), format!("raea-cli-r{}-ThreadId(2)", live.id()), "raea-cli-bx".into()];
        let go = [format!("raea-cli-b{}", ended.id()), format!("raea-cli-r{}-ThreadId(2)", ended.id())];
        for n in keep.iter().chain(&go) {
            std::fs::create_dir_all(dir.join(n)).unwrap();
        }
        crate::cache::sweep_stale_work_in(&dir, std::time::SystemTime::now());
        let left: Vec<bool> = keep.iter().chain(&go).map(|n| dir.join(n).exists()).collect();
        drop(live.stdin.take());
        live.wait().unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(left, [true, true, true, false, false], "kept: {keep:?}, removed: {go:?}");
    }

    /// Two hours untouched removes a folder only when its render isn't known to run: a long render stops touching its
    /// folder's top level well before it ends (#31)
    #[test]
    fn the_age_rule_spares_a_running_render() {
        let dir = std::env::temp_dir().join(format!("raea-pid-age-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut live = child();
        let mut ended = child();
        drop(ended.stdin.take());
        ended.wait().unwrap();
        let names = [
            format!("raea-cli-b{}", live.id()),
            format!("raea-cli-r{}-ThreadId(2)", std::process::id()),
            format!("raea-cli-b{}", ended.id()),
            "raea-cli-bx".to_string(),
        ];
        for n in &names {
            std::fs::create_dir_all(dir.join(n)).unwrap();
        }
        // three hours from now, every folder is two hours untouched
        crate::cache::sweep_stale_work_in(&dir, std::time::SystemTime::now() + std::time::Duration::from_secs(3 * 3600));
        let left: Vec<bool> = names.iter().map(|n| dir.join(n).exists()).collect();
        drop(live.stdin.take());
        live.wait().unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        // a running render's (another process's, and this one's) stay; an ended one's and one naming no process go
        assert_eq!(left, [true, true, false, false], "{names:?}");
    }

    #[test]
    fn a_pid_naming_no_one_process_is_unknown_or_gone() {
        // 0 and pids past i32::MAX are groups to kill(2), never "running"
        assert_ne!(running(0), Some(true));
        assert_ne!(running(u32::MAX), Some(true));
    }
}
