# P26 baseline: what capture.sh and compare.sh share. Needs HERE (this folder), ROOT (the checkout) and WORK (a scratch
# folder of the caller's).

# The file a composition renders from (src), its folder (dir) and its slug: the composition itself, or a copy of its
# folder with props/<slug>/ laid over it.
p26_src() {
  local rel=$1
  # "file.tsx#ID": one composition of a file that registers several (a film's cut), rendered with --composition ID
  comp=""
  copt=()
  if [[ "$rel" == *"#"* ]]; then
    comp=${rel#*#}
    rel=${rel%%#*}
    copt=(--composition "$comp")
  fi
  src=$ROOT/$rel
  slug=$(echo "$rel" | sed 's#/#__#g; s#\.tsx$##')${comp:+@$comp}
  # props/<slug>/ laid over a copy of its folder (@ROOT@ in those files is the checkout): a project file that imports
  # what its tool writes (the template's edit.tsx and cuts.ts), or a component that takes props rendered through
  # wrap.tsx, which gives it its documented example props
  if [ -d "$HERE/props/$slug" ]; then
    rm -rf "$WORK/proj" && mkdir -p "$WORK/proj"
    find "$(dirname "$src")" -maxdepth 1 -type f -exec cp {} "$WORK/proj/" \;
    for f in "$HERE/props/$slug"/*; do sed "s#@ROOT@#$ROOT#g" "$f" > "$WORK/proj/$(basename "$f")"; done
    src=$WORK/proj/$(basename "$src")
    # a part's example: the brand kit its styles.gen is compiled from (props/brand.css as the project's brand/brand.css)
    if [ -f "$WORK/proj/wrap.tsx" ]; then
      src=$WORK/proj/wrap.tsx
      mkdir -p "$WORK/proj/brand" && cp "$HERE/props/brand.css" "$WORK/proj/brand/brand.css"
      # a stub older than everything, so the probe can import it and the render compiles it
      echo 'export const css = "";' > "$WORK/proj/styles.gen.ts" && touch -t 200001010000 "$WORK/proj/styles.gen.ts"
    fi
  fi
  dir=$(dirname "$src")
}
