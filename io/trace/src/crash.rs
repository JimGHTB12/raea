//! What a crash leaves: stderr teed into the session, the binary's image for symbolicating, a panic's report, a fatal
//! signal's backtrace, and macOS's crash report for an earlier session that ended unclean.

use super::*;

// ---- stderr, teed

pub(crate) fn tee_stderr(dir: &Path) {
    #[cfg(not(unix))]
    let _ = dir;
    #[cfg(unix)]
    unsafe {
        let mut fds = [0i32; 2];
        if libc::pipe(fds.as_mut_ptr()) != 0 {
            return;
        }
        let original = libc::dup(2);
        if original < 0 || libc::dup2(fds[1], 2) < 0 {
            return;
        }
        libc::close(fds[1]);
        let Ok(mut log) = OpenOptions::new().create(true).append(true).open(dir.join("stderr.log")) else { return };
        let read = fds[0];
        let _ = std::thread::Builder::new().name("raea trace stderr".into()).spawn(move || {
            let mut buf = [0u8; 8192];
            loop {
                let n = libc::read(read, buf.as_mut_ptr() as *mut libc::c_void, buf.len());
                if n <= 0 {
                    break;
                }
                let b = &buf[..n as usize];
                libc::write(original, b.as_ptr() as *const libc::c_void, b.len());
                let _ = log.write_all(b);
                let _ = log.flush();
            }
        });
    }
}

// ---- the image, for symbolicating a stripped build's backtraces

/// The crash file's image line, made at start (the signal handler only reads it).
#[cfg(unix)]
pub(crate) static CRASH_IMAGE: std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// The main executable's Mach-O UUID, load address and ASLR slide: what atos needs to turn a stripped build's backtrace
/// addresses into names against the dSYM kept for that UUID (`atos -o NAME.dSYM -l LOAD ADDR...`).
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub(crate) fn image_line() -> Option<String> {
    unsafe extern "C" {
        fn _dyld_get_image_header(i: u32) -> *const u8;
        fn _dyld_get_image_vmaddr_slide(i: u32) -> isize;
    }
    unsafe {
        let h = _dyld_get_image_header(0);
        if h.is_null() {
            return None;
        }
        let slide = _dyld_get_image_vmaddr_slide(0);
        // mach_header_64: magic, cputype, cpusubtype, filetype, ncmds, sizeofcmds, flags, reserved (32 bytes)
        let ncmds = (h.add(16) as *const u32).read_unaligned();
        let mut p = h.add(32);
        let mut uuid = None;
        for _ in 0..ncmds {
            let cmd = (p as *const u32).read_unaligned();
            let size = (p.add(4) as *const u32).read_unaligned() as usize;
            // LC_UUID
            if cmd == 0x1b {
                let b = std::slice::from_raw_parts(p.add(8), 16);
                uuid = Some(format!(
                    "{:02X}{:02X}{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
                    b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]
                ));
                break;
            }
            if size == 0 {
                break;
            }
            p = p.add(size);
        }
        Some(format!("uuid {} load 0x{:x} slide 0x{:x}", uuid.unwrap_or_else(|| "?".into()), h as usize, slide))
    }
}

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
pub(crate) fn image_line() -> Option<String> {
    None
}

// ---- panics

pub(crate) fn install_panic_hook() {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let bt = std::backtrace::Backtrace::force_capture();
        let thread = std::thread::current().name().unwrap_or("unnamed").to_string();
        let place = info.location().map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column())).unwrap_or_default();
        let msg = info.payload().downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| info.payload().downcast_ref::<String>().cloned()).unwrap_or_else(|| "(not a string)".into());
        // the session's lock may be held by the panicking thread: try, never wait
        if let Ok(mut g) = SESSION.try_lock() {
            if let Some(s) = g.as_mut() {
                let (iso, _) = utc(SystemTime::now());
                let recent: Vec<String> = s.ring.iter().cloned().collect();
                let text = format!("panic at {iso} ({:.3} s into the run)\nthread: {thread}\nat: {place}\nmessage: {msg}\nimage: {}\n\nbacktrace:\n{bt}\n\nthe last {} events:\n{}\n", s.t0.elapsed().as_secs_f64(), image_line().unwrap_or_default(), recent.len(), recent.join("\n"));
                let _ = std::fs::write(s.dir.join("panic.txt"), text);
                let line = json!({"t": s.t0.elapsed().as_secs_f64(), "kind": "panic", "thread": thread, "at": place, "message": msg});
                write_line(s, line.to_string());
            }
        }
        prev(info);
    }));
}

// ---- fatal signals

