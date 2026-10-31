#!/bin/bash
# The oracle's browser, pinned: Chrome for Testing's chrome-headless-shell (no window, ever), fetched once into
# ~/Library/Caches/raea/ui-parity/chrome/<version>/ and its path printed. Bumping the pin is a change here, and the
# corpus's numbers move with it.
#   sdk/ui/parity/chrome.sh        prints the binary's path (downloading it first if it isn't there)
set -euo pipefail
VERSION=152.0.7977.30
case "$(uname -s)-$(uname -m)" in
  Darwin-arm64) PLAT=mac-arm64 ;;
  Darwin-x86_64) PLAT=mac-x64 ;;
  Linux-x86_64) PLAT=linux64 ;;
  *) echo "chrome.sh: no Chrome for Testing build for $(uname -s)-$(uname -m)" >&2; exit 2 ;;
esac
ROOT=${RAEA_UI_PARITY_CHROME_DIR:-$HOME/Library/Caches/raea/ui-parity/chrome}/$VERSION
BIN=$ROOT/chrome-headless-shell-$PLAT/chrome-headless-shell
if [ ! -x "$BIN" ]; then
  free=$(df -g "$HOME" | awk 'NR==2 {print $4}')
  [ "${free:-0}" -ge 2 ] || { echo "chrome.sh: under 2 GB free; not downloading" >&2; exit 2; }
  mkdir -p "$ROOT"
  zip=$ROOT/download.zip
  curl -fsSL -o "$zip" "https://storage.googleapis.com/chrome-for-testing-public/$VERSION/$PLAT/chrome-headless-shell-$PLAT.zip" >&2
  (cd "$ROOT" && unzip -q -o "$zip") >&2
  rm -f "$zip"
  # downloaded binaries carry the quarantine flag; Gatekeeper would stop a headless launch on it
  xattr -dr com.apple.quarantine "$ROOT" 2>/dev/null || true
fi
echo "$BIN"
