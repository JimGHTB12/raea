//! The net.* host calls (fetch, WebSockets, localStorage under sdk/ui/src/platform.ts; docs/ts-native.feature P17),
//! which every root has unless the app registers its own under the same names. raea-io-net-calls answers them, as it
//! does for the native target (docs/targets.feature T2a lists the call surface); this hands it each call with the
//! calling root's Entity. The clipboard's net.clipboard.* calls are src/os.rs's (raea-io-os): a headless host's
//! clipboard is its process's own, so a test never touches the system's.

use std::sync::Arc;
use std::sync::mpsc::Sender;

use bevy::prelude::*;
use raea_io_net_calls::{CALLS, Net};
use serde_json::Value;

use crate::calls::HostCalls;

/// The calls, onto `calls` (HostCalls's default has them); `push` carries a socket's events to its root.
pub fn install(calls: &mut HostCalls, push: Sender<(Entity, Value)>) {
    let net: Net<Entity> = Net::new(Arc::new(move |root, ev| {
        let _ = push.send((root, ev));
    }));
    for name in CALLS {
        let net = net.clone();
        calls.on(name, move |_w, args, answer| {
            let root = answer.root();
            net.call(root, name, args, Box::new(move |r| match r {
                Ok(v) => answer.ok(v),
                Err(e) => answer.err(e),
            }));
        });
    }
    // localStorage's kept values, synchronously too: sdk/ui's storage.ts has them before the page's code reads them
    crate::calls::on_sync("net.storage.get", raea_io_net_calls::storage_get);
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    /// A page reads what an earlier run kept at once (raea.app's open workspace and dismissed hints, 11-09).
    #[test]
    fn kept_storage_answers_synchronously() {
        let dir = std::env::temp_dir().join(format!("raea-js-storage-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("local.json"), r#"{"raea.settings":"{\"workspace\":\"raea\"}"}"#).unwrap();
        // SAFETY: nothing else in this crate's tests reads BEVY_TSX_STORAGE
        unsafe { std::env::set_var("BEVY_TSX_STORAGE", &dir) };
        let _calls = crate::calls::HostCalls::default();
        let got: Value = serde_json::from_str(&crate::calls::sync("net.storage.get", r#"{"area":"local"}"#)).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(got["ok"]["raea.settings"], r#"{"workspace":"raea"}"#, "{got}");
    }
}
