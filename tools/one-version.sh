#!/bin/bash
# One version of each crate the page and Bevy share (docs/blitz-bevy.feature B0b): exits 1 when a tracked Cargo.lock
# resolves two versions of stylo, taffy, parley, fontique, wgpu, vello or anyrender, naming each and its versions.
# Two would mean two text engines, two layout engines or two GPU APIs in one process, and a page Bevy can't draw on
# its own device. A lock that doesn't resolve as locked (its workspace broken, or the lock out of date) fails too,
# since what it says can't be trusted. The check reads; it never writes a lock.
#   tools/one-version.sh
set -u
ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT" || exit 2
CRATES='stylo taffy parley fontique wgpu vello anyrender'
bad=0
for lock in $(git ls-files '*Cargo.lock' | grep -vE '(^|/)vendor/'); do
  if ! cargo metadata --format-version 1 --offline --locked --manifest-path "$(dirname "$lock")/Cargo.toml" >/dev/null 2>&1; then
    echo "one-version: $lock doesn't resolve as locked (a path dependency gone, or the lock out of date)"
    bad=1
    continue
  fi
  for c in $CRATES; do
    vs=$(awk -v n="$c" '$0 == "name = \"" n "\"" { getline; gsub(/"/, "", $3); print $3 }' "$lock" | sort -u | tr '\n' ' ')
    if [ "$(echo "$vs" | wc -w)" -gt 1 ]; then
      echo "one-version: $lock: $c $vs"
      bad=1
    fi
  done
done
[ $bad = 0 ] && echo "one-version: ok ($CRATES)"
exit $bad
