#!/bin/bash
# #44: text-autospace's initial value is no-autospace, as Chrome's is (raea had normal: digits next to Hangul or CJK
# got 1/8 em gaps Chrome doesn't draw). index.html draws "2026년" at 48 px three times: unstyled, with
# text-autospace: normal, and with no-autospace. The unstyled line's ink must end where the no-autospace line's does,
# and the normal line's 6 px (1/8 em) further right, as Chrome 148 measures them (148.31, 154.31, 148.31 px).
# Exits 1 on a failure.
#   bench/text-autospace/check.sh      (RAEA_BIN: the raea to run, default target/release/raea)
cd "$(dirname "$0")/../.." || exit 2
BIN=${RAEA_BIN:-$PWD/target/release/raea}
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-text-autospace.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
"$BIN" render bench/text-autospace/index.html -o "$OUT/a.png" > "$OUT/log" 2>&1 || { echo "FAIL render (log below)"; sed 's/^/    /' "$OUT/log"; exit 1; }
# the x just past a line's last ink (line n: y 80n to 80n+80)
end() { magick "$OUT/a.png" -crop "x80+0+$((80 * $1))" +repage -threshold 50% -negate -trim -format '%[fx:page.x+w]' info:; }
unstyled=$(end 0) normal=$(end 1) none=$(end 2)
status=0
[ "$unstyled" = "$none" ] || { echo "FAIL the unstyled line ends at x $unstyled, the no-autospace one at $none (Chrome: the same)"; status=1; }
gap=$((normal - none))
(( gap >= 5 && gap <= 7 )) || { echo "FAIL text-autospace: normal ends $gap px past no-autospace, wanted 6 (1/8 em)"; status=1; }
[ $status = 0 ] && echo "PASS text-autospace (ends at $unstyled, $normal, $none)"
exit $status
