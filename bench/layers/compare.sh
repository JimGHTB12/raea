#!/bin/bash
# The layer pass against its oracle (docs/review.feature R2e): each layer's mask in NAME.layers (`raea render --layers`
# of bench/layers/conformance.tsx at the oracle's moments) against where hiding it changed the frame (oracle.sh), as
# IoU, at each moment. A layer's mask is its own pixels and those of every layer inside it (a hide hides those too).
# A layer passes at IoU >= MIN (0.95) at every moment where either mask has pixels; exits 1 when one fails.
#   bench/layers/compare.sh ORACLE NAME.layers [MIN]
ORACLE=${1:?ORACLE: oracle.sh OUT folder}
LAYERS=${2:?NAME.layers}
MIN=${3:-0.95}
[ -f "$ORACLE/keys.txt" ] || { echo "no $ORACLE/keys.txt: run bench/layers/oracle.sh first"; exit 2; }
gunzip -c "$LAYERS" | perl "$(dirname "$0")/compare.pl" "$ORACLE" "$MIN"
