#!/usr/bin/env bash
# Builds raea's solid shapes for the web (engine/shape-web, core/shape compiled to wasm) into sdk/r3f/wasm: what
# <Raea> and the three host mesh solids, minus, hull, extrude and Text3D with (loadShapes). Not in git; built here.
set -euo pipefail
R=$(cd "$(dirname "$0")/../../.." && pwd)
OUT="$R/sdk/r3f/wasm"
export CARGO_TARGET_DIR="$R/target/web"
lane build -- cargo build -p raea-shape-web --target wasm32-unknown-unknown --profile web -j 6
wasm-bindgen --target web --out-dir "$OUT" "$CARGO_TARGET_DIR/wasm32-unknown-unknown/web/raea_shape_web.wasm"
ls -la "$OUT"
