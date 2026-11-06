// The cdylib's install name on macOS is @rpath/libraea_bundle.dylib, not the path it was linked at: raea loads it by
// path (runtime/bundle/src/fresh.rs), so the name is only a label, but the default one carried the build machine's
// folder into every release (public issue #6). Set at link time, so the linker's own signature stays valid.
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-cdylib-link-arg=-Wl,-install_name,@rpath/libraea_bundle.dylib");
    }
}
