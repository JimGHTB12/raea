#!/bin/zsh
# A release of the raea engine for macOS on Apple Silicon or for Linux on x86_64 (whichever machine runs it; Linux's
# is built on the probe box in an ubuntu:24.04 root, for glibc 2.39): raea and its bundler, with raea's own files beside
# them (share/raea: the raea SDK's TypeScript, the lite runtime and its packages, the shell's
# sources, the tools that install the local music and foley models, and the agent skills), laid out as
# tools/install.sh puts it under ~/.raea. Built from the committed tree at REF (never anyone's unfinished edit); the
# runtime's packages (sdk/react/js/node_modules, installed, not committed) come from this checkout.
#
#   tools/release.sh VERSION BIN_DIR [REF]     BIN_DIR holds a release build's raea and its bundler
#                                              (libraea_bundle.dylib or .so), and on macOS raea-perceive (`raea song`
#                                              runs it; it isn't ported, so Linux's tarball has no `raea song`)
#
# The public engine is free and needs no raea account: its raea is built without raea's API (engine/cli's service
# feature) and with the plugin catalog (raea/plugins' 56 Bevy plugins and raea/input's gamepad and picking):
#   cargo build --profile dist --locked -p raea-cli --bin raea --no-default-features \
#     --features "$(bash tools/bevy-plugins.sh features sdk/raea/bench/bevy-plugins)"
# The tarball's check opens avian and the virtual gamepad, so a raea without the catalog is refused. The check below refuses a build that names
# raea's API hosts or its token variables, and any file in the tarball that looks like a key. That build's raea-cli
# (thin LTO) prints nothing for over 25 minutes, so under antsy run it beside tools/cpu-heartbeat.sh (a line while
# rustc's CPU grows) and keep `--stall 25m`; a raised stall would only wait longer on a real hang.
#
# Writes dist/raea-VERSION-macos-arm64.tar.gz (or -linux-x86_64) and its .sha256, and checks the tarball: unpacked into a fresh folder, its
# raea renders a frame with RAEA_NO_TREE=1 (so it can't be reading this checkout). Nothing is published.
set -eu
version=${1:?usage: release.sh VERSION BIN_DIR [REF]}
bin=${2:?usage: release.sh VERSION BIN_DIR [REF]}
ref=${3:-HEAD}
root=${0:A:h}/..
cd $root
out=$root/dist
linux=; platform=macos-arm64
[ "$(uname -s)" = Linux ] && { linux=1; platform=linux-x86_64 }
name=raea-$version-$platform
stage=$(mktemp -d)/raea
mkdir -p $stage/bin $stage/share/raea $out
if [ -n "$linux" ]; then
  cp $bin/raea $bin/libraea_bundle.so $stage/bin/
  # what a Linux machine needs beside it: the bundled Noto Sans JP is a Latin subset, so kanji and kana come from the
  # system, as on a Mac (raea warns once a character when no font has it)
  print -r -- "raea $version for Linux x86_64 (glibc 2.39 or newer, Vulkan: a GPU driver or Mesa's llvmpipe).
