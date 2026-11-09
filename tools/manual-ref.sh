#!/usr/bin/env bash
# The manual's generated references (raea.io/docs/reference/cli/ and /sdk/): `gen` rewrites their generated blocks,
# `check` fails when one is out of date, a flag has no description, or a verb or import path is in no group.
#   tools/manual-ref.sh gen|check      (RAEA_BIN names the raea whose `help --json` the CLI pages come from)
set -euo pipefail
cd "$(dirname "$0")/.."
mode="${1:-check}"
bun tools/manual-ref/cli.ts "$mode"
bun tools/manual-ref/sdk.ts "$mode"
