#!/bin/bash
# A release of the raea engine for macOS on Apple Silicon, Linux on x86_64 or Windows on x86_64 (whichever machine runs
# it; Linux's is built on the probe box in an ubuntu:24.04 root, for glibc 2.39; Windows's on GitHub's windows-latest
# runner, .github/workflows/windows.yml's release mode, in Git's bash): raea and its bundler, with raea's own files
# beside them (share/raea: the raea SDK's TypeScript, the lite runtime and its packages, the shell's sources, the tools
# that install the local music and foley models, and the agent skills), laid out as the installer puts it under
# ~/.raea. Built from the committed tree at REF (never anyone's unfinished edit); the runtime's packages
# (sdk/react/js/node_modules, installed, not committed) come from this checkout.
#
#   tools/release.sh VERSION BIN_DIR [REF]     BIN_DIR holds a release build's raea and its bundler
#                                              (libraea_bundle.dylib, .so, or raea.exe and raea_bundle.dll), and on
#                                              macOS raea-perceive (`raea song` runs it; it isn't ported, so Linux's
#                                              and Windows's archives have no `raea song`)
#
# The public engine is free and needs no raea account: its raea is built without raea's API (engine/cli's service
# feature) and with the plugin catalog (raea/plugins' 56 Bevy plugins and raea/input's gamepad and picking):
#   cargo build --profile dist --locked -p raea-cli --bin raea --no-default-features \
#     --features "$(bash tools/bevy-plugins.sh features sdk/raea/bench/bevy-plugins)"
# That build's raea-cli (thin LTO) prints nothing for over 25 minutes, so under antsy run it beside
# tools/cpu-heartbeat.sh (a line while rustc's CPU grows) and keep `--stall 25m`; a raised stall would only wait longer
# on a real hang.
#
# Writes dist/raea-VERSION-macos-arm64.tar.gz (-linux-x86_64.tar.gz, or -windows-x86_64.zip) and its .sha256, then
# checks the archive with tools/release-check.sh: unpacked into a fresh folder, its raea renders a frame with
# RAEA_NO_TREE=1 (so it can't be reading this checkout), has the plugin catalog, names none of raea's API and holds
# nothing key-shaped (on Windows, no Microsoft file). Nothing is published.
set -eu
version=${1:?usage: release.sh VERSION BIN_DIR [REF]}
bin=$(cd "${2:?usage: release.sh VERSION BIN_DIR [REF]}" && pwd)
ref=${3:-HEAD}
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"
out=$root/dist
case $(uname -s) in
  Linux) platform=linux-x86_64 ;;
  MINGW*|MSYS*|CYGWIN*) platform=windows-x86_64 ;;
  *) platform=macos-arm64 ;;
esac
name=raea-$version-$platform
stage=$(mktemp -d)/raea
mkdir -p "$stage/bin" "$stage/share/raea" "$out"
case $platform in
  linux-x86_64)
    cp "$bin/raea" "$bin/libraea_bundle.so" "$stage/bin/"
    # what a Linux machine needs beside it: the bundled Noto Sans JP is a Latin subset, so kanji and kana come from the
    # system, as on a Mac (raea warns once a character when no font has it)
    printf '%s\n' "raea $version for Linux x86_64 (glibc 2.39 or newer, Vulkan: a GPU driver or Mesa's llvmpipe).
Install a CJK font for Japanese, Chinese and Korean text (Debian and Ubuntu: apt install fonts-noto-cjk).
Not in this build: raea song (its beat model's runner is macOS only so far)." > "$stage/README" ;;
  windows-x86_64)
    cp "$bin/raea.exe" "$bin/raea_bundle.dll" "$stage/bin/"
    # the C runtime isn't in the zip (ort's ONNX Runtime needs its DLLs, and they're Microsoft's to ship): the VC++
    # redistributable, which most Windows machines already have, provides it
    printf '%s\r\n' "raea $version for Windows x86_64 (Windows 10 or 11, a DirectX 12 or Vulkan GPU driver; with no GPU, Mesa's lavapipe and WGPU_BACKEND=vulkan)." \
      "It needs the Microsoft Visual C++ Redistributable (x64), which most machines already have:" \
      "https://aka.ms/vs/17/release/vc_redist.x64.exe" \
      "Not in this build: raea song (its beat model's runner is macOS only so far)." > "$stage/README.txt" ;;
  *) cp "$bin/raea" "$bin/libraea_bundle.dylib" "$bin/raea-perceive" "$stage/bin/" ;;
