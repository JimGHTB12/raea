#!/bin/bash
# The TypeScript-driven plugin proofs (sdk/raea/docs/bevy-plugins.md), each a headless program on the plugin build:
#   avian      avian/raycast.ts: rigid bodies and colliders from generated handles, avian3d/cast_ray and RayCaster,
#              a collision observed
#   lightyear  lightyear/peers.ts: two peers in lockstep, a replicated Position moving on both, input predicted
#   remote     remote/devtool.ts served over BRP's HTTP, driven here with curl (rpc.discover, world.query,
#              world.get_components, registry.schema, raea/stop)
#   catalog    every other declared plugin, one process each: opened headless from catalog/proof.ts and stepped,
#              driven and checked where it has a TypeScript API; a plugin that draws is proved in the render Engine
#              (its declaration's "proof" says why it can't open headless)
# Exits 1 on any failure. Build first: tools/bevy-plugins.sh build sdk/raea/bench/bevy-plugins
#   sdk/raea/bench/bevy-plugins/run.sh [avian|lightyear|remote|catalog ...]
set -u
HERE=$(cd "$(dirname "$0")" && pwd)
BIN="$HERE/motion.sh"
# the generated modules (package.json raea.generated): the SDK's raea/plugins
GEN=$(cd "$HERE/$(jq -r .raea.generated "$HERE/package.json")" && pwd)
OUT=${TMPDIR:-/tmp}/raea-bevy-run
mkdir -p "$OUT"
status=0
brp() { # PORT METHOD PARAMS-JSON
  curl -s -m 5 -X POST "http://127.0.0.1:$1" -H 'content-type: application/json' -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"$2\",\"params\":$3}"
}
for name in ${*:-avian lightyear remote catalog}; do
  case $name in
    avian) prog=avian/raycast.ts ;;
    lightyear) prog=lightyear/peers.ts ;;
    remote)
      port=15703
      "$BIN" run "$HERE/remote/devtool.ts" > "$OUT/remote.log" 2>&1 &
      pid=$!
      # the server is up when the program says so (it prints before its first step)
      for _ in $(seq 1 600); do grep -q '^serving' "$OUT/remote.log" && break; kill -0 $pid 2>/dev/null || break; sleep 0.1; done
      ok=1
      methods=$(brp "$port" rpc.discover null | jq -r '[.result.methods[].name] | length')
      [ "${methods:-0}" -gt 10 ] || ok=0
      rows=$(brp "$port" world.query '{"data":{"components":["avian3d::physics_transform::transform::Position"]},"filter":{"with":["avian3d::dynamics::rigid_body::RigidBody"]}}')
      y=$(jq -r '.result[0].components["avian3d::physics_transform::transform::Position"][1]' <<<"$rows")
      ent=$(jq -r '.result[0].entity' <<<"$rows")
      # it is falling while served: a second read is lower
      y2=$(brp "$port" world.get_components "{\"entity\":$ent,\"components\":[\"avian3d::physics_transform::transform::Position\"]}" | jq -r '.result.components["avian3d::physics_transform::transform::Position"][1]')
      awk -v a="$y" -v b="$y2" 'BEGIN { exit !(a + 0 > b + 0 && a + 0 < 20) }' || ok=0
      schema=$(brp "$port" registry.schema '{"with_crates":["avian3d"]}' | jq -r '.result | length')
      [ "${schema:-0}" -gt 50 ] || ok=0
      brp "$port" raea/stop null > /dev/null
      wait $pid || ok=0
      if [ $ok = 1 ]; then echo "remote: PASS  $methods methods over HTTP; the ball fell from y $y to $y2 between two reads; $schema avian types in registry.schema; $(grep '"proof"' "$OUT/remote.log")"
      else echo "remote: FAIL  methods=$methods y=$y y2=$y2 schema=$schema (see $OUT/remote.log)"; status=1; fi
      continue ;;
    catalog)
      # every declared plugin (a builtin only when it says what it needs headless: bevy's gamepads and picking): a
      # headless App opened with it from TypeScript and stepped (catalog/proof.ts drives the
      # ones with a TypeScript API and checks them); a plugin that draws can't open without a renderer, so its proof is
      # the render Engine every plugin joined (the generator starts it with all of them, and its types are what
      # raea/plugins/<name> counts) and check.sh's renders with the whole catalog in
      decl="$HERE/package.json"
      minor=$(sed -n 's/^bevy = { version = "=\([0-9]*\.[0-9]*\)\.[0-9]*".*/\1/p' "$HERE/../../../../engine/bevy-plugins/Cargo.toml" | head -1)
      pass=0; fail=0
      # http's proof asks a local server (bun), which echoes the path, method, x-proof header and body as JSON (404 for
      # /missing); stopped by its own PID when the catalog ends
      HTTP_PORT=$((20000 + RANDOM % 20000))
      P=$HTTP_PORT bun -e 'Bun.serve({ port: +process.env.P, fetch: async (r) => { const path = new URL(r.url).pathname; return Response.json({ path, method: r.method, proof: r.headers.get("x-proof"), body: await r.text() }, { status: path === "/missing" ? 404 : 200 }); } })' > "$OUT/http-server.log" 2>&1 &
      http_server=$!
      export RAEA_PROOF_HTTP="http://127.0.0.1:$HTTP_PORT"
      for p in $(jq -r --arg m "$minor" '((.raea.bevy.plugins // {}) + (.raea.plugins // {})) | to_entries[] | select(.value.versions[$m] != null) | select(((.value.builtin // false) | not) or .value.headless != null) | select(.key != "avian" and .key != "lightyear") | .key' "$decl"); do
        spec=$(jq -c --arg p "$p" '((.raea.bevy.plugins // {}) + (.raea.plugins // {}))[$p]' "$decl")
        counts=$(sed -n '2s/.*gen)\. \(.*\)\.$/\1/p' "$GEN/$p.gen.tsx" 2>/dev/null)
        if jq -e '.library and (.calls | not)' <<<"$spec" >/dev/null; then
          echo "catalog $p: PASS  library: compiled in, its reflected types generated (${counts:-no reflected types})"; pass=$((pass + 1)); continue
        fi
        if [ -n "$(jq -r '.proof // empty' <<<"$spec")" ]; then
          if [ -f "$GEN/$p.gen.tsx" ]; then echo "catalog $p: PASS  render Engine (with every plugin): ${counts:-its plugin added}; headless: $(jq -r .proof <<<"$spec")"; pass=$((pass + 1))
          else echo "catalog $p: FAIL  no raea/plugins/$p ($GEN/$p.gen.tsx): generate it"; fail=$((fail + 1)); status=1; fi
          continue
        fi
        if RAEA_PROOF=$p RAEA_PROOF_DIR="$HERE/catalog/data" "$BIN" run "$HERE/catalog/proof.ts" > "$OUT/catalog-$p.log" 2>&1; then
          echo "catalog $p: PASS  $(grep '"proof"' "$OUT/catalog-$p.log" | cut -c1-400)"; pass=$((pass + 1))
        else
          echo "catalog $p: FAIL  $(grep -m1 -E 'FAIL|panicked|error' "$OUT/catalog-$p.log" | cut -c1-300) (see $OUT/catalog-$p.log)"; fail=$((fail + 1)); status=1
        fi
      done
      kill $http_server 2>/dev/null
      echo "catalog: $pass passed, $fail failed"
      continue ;;
    *) echo "unknown proof $name"; status=1; continue ;;
  esac
  if "$BIN" run "$HERE/$prog" > "$OUT/$name.log" 2>&1; then
    echo "$name: PASS  $(grep '"proof"' "$OUT/$name.log")"
  else
    echo "$name: FAIL (see $OUT/$name.log)"; tail -5 "$OUT/$name.log"; status=1
  fi
done
exit $status
