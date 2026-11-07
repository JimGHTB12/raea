#!/bin/bash
# The web modules of a release, attached to its GitHub release beside the engine's archives (tools/release/README.md):
#   raea_shape_web.js, raea_shape_web_bg.wasm  core/shape for the browser (engine/shape-web): what @raea/r3f's
#                                              loadShapes() meshes solids with (minus, hull, extrude, Text3D)
#   raea_web.js, raea_web_bg.wasm              raea's engine on WebGPU (engine/web): what @raea/web's loadRaea() opens
#   SHA256SUMS
# Each is wasm-bindgen's `--target web` glue and its module, built with the web profile. A page serves them itself:
# GitHub sends release files as application/octet-stream with no CORS header, so a browser can't import them from
# github.com. Built from REF's committed tree (a git archive of it, never anyone's unfinished edit), as release.sh is.
#
#   tools/release-web.sh VERSION [REF]                writes dist/web-VERSION/ (REF: HEAD)
#   tools/release-web.sh VERSION REF --upload TAG     and attaches them to the GitHub release TAG (shinyobjectz/raea)
set -euo pipefail
version=${1:?usage: release-web.sh VERSION [REF [--upload TAG]]}
ref=${2:-HEAD}
tag=
[ "${3:-}" = --upload ] && tag=${4:?usage: release-web.sh VERSION REF --upload TAG}
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"
out=$root/dist/web-$version
export CARGO_TARGET_DIR=$root/target/web
src=$(mktemp -d)
trap 'rm -rf "$src"' EXIT
git archive "$ref" | tar -x -C "$src"
mkdir -p "$out"
for p in raea-shape-web raea-web; do
  (cd "$src" && lane build -- cargo build --locked -p "$p" --target wasm32-unknown-unknown --profile web -j 6 --message-format short)
  wasm-bindgen --target web --no-typescript --out-dir "$out" "$CARGO_TARGET_DIR/wasm32-unknown-unknown/web/${p//-/_}.wasm"
done
(cd "$out" && shasum -a 256 raea_shape_web.js raea_shape_web_bg.wasm raea_web.js raea_web_bg.wasm > SHA256SUMS && cat SHA256SUMS)
# each module opens under its glue, as loadShapes and loadRaea open it
for m in raea_shape_web raea_web; do
  bun -e "const m = await import('$out/$m.js'); await m.default({ module_or_path: await Bun.file('$out/${m}_bg.wasm').arrayBuffer() }); console.log('$m opens')" \
    || { echo "release-web.sh: $m.js didn't open its module" >&2; exit 1; }
done
[ -z "$tag" ] || gh release upload "$tag" -R shinyobjectz/raea --clobber "$out"/raea_shape_web.js "$out"/raea_shape_web_bg.wasm "$out"/raea_web.js "$out"/raea_web_bg.wasm "$out"/SHA256SUMS
