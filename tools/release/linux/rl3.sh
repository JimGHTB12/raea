#!/bin/bash
# RL3 on Linux, in a fresh debian:trixie container (the cloud node's base): the tarball unpacked, run by a new user
# with an empty HOME and no network, as linux-check.sh's clean() does on the probe box. PKGS = the runtime packages.
set -uo pipefail
V=${1:?version}; shift
export DEBIAN_FRONTEND=noninteractive
[ $# = 0 ] || apt-get update -qq >/dev/null && apt-get install -y -qq --no-install-recommends "$@" >/dev/null || { echo "rl3: apt failed"; exit 1; }
R=/tmp/rl3; mkdir -p $R/home $R/p $R/game $R/bevy
tar -xzf /out/raea-$V-linux-x86_64.tar.gz -C $R && cp -R /rel/sdk/react/bench/games/pong/. $R/game/
bin=$(ls -d $R/raea*/bin 2>/dev/null | head -1); [ -x "$bin/raea" ] || bin=$R/raea/bin
echo "rl3 missing libs: $(ldd $bin/raea $bin/libraea_bundle.so | grep -c 'not found')"; ldd $bin/raea | grep 'not found'
useradd -m fresh; chown -R fresh $R
ok=0
step() {  # NAME DIR ARGS...
  local n=$1 d=$2 t0=$(date +%s); shift 2
  (cd "$d" && su fresh -c "env -i PATH=/usr/bin:/bin HOME=$R/home RAEA_NO_TREE=1 RAEA_API=http://127.0.0.1:9 RAEA_GEN_API=http://127.0.0.1:9 $bin/raea $(printf '%q ' "$@")") > $R/$n.log 2>&1
  local c=$?; echo "rl3 $n: exit $c, $(( $(date +%s) - t0 ))s"; grep -v -E 'ALSA|^\s*$' $R/$n.log | tail -2 | cut -c1-160; [ $c = 0 ] || ok=1
}
step init $R/p init
step render-frame $R/p render edit.tsx --frame 15 -o card.png
step render-mp4 $R/p render edit.tsx -o edit.mp4
step play $R/game play edit.tsx --offscreen --seconds 2 --size 540x960 --snapshot 1 snap.png
printf '%s\n' 'import { BevyApp } from "raea/bevy";' 'import { print } from "raea/native";' 'export default function main() {' \
  '  const app = BevyApp.open({ plugins: ["avian", "bevy"] });' '  const frame = app.step(30);' '  app.close();' \
  '  if (!(frame >= 30)) throw new Error(`stepped to ${frame}`);' '  print(`bevy ok: frame ${frame}`);' '}' > $R/bevy/main.ts && chown -R fresh $R/bevy
step bevy $R/bevy run main.ts
ls -la $R/p/card.png $R/p/edit.mp4 $R/game/snap.png 2>&1 | awk '{print $5, $NF}'
echo "rl3 home left: $(cd $R/home && find . -maxdepth 3 -type d | sort | tr '\n' ' ')"
echo "rl3: $([ $ok = 0 ] && echo ok || echo FAILED)"
