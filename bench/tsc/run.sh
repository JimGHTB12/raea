#!/bin/bash
# S8a's tsc (docs/sdk.feature): the SDK's library sources (sdk/*/src) and the committed compositions (content/) typed
# by the SDK umbrella's TypeScript, with bun's own types (sdk/raea/tsconfig.check.json; the packages' tsconfig.json is
# unchanged). A ratchet: bench/tsc/known.txt lists the errors there were when it started, each as file, code and
# message (no line or column, so an edit above one doesn't move it). Exits 1 on an error not in the list; one fixed
# is printed so the list can shrink.
#
#   bench/tsc/run.sh            check
#   bench/tsc/run.sh --accept   rewrite known.txt to this run's errors (only to shrink it, or with the owner's say)
set -uo pipefail
# one collation for sort and comm, here and in known.txt: under a locale's, comm can misread a sorted list, and a known
# error reads as both fixed and new
export LC_ALL=C
cd "$(dirname "$0")/../.."
CHECK=sdk/raea/tsconfig.check.json
TSC=$(dirname "$CHECK")/node_modules/.bin/tsc
KNOWN=bench/tsc/known.txt
[ -x "$TSC" ] || { echo "FAIL tsc: $TSC missing (bun install in $(dirname "$CHECK"))"; exit 1; }
out=$(mktemp -d)
trap 'rm -rf "$out"' EXIT
"$TSC" -p "$CHECK" > "$out/raw" 2>&1
# "path(line,col): error TSn: message" -> "path: TSn: message", paths from the repo root; a message's own lines kept as one
sed -nE 's|^(\.\./)*([^(]+)\([0-9]+,[0-9]+\): error (TS[0-9]+): (.*)$|\2: \3: \4|p' "$out/raw" \
  | sed -E 's|^content/|content/|; s|^([a-z]+/src/)|sdk/\1|' | sort > "$out/now"
if [ "${1:-}" = "--accept" ]; then
  cp "$out/now" "$KNOWN"
  echo "tsc: known.txt now $(wc -l < "$KNOWN" | tr -d ' ') errors"
  exit 0
fi
[ -f "$KNOWN" ] || { echo "FAIL tsc: no $KNOWN (run with --accept once)"; exit 1; }
comm -13 "$KNOWN" "$out/now" > "$out/new"
comm -23 "$KNOWN" "$out/now" > "$out/gone"
if [ -s "$out/gone" ]; then
  echo "tsc: $(wc -l < "$out/gone" | tr -d ' ') known errors fixed (--accept to drop them):"
  sed 's/^/  /' "$out/gone"
fi
if [ -s "$out/new" ]; then
  echo "FAIL tsc: $(wc -l < "$out/new" | tr -d ' ') new errors:"
  sed 's/^/  /' "$out/new"
  exit 1
fi
echo "PASS tsc: $(wc -l < "$out/now" | tr -d ' ') errors, all known"
