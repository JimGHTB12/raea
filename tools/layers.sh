#!/bin/bash
# Dependencies point down (docs/overhaul.md, "Layers"). Reads each tracked Cargo.toml of ours, takes its normal and
# build dependencies on our own crates (dev-dependencies aren't checked: a test may use anything), and fails on an edge
# that points up, or sideways where the layer doesn't allow it (tools/layers.toml), and on a crate with no layer. The
# edges wrong today are listed in tools/layers.allow; that list only shrinks, and the split (phase 4) empties it.
#   tools/layers.sh            check
#   tools/layers.sh --edges    every edge of ours, "from (layer) -> to (layer)", wrong ones marked
#   tools/layers.sh --update   tools/layers.allow rewritten to today's wrong edges (refused if one is new)
set -u
ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT" || exit 2
MAP=tools/layers.toml
ALLOW=tools/layers.allow
ORDER="core io gpu systems runtime sdk apps tools"
SIDEWAYS="core io gpu runtime apps tools"

rank() { local i=0 l; for l in $ORDER; do [ "$l" = "$1" ] && { echo $i; return; }; i=$((i + 1)); done; echo -1; }
layer() { awk -F'"' -v c="$1" '/^\[crates\]/ {s=1; next} /^\[/ {s=0} s && $1 ~ "^"c" *= *$" {print $2}' "$MAP"; }
facility() { awk -v c="$1" '/^\[facility\]/ {s=1; next} /^\[/ {s=0} s && $1 == c {f=1} END {exit !f}' "$MAP"; }

# "name dir" per crate of ours, from the Cargo.toml files git tracks or would (as tools/size.sh)
crates() {
  git ls-files --cached --others --exclude-standard -- '*Cargo.toml' ':!vendor/*' ':!*/vendor/*' | while read -r f; do
    [ -f "$f" ] || continue
    n=$(awk -F'"' '/^\[package\]/ {p=1} p && /^name *=/ {print $2; exit}' "$f")
    [ -n "$n" ] && echo "$n $(dirname "$f")"
  done
}
# the dependency names a Cargo.toml declares outside [dev-dependencies]: `name = ...`, `name.workspace = true` and
# `[dependencies.name]`, under [dependencies], [build-dependencies] and [target.*.dependencies]
deps() {
  awk '
    /^\[/ { dep = ($0 ~ /^\[(target\..*\.)?(build-)?dependencies\]$/); one = ""
            if (match($0, /^\[(target\..*\.)?(build-)?dependencies\.[A-Za-z0-9_-]+\]$/)) { s = $0; sub(/^.*dependencies\./, "", s); sub(/\]$/, "", s); print s }
            next }
    dep && /^[A-Za-z0-9_-]+(\.workspace)? *=/ { s = $0; sub(/[ .=].*/, "", s); print s }
  ' "$1" | sort -u
}

edges() {
  local list ours
  list=$(crates)
  ours=" $(echo "$list" | awk '{print $1}' | tr '\n' ' ') "
  echo "$list" | while read -r name dir; do
    for d in $(deps "$dir/Cargo.toml"); do
      case "$ours" in *" $d "*) [ "$d" != "$name" ] && echo "$name $d" ;; esac
    done
  done | sort -u
}

# "from -> to: why" per wrong edge, and "name: no layer" per unmapped crate
wrong() {
  crates | while read -r name _; do [ -n "$(layer "$name")" ] || echo "$name: no layer in $MAP"; done
  edges | while read -r a b; do
    la=$(layer "$a"); lb=$(layer "$b")
    [ -n "$la" ] && [ -n "$lb" ] || continue
    ra=$(rank "$la"); rb=$(rank "$lb")
    if [ "$rb" -gt "$ra" ]; then echo "$a -> $b: up ($la -> $lb)"
    elif [ "$rb" -eq "$ra" ]; then
      case " $SIDEWAYS " in *" $la "*) ;; *) facility "$b" || echo "$a -> $b: sideways in $la ($b isn't a facility)" ;; esac
    fi
  done
}

if [ "${1:-}" = "--edges" ]; then
  bad=$(wrong)
  edges | while read -r a b; do
    m=$(echo "$bad" | grep -F "$a -> $b:" | sed 's/^[^:]*: /  WRONG: /')
    echo "$a ($(layer "$a")) -> $b ($(layer "$b"))$m"
  done
  exit 0
fi
now=$(wrong | sort)
if [ "${1:-}" = "--update" ]; then
  new=$(comm -23 <(echo "$now" | grep -v '^$') <(grep -v '^#' "$ALLOW" 2>/dev/null | sort))
  [ -z "$new" ] || [ ! -f "$ALLOW" ] || { echo "layers: refused, these are new:" >&2; echo "$new" >&2; exit 1; }
  { echo "# dependencies that break the layers today (tools/layers.sh): this list only shrinks; the split empties it"
    echo "$now"; } > "$ALLOW"
  echo "layers: $(echo "$now" | grep -c .) wrong today, $ALLOW updated"
  exit 0
fi
new=$(comm -23 <(echo "$now" | grep -v '^$') <(grep -v '^#' "$ALLOW" 2>/dev/null | sort))
if [ -n "$new" ]; then
  echo "layers: dependencies that break the layers (docs/overhaul.md):" >&2
  echo "$new" | sed 's/^/  /' >&2
  exit 1
fi
echo "layers: ok ($(echo "$now" | grep -c .) wrong edges left from before, none new)"
