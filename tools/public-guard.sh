#!/usr/bin/env bash
# What goes into raea's public repository is permanent, so every commit is checked before it lands, not only releases
# (docs/public-first.feature PF3). This script is the one check; hooks, the `but` wrapper and CI all call it.
#
#   tools/public-guard.sh msg "MESSAGE"     a commit message
#   tools/public-guard.sh range A..B        each commit's message and the lines it adds, then gitleaks over the range
#   tools/public-guard.sh staged            the lines staged for a plain `git commit`
#   tools/public-guard.sh install           add the checks to this checkout's git hooks (pre-commit, commit-msg, pre-push)
#
# The patterns: the generic ones below (a home folder's path, an agent's scratch path, an agent session's name), plus
# one regular expression per line from a private names file when there is one (client and product names stay out of
# the public tree, so their list does too): $RAEA_GUARD_NAMES, else private/guard/names.txt, else
# tools/agents/guard-names.txt. Lines starting with # are comments. Matching ignores case, except for a names line
# starting with case: (a name that is also an ordinary word in lower case). Exit 1 lists every hit.
set -euo pipefail
root=$(git rev-parse --show-toplevel)

generic=(
  '(^|[^A-Za-z0-9_.-])/Users/[A-Za-z0-9._-]+/' '(^|[^A-Za-z0-9_.-])/home/[a-z][a-z0-9_-]*/'   # a person's home folder (examples use ~ or $HOME)
  '/private/tmp/claude-[0-9]+' '[.]claude/projects/'     # an agent's scratch and transcript paths
  '\braea-[0-9a-f]{2}\b' '\borgmode-[a-z0-9]{2}\b'      # agent session names
)
# allowed: example home folders, the credits for what raea ports (NOTICE, ported headers), and licence boilerplate. Written
# split ('Hyper''Frames') so this line doesn't match the names list itself.
allow='/Users/(a|you|me|name|user|USER|runner)/|/home/(you|user|runner|USER)/|PERFORMANCE OF'
allow+='|Portions ported from Hyper''Frames \(Apache-2\.0|ports? from Hyper''Frames|github\.com/heygen-com/hyper''frames'

names_file() {
  for f in "${RAEA_GUARD_NAMES:-}" "$root/private/guard/names.txt" "$root/tools/agents/guard-names.txt"; do
    [[ -n $f && -f $f ]] && { echo "$f"; return; }
  done
}

names() { local f; f=$(names_file) || true; [[ -n ${f:-} ]] && grep -v -E '^\s*(#|$)' "$f" || true; }
# one alternation each: the case-insensitive patterns, and the case: ones (a pattern that matches nothing when empty)
pat_i=$( { printf '%s\n' "${generic[@]}"; { names | grep -v '^case:' || true; }; } | paste -sd '|' -)
pat_c=$(names | sed -n 's/^case://p' | paste -sd '|' -); pat_c=${pat_c:-'a^'}

# hits in stdin's text, each line prefixed with $1
scan() {
  local where=$1 t; t=$(cat)
  { grep -n -i -E -- "$pat_i" <<<"$t"; grep -n -E -- "$pat_c" <<<"$t"; } | sort -t: -k1,1n -u | grep -v -E -- "$allow" | sed "s|^|$where:|" || true
}

hits=0
report() { [[ -z $1 ]] && return; printf '%s\n' "$1" >&2; hits=1; }

