#!/bin/bash
# The WPT reftest subset's tests (bench/wpt/run.sh), pinned: web-platform-tests at one commit, only the areas run and
# what their pages load (fonts/, the css support folders, resources/), fetched once into
# ~/Library/Caches/raea/wpt/<commit>/ and its path printed. Bumping the pin is a change here, and the pass rates move
# with it.
#   bench/wpt/fetch.sh      prints the checkout's path (fetching it first if it isn't there)
set -euo pipefail
COMMIT=5af3c135c9c97381733866150a7ada27d1029c50
DIRS=(css/css-flexbox css/css-grid css/css-text css/css-backgrounds css/css-borders css/css-transforms css/css-fonts css/css-masking css/compositing css/support css/reference fonts resources/ahem images common)
ROOT=${RAEA_WPT_DIR:-$HOME/Library/Caches/raea/wpt}/$COMMIT
if [ ! -e "$ROOT/.fetched" ]; then
  free=$(df -g "$HOME" | awk 'NR==2 {print $4}')
  [ "${free:-0}" -ge 3 ] || { echo "fetch.sh: under 3 GB free; not fetching" >&2; exit 2; }
  rm -rf "$ROOT" && mkdir -p "$ROOT"
  git -C "$ROOT" init -q
  git -C "$ROOT" remote add origin https://github.com/web-platform-tests/wpt.git
  git -C "$ROOT" sparse-checkout set --no-cone "${DIRS[@]/#//}" >&2
  git -C "$ROOT" fetch -q --depth 1 --filter=blob:none origin "$COMMIT" >&2
  git -C "$ROOT" checkout -q FETCH_HEAD >&2
  : > "$ROOT/.fetched"
fi
# a checkout from before a folder joined DIRS gets it now (common/: reftest-wait.js's takeScreenshot)
for d in "${DIRS[@]}"; do [ -e "$ROOT/$d" ] || git -C "$ROOT" sparse-checkout add "/$d" >&2; done
echo "$ROOT"