esac
# the packages sdk/tsconfig.base.json lists (@raea/<name>: its sources and package.json), which the bundlers resolve by it
pkgs=()
# and its bevy/ (a package's plugin handles, generated from the plugin build: its src/plugin.ts imports ../bevy/)
for p in $(git show "$ref:sdk/tsconfig.base.json" | jq -r '.compilerOptions.paths | to_entries[] | select(.key | test("^@raea/[a-z_]+$")) | .value[0] | ltrimstr("./") | split("/")[0]' | tr -d '\r'); do
  pkgs+=("sdk/$p/src" "sdk/$p/package.json")
  git cat-file -e "$ref:sdk/$p/bevy" 2>/dev/null && pkgs+=("sdk/$p/bevy")
done
[ ${#pkgs[@]} -gt 0 ] || { echo "release: no @raea packages in sdk/tsconfig.base.json at $ref" >&2; exit 1; }
# the committed files raea reads at run time
# (@raea/ui, the page's DOM every render goes through, isn't an @raea/<name> path, so it's named here)
git archive "$ref" sdk/tsconfig.base.json sdk/ui/src sdk/ui/package.json sdk/ui/bevy sdk/raea/src sdk/raea/library sdk/raea/package.json sdk/react/js/src sdk/react/js/lite sdk/react/js/shims sdk/react/js/package.json sdk/react/js/tsconfig.json sdk/shell/src \
  "${pkgs[@]}" \
  tools/tokensynth tools/sao tools/goo tools/splat tools/matchmove sdk/guides/craft docs/sdk-coverage.md engine/cli/template/theme.css ':(glob)engine/cli/src/*.ts' \
  | tar -x -C "$stage/share/raea"
# a folder copied as it is here (links kept, as rsync -a; tar, since Git's bash on Windows has no rsync)
copy() { mkdir -p "$2" && tar -C "$(dirname "$1")" --exclude .cache -cf - "$(basename "$1")" | tar -C "$2" -xf -; }
# styles.gen's Tailwind (raea compiles it on JavaScriptCore: runtime/bundle/src/styles.rs), as installed here
copy sdk/react/tw/node_modules/tailwindcss "$stage/share/raea/sdk/react/tw/node_modules"
# `raea song`'s beat model (Beat This! small0, MIT), copied into ~/.local/share/raea-models on first use
if [ $platform = macos-arm64 ]; then
  mkdir -p "$stage/share/raea/models/beat-this"
  cp ~/.local/share/raea-models/beat-this/beat_this_small0.onnx "$stage/share/raea/models/beat-this/"
fi
# the agent skills (sdk/skills: raea, raea-bevy), which the installer copies into ~/.claude/skills; raea-bevy's topic files
# are links to the craft guides, so they go as the files they point to
skills=$(mktemp -d)
git archive "$ref" sdk/skills sdk/guides | tar -x -C "$skills"
# raea-work is internal (work.raea.io's showcase pages), so it stays out, as the public export drops it
rm -rf "$skills/sdk/skills/raea-work"
cp -RL "$skills/sdk/skills" "$stage/share/raea/skills"
# the runtime's packages, as installed here
copy sdk/react/js/node_modules "$stage/share/raea/sdk/react/js"
rm -rf "$stage/share/raea/tools/tokensynth/__pycache__" "$skills"
printf '%s\n' "$version" > "$stage/share/raea/VERSION"
if [ $platform = windows-x86_64 ]; then
  archive=$name.zip
  rm -f "$out/$archive"
  (cd "$(dirname "$stage")" && 7z a -tzip -bso0 -bsp0 "$out/$archive" raea)
else
  archive=$name.tar.gz
  tar -C "$(dirname "$stage")" -czf "$out/$archive" raea
fi
(cd "$out" && if command -v sha256sum >/dev/null; then sha256sum "$archive"; else shasum -a 256 "$archive"; fi > "$archive.sha256")
rm -rf "$(dirname "$stage")"
bash "$root/tools/release-check.sh" "$out/$archive"
echo "release: $out/$archive ($(du -h "$out/$archive" | cut -f1))"
