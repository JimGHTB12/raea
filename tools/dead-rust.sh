#!/bin/bash
# Dead Rust items in one crate (overhaul phase 1, docs/handoff/audit.md). Every `pub` in the crate's library (or its
# binary, for a crate with no library) is narrowed to `pub(crate)`, the crate is checked, and each dead_code or
# unused_imports warning is an item nothing in the crate uses. Each name is then looked for, as a whole word, in the
# crate's own bins, tests, examples and benches and in the crates that depend on it, plus TS for a field (serde fields
# reach TS by name). No hit outside means dead; a hit means used, or a common name (new, run) that grep can't tell
# apart, so read it. Items found used keep `pub` and the check repeats until no new one turns up, so nothing only a
# used item reaches is called dead. The crate is restored afterwards, even on failure. It edits files, so it refuses
# the shared tree.
#   tools/dead-rust.sh <crate> [cargo args]   run from a private checkout (~/raea-trial/<x>); prints a TSV:
#                                             verdict, kind, name, file:line, hits outside (up to 3 files)
# Default features only: an item used only under a feature shows as dead; pass --all-features or --features to check.
# The library alone: an item only its unit tests use shows as dead; `--profile test` checks with them, to tell the two
# apart (not --tests, which also builds the integration tests and bins against the narrowed library).
set -u
crate=${1:?usage: tools/dead-rust.sh <crate> [cargo args]}
shift
root=$(git rev-parse --show-toplevel 2>/dev/null) || { echo "dead-rust: run it inside a checkout" >&2; exit 2; }
shared=$(cd "${RAEA:-$HOME/raea}" 2>/dev/null && pwd -P)
[ "$(cd "$root" && pwd -P)" != "$shared" ] || { echo "dead-rust: refused in the shared tree $shared; use a private checkout" >&2; exit 2; }
cd "$root" || exit 2

meta=$(cargo metadata --no-deps --format-version 1) || exit 2
pkg=$(echo "$meta" | jq -c --arg n "$crate" '.packages[] | select(.name == $n)')
[ -n "$pkg" ] || { echo "dead-rust: no crate $crate in this workspace" >&2; exit 2; }
dir=$(dirname "$(echo "$pkg" | jq -r .manifest_path)")
# the library's sources, or the bins' when there is no library: everything under src/ except src/bin, and not a nested
# crate's
if echo "$pkg" | jq -e '[.targets[].kind[]] | any(. == "lib" or . == "rlib" or . == "cdylib")' >/dev/null; then
  target=(--lib); skip_bin=1
else
  target=(--bins); skip_bin=0
