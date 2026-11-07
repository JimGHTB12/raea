//! The system's host calls (os.open, net.clipboard.*, fs.*, dialog.message; docs/targets.feature T2d), which every
//! root has unless the app registers its own under the same names. raea-io-os answers them, as it does for the native
//! target; a headless host (raea app check) calls raea_io_os::set_headless first, so nothing reaches the system.

use raea_io_os::CALLS;

use crate::calls::HostCalls;

/// The calls, onto `calls` (HostCalls's default has them).
pub fn install(calls: &mut HostCalls) {
    for name in CALLS {
        calls.on(name, move |_w, args, answer| {
            raea_io_os::call(name, args, Box::new(move |r| match r {
                Ok(v) => answer.ok(v),
                Err(e) => answer.err(e),
            }));
        });
    }
}
