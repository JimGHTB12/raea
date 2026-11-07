#!/usr/bin/env bash
# raea-web's wasm (engine/web) built with the web profile and bound for the browser into sdk/web/wasm (not in git):
# raea_web.js and raea_web_bg.wasm, which @raea/web's loadRaea instantiates. Needs the wasm32 target and wasm-bindgen.
set -euo pipefail
cd "$(dirname "$0")/../../.."
CARGO_TARGET_DIR=$PWD/target/web lane build -- cargo build -p raea-web --target wasm32-unknown-unknown --profile web -j 6 --message-format short
wasm-bindgen --target web --out-dir sdk/web/wasm --no-typescript target/web/wasm32-unknown-unknown/web/raea_web.wasm
ls -la sdk/web/wasm
