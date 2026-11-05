// raea's main thread gets 8 MB of stack on Windows, as it has on macOS and Linux: Windows gives an executable 1 MB by
// default, while QuickJS is allowed 4 MB (runtime/jsengine) and Bevy's and Blitz's setup run there (PT20).
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!("cargo:rustc-link-arg-bin=raea=/STACK:8388608");
    }
}