fi
nested=$(echo "$meta" | jq -r '.packages[].manifest_path' | xargs -n1 dirname | grep "^$dir/." || true)
src=()
while IFS= read -r f; do
  case "$f" in "$dir"/src/bin/*) [ "$skip_bin" = 1 ] && continue ;; esac
  [ "$skip_bin" = 1 ] && [ "$f" = "$dir/src/main.rs" ] && continue
  inner=0
  for n in $nested; do case "$f" in "$n"/*) inner=1 ;; esac; done
  [ "$inner" = 0 ] && src+=("$f")
done < <(find "$dir/src" -name '*.rs' 2>/dev/null | sort)
[ ${#src[@]} -gt 0 ] || { echo "dead-rust: $crate has no sources under $dir/src" >&2; exit 2; }

work=$(mktemp -d "${TMPDIR:-/tmp}/dead-rust.XXXXXX")
child=
restore() {
  [ -z "$child" ] || kill "$child" 2>/dev/null
  for f in "${src[@]}"; do cp -p "$work/orig/${f//\//%}" "$f"; done
  rm -rf "$work"
}
mkdir -p "$work/orig"
for f in "${src[@]}"; do cp -p "$f" "$work/orig/${f//\//%}"; done
trap restore EXIT
trap 'exit 130' INT TERM

# names used outside the narrowed sources: every tracked .rs, .ts, .tsx, .js outside vendor and node_modules
# .rs only in this crate and the crates that depend on it (a file belongs to the deepest crate folder holding it); a
# crate that doesn't depend on this one can't use its items, so its matches are only the same word
echo "$meta" | jq -r '.packages[].manifest_path' | xargs -n1 dirname | awk '{print length, $0}' | sort -rn | cut -d' ' -f2- > "$work/dirs"
{ echo "$dir"
  echo "$meta" | jq -r --arg n "$crate" '.packages[] | select(any(.dependencies[]; .name == $n)) | .manifest_path' | xargs -n1 dirname
} > "$work/users"
git ls-files -- '*.rs' '*.ts' '*.tsx' '*.js' '*.mjs' ':!vendor/*' ':!*/vendor/*' ':!**/node_modules/**' | sed "s|^|$root/|" |
  awk -v dirs="$work/dirs" -v users="$work/users" '
    BEGIN { while ((getline d < dirs) > 0) D[++n] = d; while ((getline u < users) > 0) U[u] = 1 }
    !/\.rs$/ { print; next }
    { for (i = 1; i <= n; i++) if (index($0, D[i] "/") == 1) { if (D[i] in U) print; next } }' | sort > "$work/all"
printf '%s\n' "${src[@]}" | sort > "$work/mine"
comm -23 "$work/all" "$work/mine" > "$work/outside"
grep '\.rs$' "$work/outside" > "$work/outside.rs"

# one pass: `pub ` at the start of an item or field narrowed to pub(crate) (never `pub mod`: a kept item in a narrowed
# module is still private, so it and all it reaches would stay dead), except the lines in $work/keep (file:line,
# the items found used outside in an earlier pass), then the crate checked; dead.tsv gets kind, name, file:line per
# dead span in the crate's own files
narrow_check() {
  for f in "${src[@]}"; do
    awk -v f="${f#"$root"/}" -v keep="$work/keep" '
      BEGIN { while ((getline k < keep) > 0) K[k] = 1 }
      !((f ":" FNR) in K) && /^[[:space:]]*pub [^(]/ && !/^[[:space:]]*pub mod / { sub(/pub /, "pub(crate) ") } { print }' "$work/orig/${f//\//%}" > "$f"
  done
  lane build -- cargo check -j 6 -p "$crate" "${target[@]}" --message-format=json "$@" > "$work/check.raw" 2> "$work/check.err" &
  child=$!; wait "$child"; code=$?; child=
  grep "^{" "$work/check.raw" > "$work/check.json"
  if [ "$code" != 0 ] && ! grep -q '"reason":"compiler-message"' "$work/check.json"; then
    echo "dead-rust: cargo check failed ($code)" >&2; tail -20 "$work/check.err" >&2; exit 1
  fi
}
pass() {
  narrow_check "$@"
  # a narrowing that breaks the build (a kept `pub use` of a narrowed item: E0364, E0365; a narrowed type in a kept
  # signature) gets its item's pub back: each line an error points at that this script narrowed, and each item an
  # error names in backticks whose definition it narrowed. Then the same pass again; errors it can't place fail the run,
  # since rustc runs no lints after an error and the warnings would be meaningless.
  while jq -e 'select(.reason == "compiler-message" and .message.level == "error")' "$work/check.json" >/dev/null; do
    jq -r --arg r "$root/" 'select(.reason == "compiler-message" and .message.level == "error") | .message
      | (.spans[], .children[].spans[]) | "\(.file_name | ltrimstr($r)):\(.line_start)"' "$work/check.json" | sort -u > "$work/errlines"
    jq -r 'select(.reason == "compiler-message" and .message.level == "error") | .message
      | (.message, .children[].message) | scan("`([A-Za-z_][A-Za-z0-9_]*)`") | .[0]' "$work/check.json" | sort -u > "$work/errnames"
    : > "$work/fix"
    for f in "${src[@]}"; do
      rel=${f#"$root"/}
      awk -v f="$rel" -v lines="$work/errlines" -v names="$work/errnames" '
        BEGIN { while ((getline l < lines) > 0) L[l] = 1; while ((getline n < names) > 0) N[n] = 1 }
        /^[[:space:]]*pub\(crate\) / {
          if ((f ":" FNR) in L) { print f ":" FNR; next }
          if (match($0, /(struct|enum|union|trait|type|fn|const|static|mod|use) +[A-Za-z_][A-Za-z0-9_]*/)) {
            w = substr($0, RSTART, RLENGTH); sub(/.* +/, "", w); if (w in N) print f ":" FNR
          } else if (match($0, /pub\(crate\) +[A-Za-z_][A-Za-z0-9_]* *:/)) {
            w = substr($0, RSTART, RLENGTH); sub(/^pub\(crate\) +/, "", w); sub(/ *:$/, "", w); if (w in N) print f ":" FNR
          }
        }' "$f" > "$work/fix.one"
      # only lines this script narrowed (the original said `pub `)
      while IFS=: read -r _ line; do
        sed -n "${line}p" "$work/orig/${f//\//%}" | grep -qE '^[[:space:]]*pub [^(]' && echo "$rel:$line"
      done < "$work/fix.one" >> "$work/fix"
    done
    sort -u "$work/fix" | comm -23 - <(sort -u "$work/keep") > "$work/fix.new"
    if [ ! -s "$work/fix.new" ]; then
      echo "dead-rust: the narrowed crate doesn't build, and its errors name nothing this script narrowed:" >&2
      jq -r 'select(.reason == "compiler-message" and .message.level == "error") | .message.rendered' "$work/check.json" | head -40 >&2
      exit 1
    fi
    cat "$work/fix.new" >> "$work/keep"
    narrow_check "$@"
  done
  jq -r --arg d "$dir/" --arg r "$root/" '
    select(.reason == "compiler-message") | .message
    | select(.code.code == "dead_code" or .code.code == "unused_imports")
    | .message as $m
    | (($m | capture("^(?<k>[a-z ]+?)s? `") // ($m | capture("^multiple (?<k>[a-z ]+?)s are")) // {k: "item"}) | .k) as $k
    | .spans[] | select(.is_primary)
    | (if (.file_name | startswith("/")) then .file_name else $r + .file_name end) as $f
    | select($f | startswith($d))
    | [ (if .text[0] then .text[0].text[.text[0].highlight_start - 1 : .text[0].highlight_end - 1] else "?" end),
        ($m | if test("^unused import") then "import" else $k end),
        (($f | ltrimstr($r)) + ":" + (.line_start | tostring)) ]
    | @tsv' "$work/check.json" | sort -u > "$work/dead.tsv"
}

# each warned item's verdict: dead (no hit outside), used? (hits, up to 3 files) or read (no name to look for)
judge() {
  local name kind where word list hits
  while IFS=$'\t' read -r name kind where; do
    # an import's span is the whole path; the name is its last segment
    word=$(echo "$name" | sed -E 's/.*:://; s/[{} ]//g; s/^r#//')
    if [ -z "$word" ] || [ "$word" = "?" ] || [ "$word" = "*" ]; then
      printf 'read\t%s\t%s\t%s\t-\n' "$kind" "$name" "$where"; continue
    fi
    # TS can only reach a field (by its serde name); everything else is looked for in Rust
    case "$kind" in field*) list="$work/outside" ;; *) list="$work/outside.rs" ;; esac
    hits=$(tr '\n' '\0' < "$list" | xargs -0 grep -lw -- "$word" 2>/dev/null | sed "s|^$root/||")
    if [ -z "$hits" ]; then
      printf 'dead\t%s\t%s\t%s\t-\n' "$kind" "$name" "$where"
    else
      printf 'used?\t%s\t%s\t%s\t%s\n' "$kind" "$name" "$where" "$(echo "$hits" | head -3 | paste -sd, -)$( [ "$(echo "$hits" | wc -l)" -gt 3 ] && echo ",…")"
    fi
  done < "$work/dead.tsv"
}

# Narrowing everything at once makes rustc's dead_code spread: an entry point used only from outside looks unused, and
# so does everything only it reaches. So repeat: keep `pub` on each item found used outside, check again, until a pass
# finds none new. What's dead then is dead with every used item public.
: > "$work/keep"; : > "$work/used"; n=0
while :; do
  n=$((n + 1))
  pass "$@"
  judge > "$work/verdicts"
  grep '^used?' "$work/verdicts" | cut -f4 | sort -u | comm -23 - <(sort -u "$work/keep") > "$work/new"
  grep '^used?' "$work/verdicts" >> "$work/used"
  [ -s "$work/new" ] || break
  cat "$work/new" >> "$work/keep"
done
echo "dead-rust: $crate, $n passes" >&2

# an item only an exported macro uses is used by whoever calls the macro, which this can't see
grep -l '#\[macro_export\]' "${src[@]}" 2>/dev/null | sed "s|^$root/|dead-rust: exported macros in |; s|$|: a 'dead' item its macros use isn't|" >&2
printf 'verdict\tkind\tname\twhere\toutside\n'
{ sort -u -t$'\t' -k4,4 "$work/used"; grep -v '^used?' "$work/verdicts"; } | sort -t$'\t' -k1,1r -k4,4
