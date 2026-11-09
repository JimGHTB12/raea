#!/usr/bin/env bash
# The manual's generated references (raea.io/docs/reference/cli/ and /sdk/): `gen` rewrites their generated blocks,
# `check` fails when one is out of date, a flag, prop, field or member has no description, or a verb or import path is
# in no group.
#   tools/manual-ref.sh gen|check      (RAEA_BIN names the raea whose `help --json` the CLI pages come from;
#                                       RAEA_MODULES the checkout whose node_modules an export without them uses)
set -euo pipefail
cd "$(dirname "$0")/.."
root=$PWD
mode="${1:-check}"
mods=${RAEA_MODULES:-$root}
if [ "$mods" != "$root" ]; then
  # an export (tools/agents/check --lane): the SDK's imports resolve through the checkout's node_modules, linked in
  # where missing, as tools/context-examples.sh links them
  for m in "$mods"/sdk/*/node_modules; do
    d=$root/${m#"$mods"/}
    [ -d "$(dirname "$d")" ] || continue  # a package the export hasn't got (a private one)
    [ -e "$d" ] || { rm -f "$d"; ln -s "$m" "$d"; }
  done
fi
bun tools/manual-ref/cli.ts "$mode"
bun tools/manual-ref/sdk.ts "$mode"
