#!/bin/bash
# The agent's sandbox (docs/media.feature M5), built from nothing: the WASI tools (rtools, the lean file and text set;
# jaq), shrunk with wasm-opt, compiled ahead of time for Pulley by wsxc, then the Mac's suite run.
#   engine/sandbox/build.sh
set -eu
cd "$(dirname "$0")"
H=$PWD; T=$H/target; M=$H/modules; mkdir -p $M
RC=$(rustup which rustc)
wasm() { lane build -- env RUSTC=$RC CARGO_TARGET_DIR=$T ~/.cargo/bin/cargo "$@" --target wasm32-wasip1; }
wasm build --release --manifest-path $H/rtools/Cargo.toml
wasm install jaq --version 1.6.0 --locked --no-default-features --root $T/tools
lane build -- env CARGO_TARGET_DIR=$T cargo build --release --features compile --manifest-path $H/wsx/Cargo.toml
for w in $T/wasm32-wasip1/release/rtools.wasm $T/tools/bin/jaq.wasm; do
  n=$(basename $w .wasm)
  wasm-opt -Oz --strip-debug --strip-producers --enable-bulk-memory --enable-sign-ext --enable-nontrapping-float-to-int --enable-mutable-globals $w -o $T/$n.opt.wasm
  $T/release/wsxc $T/$n.opt.wasm $M/$n.cwasm
done
mkdir -p $T/work && $T/release/wsx-suite $M $T/work | python3 -c "import json,sys; d=json.load(sys.stdin); print('Mac:', d['engine'], d['as_they_must'])"
