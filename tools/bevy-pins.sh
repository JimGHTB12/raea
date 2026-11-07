#!/bin/bash
# Everything a Bevy bump has to move, listed from the tree (docs/bevy-upgrade.md): the Bevy, wgpu and plugin pins in
# every Cargo.toml, the [patch] entries for them, the versions each lockfile resolved (more than one Bevy or wgpu in a
# lock is flagged), the vendored crates raea changed (their "# raea:" header names the upstream base), and the Bevy
# shaders raea swaps at load (the upstream hash each was taken from). Exits 1 when a lock mixes Bevy or wgpu versions,
# but for the known mixes in KNOWN (lockfile, crate, version, why), which draw on a device of their own, and when Bevy's
# glam has scalar-math or libm on.
#   tools/bevy-pins.sh
set -u
ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT" || exit 2
CRATES='bevy|bevy_ecs|bevy_clipboard|wgpu|avian3d|bevy_hanabi|bevy_tweening|bevy_seedling|firewheel|firewheel-core|firewheel-graph|firewheel-macros|firewheel-nodes'
KNOWN='Cargo.lock wgpu 29.0.4 pixels 0.17 (dioxus-native ios-sim window renderer, its own device; not built on macOS)'
manifests() { git ls-files '*Cargo.toml' | grep -v '/vendor/' ; }
locks() { git ls-files '*Cargo.lock' | grep -v '/vendor/' ; }

echo "== pins (dependencies)"
for m in $(manifests); do
  awk -v m="$m" -v re="^($CRATES)[ ]*=" '
    /^\[patch/ { patch = 1; next } /^\[/ { patch = 0 }
    !patch && $0 ~ re && $0 !~ /= \[/ { sub(/^[ ]+/, ""); printf "%-45s %s\n", m, $0 }' "$m"
done

echo
echo "== patches"
for m in $(manifests); do
  awk -v m="$m" -v re="^($CRATES|wgpu-core)[ ]*=" '
    /^\[patch/ { patch = 1; next } /^\[/ { patch = 0 }
    patch && $0 ~ re { printf "%-45s %s\n", m, $0 }' "$m"
done

echo
echo "== resolved versions by lockfile"
status=0
for l in $(locks); do
  line=""
  for c in bevy bevy_ecs wgpu wgpu-core avian3d bevy_hanabi bevy_tweening bevy_seedling firewheel glam; do
    v=$(awk -v c="$c" '$0 == "name = \"" c "\"" { getline; gsub(/version = |"/, ""); print }' "$l" | sort -u | tr '\n' ' ')
    [ -n "$v" ] || continue
    line="$line $c ${v% }"
    n=$(echo "$v" | wc -w | tr -d ' ')
    if [ "$n" -gt 1 ]; then case $c in bevy|bevy_ecs|wgpu)
      known=""
      for ver in $v; do echo "$KNOWN" | grep -q "^$l $c $ver " && known="$known $ver"; done
      if [ "$(echo $known | wc -w | tr -d ' ')" -ge $((n - 1)) ]; then line="$line (mixed, known:$known)"; else line="$line (MIXED)"; status=1; fi ;;
    esac; fi
    line="$line;"
  done
  [ -n "$line" ] && printf "%-45s%s\n" "$l" "$line"
done

echo
# glam's scalar-math or libm on Bevy's own glam (feature unification: at 0.20 rapier's enhanced-determinism turned
# both on through parry3d and glamx) drops Vec4's 16-byte alignment, and Bevy's GPU structs stop matching its shaders
echo "== Bevy's glam: features that break Bevy's GPU structs"
gv=$(awk '$0 == "name = \"bevy_math\"" { m = 1 } m && /"glam / { gsub(/[" ,]/, ""); sub(/glam/, ""); print; exit }' Cargo.lock)
bad=$(cargo tree --offline --workspace -e features -i "glam@$gv" 2>/dev/null | grep -o 'glam feature "\(scalar-math\|libm\|fast-math\)"' | sort -u | tr '\n' ' ')
if [ -n "$bad" ]; then echo "glam $gv: $bad(BAD: cargo tree -e features -i glam@$gv shows who)"; status=1; else echo "glam $gv: none"; fi

echo
echo "== vendored crates raea changed (upstream base in the header; changes marked raea:)"
for m in vendor/*/Cargo.toml; do
  h=$(head -1 "$m")
  case $h in "# raea:"*) printf "%-60s %s (%s raea: marks)\n" "$(dirname "$m")" "${h#\# raea: }" "$(grep -rI 'raea:' "$(dirname "$m")" | wc -l | tr -d ' ')" ;; esac
done

echo
echo "== Bevy shaders swapped at load (engine/render, bevy_api::swap_embedded_shader)"
grep -rn 'swap_embedded_shader(app, "' engine/render/src | sed -E 's/^([^:]+):([0-9]+):.*swap_embedded_shader\(app, "([^"]+)", (0x[0-9a-f_]+).*/\1:\2  \3  upstream hash \4/'

echo
echo "== toolchain"
bv=$(awk '$0 == "name = \"bevy\"" { getline; gsub(/version = |"/, ""); print; exit }' Cargo.lock)
rv=$(grep -h '^rust-version' "$HOME"/.cargo/registry/src/*/bevy-"$bv"/Cargo.toml 2>/dev/null | head -1)
echo "bevy $bv: ${rv:-rust-version unknown (crate not in the local registry)}; rustc on PATH: $(rustc --version 2>/dev/null)"
exit $status