Install a CJK font for Japanese, Chinese and Korean text (Debian and Ubuntu: apt install fonts-noto-cjk).
Not in this build: raea song (its beat model's runner is macOS only so far)." > $stage/README
else
  cp $bin/raea $bin/libraea_bundle.dylib $bin/raea-perceive $stage/bin/
fi
# the packages sdk/tsconfig.base.json lists (@raea/<name>: its sources and package.json), which the bundlers resolve by it
pkgs=()
# and its bevy/ (a package's plugin handles, generated from the plugin build: its src/plugin.ts imports ../bevy/)
for p in $(git show ${ref}:sdk/tsconfig.base.json | jq -r '.compilerOptions.paths | to_entries[] | select(.key | test("^@raea/[a-z_]+$")) | .value[0] | ltrimstr("./") | split("/")[0]'); do
  pkgs+=(sdk/$p/src sdk/$p/package.json)
  git cat-file -e ${ref}:sdk/$p/bevy 2>/dev/null && pkgs+=(sdk/$p/bevy)
done
[ ${#pkgs} -gt 0 ] || { echo "release: no @raea packages in sdk/tsconfig.base.json at $ref" >&2; exit 1 }
# the committed files raea reads at run time
# (@raea/ui, the page's DOM every render goes through, isn't an @raea/<name> path, so it's named here)
git archive $ref sdk/tsconfig.base.json sdk/ui/src sdk/ui/package.json sdk/ui/bevy sdk/raea/src sdk/raea/library sdk/raea/package.json sdk/react/js/src sdk/react/js/lite sdk/react/js/shims sdk/react/js/package.json sdk/react/js/tsconfig.json sdk/shell/src \
  $pkgs \
  tools/tokensynth tools/sao tools/goo tools/splat tools/matchmove sdk/guides/craft docs/sdk-coverage.md engine/cli/template/theme.css ':(glob)engine/cli/src/*.ts' \
  | tar -x -C $stage/share/raea
# styles.gen's Tailwind (raea compiles it on JavaScriptCore: runtime/bundle/src/styles.rs), as installed here
mkdir -p $stage/share/raea/sdk/react/tw/node_modules
rsync -a sdk/react/tw/node_modules/tailwindcss $stage/share/raea/sdk/react/tw/node_modules/
# `raea song`'s beat model (Beat This! small0, MIT), copied into ~/.local/share/raea-models on first use
if [ -z "$linux" ]; then
  mkdir -p $stage/share/raea/models/beat-this
  cp ~/.local/share/raea-models/beat-this/beat_this_small0.onnx $stage/share/raea/models/beat-this/
fi
# the agent skills (sdk/skills: raea, raea-bevy), which the installer copies into ~/.claude/skills; raea-bevy's topic files
# are links to the craft guides, so they go as the files they point to
skills=$(mktemp -d)
git archive $ref sdk/skills sdk/guides | tar -x -C $skills
# raea-work is internal (work.raea.io's showcase pages), so it stays out, as the public export drops it
rm -rf $skills/sdk/skills/raea-work
cp -RL $skills/sdk/skills $stage/share/raea/skills
# the runtime's packages, as installed here
mkdir -p $stage/share/raea/sdk/react/js && rsync -a --exclude .cache sdk/react/js/node_modules $stage/share/raea/sdk/react/js/
rm -rf $stage/share/raea/tools/tokensynth/__pycache__
print -r -- "$version" > $stage/share/raea/VERSION
tar -C ${stage:h} -czf $out/$name.tar.gz raea
(cd $out && if [ -n "$linux" ]; then sha256sum $name.tar.gz; else shasum -a 256 $name.tar.gz; fi > $name.tar.gz.sha256)
# the check: unpacked elsewhere, it renders a frame without the tree it was built from
check=$(mktemp -d)
tar -xzf $out/$name.tar.gz -C $check
# the edit in a folder of its own: raea reads every file under a project, and the install isn't the founder's project
mkdir -p $check/p
cat > $check/p/hello.tsx <<'TSX'
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
function Hello() {
  const f = useCurrentFrame();
  return <AbsoluteFill style={{ background: "#000", color: "#fff", fontSize: 120, justifyContent: "center", alignItems: "center" }}>raea {f}</AbsoluteFill>;
}
registerRoot(() => <Composition id="hello" component={Hello} width={640} height={360} fps={30} durationInFrames={30} />);
TSX
(cd $check/p && RAEA_NO_TREE=1 ../raea/bin/raea render hello.tsx --frame 12 -o hello.png >/dev/null 2>&1) || { echo "release: the unpacked raea didn't render (see $check)"; exit 1; }
[ -s $check/p/hello.png ] || { echo "release: no frame written (see $check)"; exit 1; }
# the plugin catalog is in it (track 3, raea/plugins): a headless App with avian and Bevy's input, raea/input's virtual
# gamepad pressed and read back. A raea built without --features "bevy-plugins,..." says "no plugin "avian" in this
# build" here
mkdir -p $check/plugins
cat > $check/plugins/check.ts <<'TS'
import { BevyApp } from "raea/bevy";
import { print } from "raea/native";
export default function main() {
  const app = BevyApp.open({ plugins: ["avian", "bevy"], fps: 60 });
  app.step(2);
  app.call("raea/input", { gamepad: "South" });
  app.step(2);
  const held = app.call<{ buttons: string[] }>("raea/input.gamepad");
  if (!held.buttons.includes("South")) throw new Error(`the virtual gamepad's South isn't held: ${JSON.stringify(held)}`);
  print("plugins: avian and the virtual gamepad");
}
TS
(cd $check/plugins && RAEA_NO_TREE=1 ../raea/bin/raea run check.ts 2>&1 | grep -q "plugins: avian and the virtual gamepad") || { echo "release: the unpacked raea has no plugin catalog (build raea with --features \"\$(tools/bevy-plugins.sh features sdk/raea/bench/bevy-plugins)\"; see $check/plugins)"; exit 1; }
# and its guides and styles.gen come from the release too
(cd $check/p && RAEA_NO_TREE=1 ../raea/bin/raea guide music | grep -q "music") || { echo "release: the unpacked raea has no craft guides (see $check)"; exit 1; }
mkdir -p $check/styled && { print -r -- 'import { css } from "./styles.gen";'; cat $check/p/hello.tsx; print -r -- 'void css;'; } > $check/styled/hello.tsx
(cd $check/styled && RAEA_NO_TREE=1 ../raea/bin/raea render hello.tsx --frame 1 -o hello.png >/dev/null 2>&1 && [ -s styles.gen.ts ]) || { echo "release: the unpacked raea didn't write styles.gen.ts (see $check/styled)"; exit 1; }
# engine only: no raea API in the binaries, nothing key-shaped anywhere
api=$(strings $check/raea/bin/* | grep -oE '(api|sign)\.raea\.io|RAEA_(SIGN|GEN)_TOKEN' | sort -u | tr '\n' ' ')
[ -z "$api" ] || { echo "release: the binaries name raea's API ($api): build raea-cli with --no-default-features"; exit 1; }
keys=$(grep -rlE 'sk-or-v1-[0-9a-f]{20}|sk-ant-[A-Za-z0-9_-]{20}|AKIA[0-9A-Z]{16}|BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY' $check/raea 2>/dev/null | head -5)
[ -z "$keys" ] || { echo "release: key-shaped text in: $keys"; exit 1; }
echo "release: $out/$name.tar.gz ($(du -h $out/$name.tar.gz | cut -f1)), checked: an unpacked copy rendered a frame with RAEA_NO_TREE=1"
rm -rf $check ${stage:h}
