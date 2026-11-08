#!/bin/bash
# #34: a render says once, when it ends, what its page didn't apply, and its output is otherwise free of warnings no
# one acts on. page.tsx (30 frames; its outline's style attribute parsed again each frame) and index.html each draw
# outline text (-webkit-text-stroke, which stylo drops) with a font face whose file isn't there. Each render must exit
# 0, print one NOTE naming -webkit-text-stroke and the missing file once each, and print no INFO line and no bevy_asset
# or fontdb warning; page.tsx's --manifest must list them under not_applied. Exits 1 on a failure.
#   bench/dropped-css/check.sh      (RAEA_BIN: the raea to run, default target/release/raea)
cd "$(dirname "$0")/../.." || exit 2
BIN=${RAEA_BIN:-$PWD/target/release/raea}
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-dropped-css.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
status=0
check() { # NAME LOG RC
  local name=$1 log=$2 rc=$3 bad=""
  [ "$rc" = 0 ] || bad+=" exit $rc;"
  [ "$(grep -c '^NOTE  the page didn.t apply' "$log")" = 1 ] || bad+=" not one NOTE;"
  [ "$(grep -c -- '-webkit-text-stroke in ' "$log")" = 1 ] || bad+=" -webkit-text-stroke not listed once;"
  [ "$(grep -c 'not-here.woff2 (it didn.t load)' "$log")" = 1 ] || bad+=" the missing font not listed once;"
  ! grep -qE 'INFO|WARN (bevy_asset|fontdb)|\[WARN fontdb' "$log" || bad+=" boilerplate printed;"
  if [ -z "$bad" ]; then echo "PASS $name"; else echo "FAIL $name:$bad (log below)"; sed 's/^/    /' "$log"; status=1; fi
}
"$BIN" render bench/dropped-css/page.tsx -o "$OUT/page.mp4" --manifest "$OUT/m.json" > "$OUT/page.log" 2>&1
check page.tsx "$OUT/page.log" $?
listed=$(grep -o '"not_applied":\[[^]]*\]' "$OUT/m.json" 2>/dev/null)
if [[ $listed == *-webkit-text-stroke* && $listed == *not-here.woff2* ]]; then echo "PASS page.tsx manifest: $listed"
else echo "FAIL page.tsx manifest: not_applied is ${listed:-missing}"; status=1; fi
"$BIN" render bench/dropped-css/index.html -o "$OUT/html.mp4" > "$OUT/html.log" 2>&1
check index.html "$OUT/html.log" $?
exit $status
