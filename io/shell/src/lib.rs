//! The shell's natives, as raea's native calls: a project's files (fs), its network under a policy (net), SQLite (sql),
//! and tar, gzip and HTML-to-markdown (archive). raea-native seeds the registry with them.

mod archive;
mod fs;
// the shell's network (raea-io-net): off in an exported game's player, which fetches nothing (lightspeed R7)
#[cfg(feature = "net")]
mod net;
mod sql;

/// Every call the shell answers.
pub fn calls() -> Vec<raea_core_calls::Call> {
    let mut all = fs::calls();
    #[cfg(feature = "net")]
    all.extend(net::calls());
    all.extend(sql::calls().into_iter().chain(archive::calls()));
    all
}
