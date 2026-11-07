#!/bin/bash
# A tool raea and the feed run that a clean Mac doesn't have (docs/app.feature AP6), its path printed, installed first
# if it isn't there: one place, so `raea cut` (engine/cli/src/native.ts), the feed's host (apps/app host/tools.ts) and
# raea.app ask the same way. Each install is pinned and its download's sha256 checked, under $RAEA_TOOLS_CACHE (else
# ~/.cache/raea-tools)/NAME, once; RAEA_AUDIO_AUTOSETUP=0 (raea's one switch for local tools installing themselves)
# refuses instead. A working one already on PATH is used as it is.
#   tools/ensure.sh git            git with git-lfs (GitHub Desktop's portable build, dugite-native: git 2.56.0, lfs 3.8.1)
#   tools/ensure.sh parakeet-mlx   the take transcriber (parakeet-mlx 0.5.2 in its own venv; its model loads on first use)
#   tools/ensure.sh uv             uv 0.12.24 (Astral's standalone build), what the Python installs are made with
# Progress goes to stderr; stdout is only the path.
set -euo pipefail
name=${1:?usage: tools/ensure.sh git|parakeet-mlx|uv}
cache=${RAEA_TOOLS_CACHE:-$HOME/.cache/raea-tools}

UV_VER=0.12.24
UV_SHA=0c4346de7abdb49495b393b9ec809fe387aa43e586be20fecb972216c1e71732
GIT_TAG=v2.56.0-2
GIT_FILE=dugite-native-v2.56.0-a44aeca-macOS-arm64.tar.gz
GIT_SHA=1110de343de11cc6f469b782b60f36217b9199b5e06875dd65baa26dd60563c7
PARAKEET_VER=0.5.2

say() { echo "raea: $*" >&2; }
refuse() { say "$name isn't installed, and RAEA_AUDIO_AUTOSETUP=0 keeps it from installing itself (into $cache/$name)"; exit 1; }

# a download, its sha256 checked, at $2
fetch() {
  local url=$1 out=$2 sha=$3
  curl -fL --retry 3 --silent --show-error -o "$out.part" "$url"
  [ "$(shasum -a 256 "$out.part" | cut -d' ' -f1)" = "$sha" ] || { rm -f "$out.part"; say "$url: its sha256 isn't $sha"; exit 1; }
  mv "$out.part" "$out"
}

# one install at a time per tool (two renders asking at once): a lock directory, released on exit
lock() {
  mkdir -p "$cache"
  local l="$cache/.$1.lock" n=0
  until mkdir "$l" 2>/dev/null; do
    n=$((n + 1))
    [ $n -gt 1200 ] && { say "$l: another install has held it for 10 minutes (remove it if none is running)"; exit 1; }
    sleep 0.5
  done
  trap "rmdir '$l' 2>/dev/null" EXIT
}

# git on PATH, if it works without asking for anything: /usr/bin/git is a stub that opens the Command Line Tools
# installer when they're missing, so it counts only when xcode-select has a developer directory
path_git() {
  local g
  g=$(command -v git 2>/dev/null) || return 1
  if [ "$g" = /usr/bin/git ]; then xcode-select -p >/dev/null 2>&1 || return 1; fi
  "$g" --version >/dev/null 2>&1 && "$g" lfs version >/dev/null 2>&1 || return 1
  echo "$g"
}

ensure_uv() {
  local d=$cache/uv
  if [ -x "$d/uv" ]; then echo "$d/uv"; return; fi
  local u
  if u=$(command -v uv 2>/dev/null); then echo "$u"; return; fi
  [ "${RAEA_AUDIO_AUTOSETUP:-1}" = 0 ] && refuse
  lock uv
  if [ ! -x "$d/uv" ]; then
    say "installing uv $UV_VER into $d"
    mkdir -p "$d.part"
    fetch "https://github.com/astral-sh/uv/releases/download/$UV_VER/uv-aarch64-apple-darwin.tar.gz" "$d.part/uv.tgz" $UV_SHA
    tar xzf "$d.part/uv.tgz" -C "$d.part" --strip-components 1
    rm "$d.part/uv.tgz"
    rm -rf "$d" && mv "$d.part" "$d"
  fi
  echo "$d/uv"
}