case ${1:-} in
  msg)
    report "$(printf '%s\n' "${2:-}" | scan "message")"
    ;;
  range)
    range=${2:?usage: public-guard.sh range A..B}
    for c in $(git rev-list --no-merges "$range"); do
      report "$(git log -1 --format=%B "$c" | scan "$(git rev-parse --short "$c") message")"
      report "$(git show --format= --no-color --unified=0 "$c" | grep -E '^\+[^+]' | cut -c2- | scan "$(git rev-parse --short "$c") adds")"
    done
    if command -v gitleaks >/dev/null; then
      gitleaks git --no-banner --redact --log-opts="$range" "$root" >&2 || { echo "public-guard: gitleaks found secrets in $range" >&2; hits=1; }
    else
      echo "public-guard: gitleaks isn't installed; secrets weren't checked (brew install gitleaks)" >&2
      hits=1
    fi
    ;;
  staged)
    report "$(git diff --cached --no-color --unified=0 | grep -E '^\+[^+]' | cut -c2- | scan "staged")"
    ;;
  files)
    # NUL-separated paths on stdin (git ls-files -z): every line of each text file
    list=$(mktemp); cat > "$list"
    report "$( { xargs -0 grep -n -i -I -E -- "$pat_i" < "$list"; xargs -0 grep -n -I -E -- "$pat_c" < "$list"; } 2>/dev/null | sort -u | grep -v -E -- "$allow" || true)"
    # a symlink's target is what git stores for it (grep reads the file it points to instead)
    report "$(tr '\0' '\n' < "$list" | while IFS= read -r f; do [[ -L $f ]] && printf '%s -> %s\n' "$f" "$(readlink "$f")"; done | scan "symlink")"
    rm -f "$list"
    ;;
  install)
    # only this checkout's own hooks: a core.hooksPath outside its .git (a global one) would put the checks into
    # every repository on the machine, where tools/public-guard.sh doesn't exist and every commit would fail
    gitdir=$(cd "$(git rev-parse --git-common-dir)" && pwd -P)
    hooks=$(git config --local core.hooksPath || git rev-parse --git-path hooks)
    [[ $hooks = /* ]] || hooks="$root/$hooks"
    mkdir -p "$hooks"
    hooks=$(cd "$hooks" && pwd -P)
    if [[ $hooks != "$gitdir"/* ]] || [[ -z $(git config --local core.hooksPath || true) && -n $(git config --global core.hooksPath || true) ]]; then
      echo "public-guard: this checkout's hooks resolve to $hooks, outside its .git (or a global core.hooksPath applies)." >&2
      echo "Give it hooks of its own first: git config --local core.hooksPath .git/hooks-raea (copy the global ones in), then install again." >&2
      exit 1
    fi
    add() { # hook, line: put right after the shebang, so it runs before the rest (and before anything reads stdin)
      local h="$hooks/$1"
      [[ -f $h ]] || printf '#!/bin/sh\n' >"$h"
      if ! grep -q 'public-guard.sh' "$h"; then
        { head -1 "$h"; printf '%s\n' "$2"; tail -n +2 "$h"; } >"$h.pg" && mv "$h.pg" "$h"
      fi
      chmod +x "$h"
    }
    # GitButler keeps the user's pre-commit as pre-commit-user and calls it first
    pre=pre-commit; [[ -f $hooks/pre-commit ]] && grep -q GITBUTLER_MANAGED_HOOK "$hooks/pre-commit" && pre=pre-commit-user
    add "$pre" '"$(git rev-parse --show-toplevel)/tools/public-guard.sh" staged || exit 1'
    add commit-msg '"$(git rev-parse --show-toplevel)/tools/public-guard.sh" msg "$(cat "$1")" || exit 1'
    # pre-push gets "<local ref> <local sha> <remote ref> <remote sha>" lines on stdin, which the rest of the hook (Git
    # LFS) reads too: kept in a file and handed back. A new branch is checked from where it leaves origin/main.
    add pre-push 'pg_in=$(mktemp); cat >"$pg_in"; z=0000000000000000000000000000000000000000; while read -r _ l _ r; do [ "$l" = "$z" ] && continue; b=$r; [ "$r" = "$z" ] && b=$(git merge-base "$l" origin/main 2>/dev/null || echo "$l~1"); "$(git rev-parse --show-toplevel)/tools/public-guard.sh" range "$b..$l" || exit 1; done <"$pg_in"; exec <"$pg_in"'
    echo "public-guard: installed into $hooks (pre-commit, commit-msg, pre-push)"
    ;;
  *)
    sed -n '2,13p' "$0" | sed 's/^# \{0,1\}//' >&2
    exit 2
    ;;
esac
exit $hits