#[cfg(unix)]
extern "C" fn on_signal(sig: i32) {
    // async-signal-safe only: write(2) and backtrace(3)/backtrace_symbols_fd(3), into the file opened at start
    unsafe {
        let fd = CRASH_FD;
        if fd >= 0 {
            let name: &[u8] = match sig {
                libc::SIGSEGV => b"fatal signal SIGSEGV (a bad memory access)\n",
                libc::SIGBUS => b"fatal signal SIGBUS (a bad memory access)\n",
                libc::SIGILL => b"fatal signal SIGILL (an illegal instruction)\n",
                libc::SIGFPE => b"fatal signal SIGFPE (an arithmetic fault)\n",
                libc::SIGABRT => b"fatal signal SIGABRT (abort)\n",
                libc::SIGTRAP => b"fatal signal SIGTRAP (a trap: a Swift/Objective-C runtime check or a breakpoint)\n",
                _ => b"fatal signal\n",
            };
            libc::write(fd, name.as_ptr() as *const libc::c_void, name.len());
            if let Some(img) = CRASH_IMAGE.get() {
                libc::write(fd, img.as_ptr() as *const libc::c_void, img.len());
            }
            let head = b"backtrace (raw; macOS's crash report, copied in as crash.ips at the next start, has it symbolicated):\n";
            libc::write(fd, head.as_ptr() as *const libc::c_void, head.len());
            let mut frames = [std::ptr::null_mut::<libc::c_void>(); 128];
            let n = backtrace(frames.as_mut_ptr(), 128);
            backtrace_symbols_fd(frames.as_ptr(), n, fd);
        }
        // the default action (a core and macOS's crash report), as if this handler weren't here
        libc::signal(sig, libc::SIG_DFL);
        libc::raise(sig);
    }
}

#[cfg(unix)]
unsafe extern "C" {
    fn backtrace(buf: *mut *mut libc::c_void, size: i32) -> i32;
    fn backtrace_symbols_fd(buf: *const *mut libc::c_void, size: i32, fd: i32);
}

pub(crate) fn install_signal_handlers() {
    #[cfg(unix)]
    unsafe {
        for sig in [libc::SIGSEGV, libc::SIGBUS, libc::SIGILL, libc::SIGFPE, libc::SIGABRT, libc::SIGTRAP] {
            libc::signal(sig, on_signal as extern "C" fn(i32) as libc::sighandler_t);
        }
    }
}

// ---- earlier sessions

/// Sessions under `base` that ended unclean (no end.json, their process gone) get macOS's crash report for their pid.
pub(crate) fn adopt_crash_reports(base: &Path) {
    let reports = std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Logs/DiagnosticReports"));
    let Ok(entries) = std::fs::read_dir(base) else { return };
    for e in entries.flatten() {
        let d = e.path();
        if d.join("end.json").exists() || d.join("crash.ips").exists() || d.join("unclean.json").exists() {
            continue;
        }
        let Some(pid) = d.file_name().and_then(|n| n.to_str()).and_then(|n| n.rsplit('-').next()).and_then(|p| p.parse::<i32>().ok()) else { continue };
        if alive(pid) {
            continue; // still running (another instance)
        }
        let mut found = None;
        let since = std::fs::metadata(&d).and_then(|m| m.created().or(m.modified())).ok();
        if let Some(dir) = &reports {
            if let Ok(list) = std::fs::read_dir(dir) {
                for r in list.flatten() {
                    let p = r.path();
                    if p.extension().and_then(|x| x.to_str()) != Some("ips") {
                        continue;
                    }
                    // only reports written after the session began
                    let at = r.metadata().and_then(|m| m.modified()).ok();
                    if let (Some(at), Some(since)) = (at, since) {
                        if at < since {
                            continue;
                        }
                    }
                    // the report's header line and body name the pid ("pid" : 1234)
                    if let Ok(text) = std::fs::read_to_string(&p) {
                        let needle = format!("\"pid\" : {pid},");
                        if text.contains(&needle) && text.len() < 64 << 20 {
                            found = Some(p);
                            break;
                        }
                    }
                }
            }
        }
        match found {
            Some(p) => {
                let _ = std::fs::copy(&p, d.join("crash.ips"));
            }
            None => {
                let _ = std::fs::write(d.join("unclean.json"), json!({"note": "ended without end.json and no macOS crash report names its pid: killed (SIGKILL, memguard, a force quit) or the report wasn't written yet"}).to_string());
            }
        }
    }
}

#[cfg(unix)]
fn alive(pid: i32) -> bool {
    unsafe { libc::kill(pid, 0) == 0 }
}

#[cfg(windows)]
fn alive(pid: i32) -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, STILL_ACTIVE};
    use windows_sys::Win32::System::Threading::{GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid as u32);
        if h.is_null() {
            return false;
        }
        let mut code = 0u32;
        let ok = GetExitCodeProcess(h, &mut code) != 0;
        CloseHandle(h);
        ok && code == STILL_ACTIVE as u32
    }
}

/// Only the last KEEP sessions stay.
pub(crate) fn prune(base: &Path) {
    let Ok(entries) = std::fs::read_dir(base) else { return };
    let mut dirs: Vec<PathBuf> = entries.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
    dirs.sort();
    while dirs.len() >= KEEP {
        let d = dirs.remove(0);
        let _ = std::fs::remove_dir_all(d);
    }
}
