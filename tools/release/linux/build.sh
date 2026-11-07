#!/bin/bash
# In the container (Dockerfile here): a Linux release from /rel, a clone at the release SHA, with /target a folder of its
# own: raea with the plugin catalog and the two app hosts (targets.feature T1f), each in a cargo call, stripped, then
# tools/release.sh under Xvfb (its release check opens the web target's window, which needs a display on Linux). Run
# release.sh with bash: zsh reads `$ref:apps` as a path modifier. The skills gate reads /rel/apps/app (clone the
# feed app there at the commit the release SHA records). A virtiofs /target holds deleted files until the VM stops.
set -uo pipefail
V=${1:?version}; jobs=${2:-8}
export CARGO_TARGET_DIR=/target CARGO_INCREMENTAL=0
git config --global --add safe.directory /rel
bash /hb.sh /target 60 &
cd /rel
for d in sdk/react/js sdk/react/tw; do (cd "$d" && bun install --frozen-lockfile 2>&1 | tail -1); done
feat=$(bash tools/bevy-plugins.sh features sdk/raea/bench/bevy-plugins) && [ -n "$feat" ] || { echo "linux: no catalog features"; exit 1; }
b() { cargo build --release --locked -j "$jobs" "$@" 2>&1 | tee -a /target/c3-build.log | grep --line-buffered -E '^ *Compiling|^error|Finished'; grep -q '^ *Finished' <(tail -3 /target/c3-build.log) || { grep -E '^error' -A8 /target/c3-build.log | head -40; echo "linux: build FAILED: $*"; exit 1; }; }
: > /target/c3-build.log
b -p raea-cli --bin raea --no-default-features --features "$feat"
b -p raea-rune-host
b -p raea-page-host --features window
r=/target/release; echo "linux build: $(objdump -T $r/raea | grep -o 'GLIBC_[0-9.]*' | sort -uV | tail -1)"
ship=/target/ship; rm -rf "${ship:?}"; mkdir -p "$ship"
cp $r/raea $r/libraea_bundle.so $r/raea-rune-host $r/raea-page-host "$ship/" && strip "$ship/raea" "$ship/raea-rune-host" "$ship/raea-page-host" \
  && strip --strip-unneeded "$ship/libraea_bundle.so" || { echo "linux: strip failed"; exit 1; }
xvfb-run -a -s "-screen 0 1280x1024x24" bash tools/release.sh "$V" "$ship" HEAD || { echo "linux: release.sh FAILED"; exit 1; }
cp dist/raea-$V-linux-x86_64.tar.gz* /out/ && ls -l /out && echo "linux: done"