ensure_git() {
  local d=$cache/git
  if [ -x "$d/raea-git" ]; then echo "$d/raea-git"; return; fi
  path_git && return
  [ "${RAEA_AUDIO_AUTOSETUP:-1}" = 0 ] && refuse
  lock git
  if [ ! -x "$d/raea-git" ]; then
    say "installing git (dugite-native $GIT_TAG, with git-lfs, 150 MB) into $d"
    rm -rf "$d.part" && mkdir -p "$d.part"
    fetch "https://github.com/desktop/dugite-native/releases/download/$GIT_TAG/$GIT_FILE" "$d.part/git.tgz" $GIT_SHA
    tar xzf "$d.part/git.tgz" -C "$d.part"
    rm "$d.part/git.tgz"
    # its programs found where it is, as GitHub Desktop runs it (git-lfs is one of them, so the LFS filter works)
    cat >"$d.part/raea-git" <<'EOF'
#!/bin/sh
d=$(cd "$(dirname "$0")" && pwd)
export GIT_EXEC_PATH="$d/libexec/git-core" GIT_TEMPLATE_DIR="$d/share/git-core/templates" PATH="$d/libexec/git-core:$PATH"
exec "$d/bin/git" "$@"
EOF
    chmod +x "$d.part/raea-git"
    "$d.part/raea-git" --version >/dev/null && "$d.part/raea-git" lfs version >/dev/null || { say "the installed git doesn't run ($d.part)"; exit 1; }
    rm -rf "$d" && mv "$d.part" "$d"
  fi
  echo "$d/raea-git"
}

ensure_parakeet() {
  local d=$cache/parakeet-mlx
  if [ -f "$d/.ready" ]; then echo "$d/parakeet-mlx"; return; fi
  local p
  if p=$(command -v parakeet-mlx 2>/dev/null); then echo "$p"; return; fi
  [ "${RAEA_AUDIO_AUTOSETUP:-1}" = 0 ] && refuse
  local uv
  uv=$(ensure_uv)
  lock parakeet-mlx
  if [ ! -f "$d/.ready" ]; then
    say "installing parakeet-mlx $PARAKEET_VER into $d (its own Python, from uv)"
    rm -rf "$d" && mkdir -p "$d"
    # uv's Python and its cache kept beside it, not in ~/.local
    export UV_PYTHON_INSTALL_DIR=$cache/uv/python UV_CACHE_DIR=$cache/uv/cache
    "$uv" venv -q --python 3.12 --python-preference only-managed "$d/.venv" >&2
    VIRTUAL_ENV="$d/.venv" "$uv" pip install -q --python "$d/.venv/bin/python" "parakeet-mlx==$PARAKEET_VER" >&2
    # parakeet-mlx reads its audio through ffmpeg (-f s16le -ac 1 -ar RATE -), which a clean Mac doesn't have. What
    # raea hands it is already a mono 16 kHz wav (raea media audio --rate 16000 --mono), so this stand-in answers
    # that one call from the wav itself, PCM 16 or float 32, and refuses anything it would have to resample
    mkdir -p "$d/shim"
    cat >"$d/shim/ffmpeg" <<SHIM
#!$d/.venv/bin/python
import struct, sys
a = sys.argv[1:]
src = a[a.index("-i") + 1]
rate = int(a[a.index("-ar") + 1])
b = open(src, "rb").read()
if b[:4] != b"RIFF" or b[8:12] != b"WAVE":
    sys.exit(f"raea's ffmpeg stand-in reads only a wav, not {src}: make one with raea media audio --rate {rate} --mono")
at, fmt, data = 12, None, None
while at + 8 <= len(b):
    kind, n = b[at:at + 4], struct.unpack("<I", b[at + 4:at + 8])[0]
    if kind == b"fmt ": fmt = struct.unpack("<HHIIHH", b[at + 8:at + 24])
    elif kind == b"data": data = b[at + 8:at + 8 + n]
    at += 8 + n + (n & 1)
code, ch, sr, _, _, bits = fmt
if sr != rate: sys.exit(f"{src} is {sr} Hz, not {rate}: make it with raea media audio --rate {rate} --mono")
if code == 1 and bits == 16: s = struct.unpack(f"<{len(data) // 2}h", data)
elif code == 3 and bits == 32: s = [max(-32768, min(32767, round(v * 32767))) for v in struct.unpack(f"<{len(data) // 4}f", data)]
else: sys.exit(f"{src}: a wav of format {code}, {bits}-bit, isn't read here")
if ch > 1: s = [round(sum(s[i:i + ch]) / ch) for i in range(0, len(s) - ch + 1, ch)]
sys.stdout.buffer.write(struct.pack(f"<{len(s)}h", *s))
SHIM
    chmod +x "$d/shim/ffmpeg"
    cat >"$d/parakeet-mlx" <<WRAP
#!/bin/sh
PATH="$d/shim:\$PATH" exec "$d/.venv/bin/parakeet-mlx" "\$@"
WRAP
    chmod +x "$d/parakeet-mlx"
    "$d/parakeet-mlx" --help >/dev/null || { say "the installed parakeet-mlx doesn't run"; exit 1; }
    # the downloads uv kept are done with (the venv has its own copies)
    rm -rf "$cache/uv/cache"
    touch "$d/.ready"
  fi
  echo "$d/parakeet-mlx"
}

case $name in
  git) ensure_git ;;
  uv) ensure_uv ;;
  parakeet-mlx) ensure_parakeet ;;
  *) say "no tool $name (git, parakeet-mlx, uv)"; exit 2 ;;
esac
