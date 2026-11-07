#!/bin/bash
# Dependency scan (C2PA Generator Product Security Requirements O.3/O.4, Level 1): osv-scanner over the lockfiles of
# everything that makes or signs an export, CRITICAL and HIGH advisories (CVSS 7 and up) listed with when raea first
# saw each. Fails when one has gone unfixed more than 90 days after it was first seen.
#   tools/sca.sh            scan and report (CI's gate); a finding not yet recorded counts as seen today
#   tools/sca.sh --record   also add new findings to the record (io/provenance/conformance/sca-seen.tsv)
# Needs osv-scanner (brew install osv-scanner) and jq.
set -eu
cd "$(dirname "$0")/.."
SEEN=io/provenance/conformance/sca-seen.tsv
DAYS=90
LOCKS=(Cargo.lock engine/sandbox/wsx/Cargo.lock apps/site/api/pnpm-lock.yaml)
record=0; [ "${1:-}" = --record ] && record=1
command -v osv-scanner >/dev/null || { echo "sca: no osv-scanner (brew install osv-scanner)" >&2; exit 2; }

args=(); for l in "${LOCKS[@]}"; do [ -f "$l" ] && args+=(--lockfile "$l"); done
out=$(mktemp); trap 'rm -f "$out"' EXIT
# exit 1 means it found something, which is the normal case; anything else is the scanner failing
osv-scanner scan source "${args[@]}" --format json > "$out" 2>/dev/null || [ $? = 1 ] || { echo "sca: osv-scanner failed" >&2; exit 2; }

today=$(date +%F); now=$(date +%s); late=0; new=0
[ -f "$SEEN" ] || printf 'first_seen\tid\tpackage\tversion\tseverity\n' > "$SEEN"
while IFS=$'\t' read -r id pkg ver sev; do
  first=$(awk -F'\t' -v i="$id" -v p="$pkg" -v v="$ver" '$2==i && $3==p && $4==v {print $1; exit}' "$SEEN")
  if [ -z "$first" ]; then
    first=$today; new=$((new + 1))
    [ $record = 1 ] && printf '%s\t%s\t%s\t%s\t%s\n' "$today" "$id" "$pkg" "$ver" "$sev" >> "$SEEN"
  fi
  age=$(( (now - $(date -j -f %F "$first" +%s)) / 86400 ))
  state=open; [ $age -gt $DAYS ] && { state=LATE; late=$((late + 1)); }
  printf '%-5s %-22s %-28s %-10s cvss %-4s seen %s (%d days)\n' "$state" "$id" "$pkg" "$ver" "$sev" "$first" "$age"
done < <(jq -r '.results[].packages[] | .package as $p | .groups[]
  | select((.max_severity | tonumber? // 0) >= 7)
  | [(.ids | sort | .[0]), $p.name, $p.version, .max_severity] | @tsv' "$out" | sort -u)

echo "sca: $late past $DAYS days, $new not yet recorded$([ $record = 1 ] && echo ' (recorded)')"
[ $late = 0 ]
