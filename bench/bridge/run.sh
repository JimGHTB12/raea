#!/bin/bash
# B3 H11 (docs/blitz-bevy.feature): the bridge's bench, a test per hook and per scenario. Prints one line each: PASS,
# FAIL, or GAP (not built yet, with what's missing; a gap doesn't fail the run). Exits 1 on any FAIL.
#
#   bench/bridge/run.sh [--pixels]
#
# Always: raea-page's lib tests (bridge.rs, pick.rs, keys.rs and scenarios.rs, H11's scenarios), raea-runtime-react's shared.rs
# test, and sdk/raea's conformance/hooks.test.tsx under bun. --pixels adds the live Apps, on the GPU: shared_state
# (H4 end to end: React's write reaches the World, React shows the World's values) and plain_page against raea's
# render of the same frame on the Blitz page (H5's overlay; H10: the title area at SSIM 1.000000), page_layers (H5's
# world and surface modes, read by their pixels), video.tsx in plain_page (a <video> in a plain App's page, its area
# against raea's), screen.tsx in plain_page (a <Screen>'s nested composition on its screen in the 3D) and the layers
# fixture in plain_page (a layered Canvas3D: its 3D between the page's elements), and element_camera (H7: a camera's
# texture in an element) and texture_slot (an <img data-raea-texture> showing the App's PageTextures), planes (H5's 3D planes: sorted, cut at w = 0), click_make (H3 live: a click's new element has its twin in that event) page_replay (H10: a page played live, recorded and rendered from the log) and widget.tsx on the plugin build (S8b: a
# Blitz widget crate declared in raea.plugins, drawn; PLUGINS= names the build, a GAP without one). They need
# target/release/raea; the examples are built through lane.
set -uo pipefail
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
cd "$ROOT"
# renders compared by their pixels are no exports: unsigned and unwatermarked (a TrustMark id differs per render; CI sets it too)
export RAEA_SIGN=0
PIXELS=0
[ "${1:-}" = "--pixels" ] && PIXELS=1
WORK=$(mktemp -d "${TMPDIR:-/tmp}/bridge-bench.XXXXXX")
# kept when a line fails, so a CI run's renders and logs stay as evidence (CI's logs name it)
trap 'if [ "$FAILS" -gt 0 ]; then echo "bridge: $FAILS failed; renders and logs kept in $WORK"; else rm -rf "$WORK"; fi' EXIT
FAILS=0

line() { printf '%-5s %-4s %s\n' "$1" "$2" "$3"; [ "$2" = FAIL ] && FAILS=$((FAILS + 1)); return 0; }
# PASS when every named test passed in the log
tests() {
  local hook=$1 what=$2 log=$3; shift 3
  for t in "$@"; do
    grep -q "^test $t \.\.\. ok" "$log" || { line "$hook" FAIL "$what ($t)"; return; }
  done
  line "$hook" PASS "$what"
}

lane build -- cargo test -p raea-page -p raea-runtime-twins -p raea-runtime-input --lib -j 6 > "$WORK/page.log" 2>&1
lane build -- cargo test -p raea-runtime-react -p raea-runtime-shared -p raea-runtime-page --lib -j 6 > "$WORK/react.log" 2>&1
(cd sdk/raea && bun test conformance/hooks.test.tsx) > "$WORK/bun.log" 2>&1
P=$WORK/page.log

tests H1 "twins: an entity per element, its box, found by selector" "$P" bridge::tests::twins_carry_boxes_and_follow_their_elements
tests H2 "PageCommands land before restyle, the same frame" "$P" bridge::tests::commands_land_before_restyle_in_the_same_frame
tests H3 "DOM events on the twins up the path, after the page's handlers" "$P" bridge::tests::dom_events_reach_the_twins_up_the_path bridge::tests::keys_focus_typing_hover_and_wheel_reach_the_twins
tests H4 "useResource and useComponent: one copy, the World's (Rust side)" "$WORK/react.log" tests::react_and_the_world_share_one_value
if grep -qE "^ *[0-9]+ pass" "$WORK/bun.log" && grep -qE "^ *0 fail" "$WORK/bun.log"; then line H4 PASS "the hooks under bun (conformance/hooks.test.tsx)"; else line H4 FAIL "the hooks under bun (conformance/hooks.test.tsx)"; fi
tests H6 "3D input: picking's backend and route, pointer capture, a page on a mesh, one focus and keys" "$P" pick::tests::bevy_picking_events_reach_twins_and_the_page_hears_the_press pick::tests::a_held_pointer_stays_on_its_page_past_its_edge pick::tests::a_page_on_a_tilted_mesh_is_clicked_through_its_uvs pick::tests::a_scene_in_an_element_is_picked_through_its_camera keys::tests::focus_is_one_and_keys_follow_it
tests H8 "the frame's order: input, events, game systems, commands, restyle, twins, in one update" "$P" bridge::tests::commands_land_before_restyle_in_the_same_frame scenarios::a_system_recolours_a_menu_in_the_same_frame scenarios::a_host_pages_commit_has_its_twins_for_that_frames_game_systems
tests H8 "a React page in WorldPagePlugin: its DOM events reach React's on* props" "$WORK/react.log" tests::a_dom_event_on_a_react_pages_element_goes_to_its_handler
tests H9 "one owner per property, conflicts named; a twin's PageElement and ElementBox are the page's" "$P" bridge::tests::each_property_has_one_owner bridge::tests::a_twins_own_components_are_the_pages
tests H9 "a layer's place is the ECS's: the page's transforms never move it" "$WORK/react.log" layers::tests::a_pages_transforms_never_move_its_layer

tests H11 "a click on a button on a tilted 3D screen" "$P" pick::tests::a_page_on_a_tilted_mesh_is_clicked_through_its_uvs
tests H11 "typing into a field on a phone in the scene, IME included" "$P" scenarios::typing_into_a_field_on_a_phone_in_the_scene
tests H11 "a system recolouring a menu in the same frame" "$P" scenarios::a_system_recolours_a_menu_in_the_same_frame
tests H11 "a card passing behind a 3D object, and out from behind it" "$P" scenarios::a_card_passing_behind_a_3d_object
tests H11 "a page view resized" "$P" scenarios::a_page_view_resized
tests H11 "a layer flown through the scene and clicked" "$P" scenarios::a_layer_flown_through_the_scene_and_clicked

if [ $PIXELS = 1 ]; then
  MOTION=${MOTION:-$ROOT/target/release/raea}
  if lane build -- cargo build --release -p raea-bridge-bench --example shared_state --example host_query --example entity_ref --example plain_page --example page_layers --example element_camera --example texture_slot --example planes --example click_make --example page_replay --example twin_cost -j 6 > "$WORK/examples.log" 2>&1; then
    cp bench/bridge/shared.tsx bench/bridge/hostquery.tsx bench/bridge/entity.tsx bench/bridge/video.tsx bench/bridge/screen.tsx sdk/raea/bench/bevy/classbg/edit.tsx "$WORK/"
    cp bench/bridge/can-turn.mp4 "$WORK/can.mp4"
    cp sdk/raea/bench/bevy/layers/edit.tsx "$WORK/layers.tsx"
    cp bench/bridge/texture.tsx "$WORK/"
    (cd "$WORK" && for c in shared hostquery entity video screen layers texture edit; do "$MOTION" bundle $c.tsx --composition-bundle -o $c.js || exit 1; done) > "$WORK/bundle.log" 2>&1
    out=$(target/release/examples/entity_ref "$WORK/entity.js" 2>/dev/null | grep '^entity_ref:')
    case "$out" in *PASS*) line H1 PASS "useEntity(ref) is the element's twin in a plain App (entity_ref)" ;; *) line H1 FAIL "useEntity(ref) in a plain App: ${out:-no result}" ;; esac
    out=$(target/release/examples/shared_state "$WORK/shared.js" 2>/dev/null | grep '^shared_state:')
    case "$out" in *PASS*) line H4 PASS "end to end in a plain App: ${out#shared_state: PASS }" ;; *) line H4 FAIL "end to end in a plain App: ${out:-no result}" ;; esac
    out=$(target/release/examples/host_query "$WORK/hostquery.js" 2>/dev/null | grep '^host_query:')
    case "$out" in *PASS*) line S2d PASS "a typed query on the host's World (HostWorld) in a plain App, a reflected component never shared: ${out#host_query: PASS }" ;; *) line S2d FAIL "a typed query on the host's World: ${out:-no result}" ;; esac
    (cd "$WORK" && RAEA_PAGE=blitz "$MOTION" render edit.tsx --at 30 -o "$WORK/ref/") > "$WORK/render.log" 2>&1
    target/release/examples/plain_page "$WORK/edit.js" "$WORK/plain.png" 1280 720 30 30 > "$WORK/plain.log" 2>&1
    ssim() { ffmpeg -hide_banner -nostats -i "${REF:-$WORK/ref/frame-00000.png}" -i "${GOT:-$WORK/plain.png}" -lavfi "[0:v]format=rgb24${1}[a];[1:v]format=rgb24${1}[b];[a][b]ssim" -f null - 2>&1 | grep -o 'All:[0-9.]*' | cut -d: -f2; }
    whole=$(ssim "") title=$(ssim ",crop=400:120:40:30")
    if [ -z "$whole" ]; then line H5 FAIL "the page over a plain App's scene: no frame (see plain_page, raea render)"
    else
      awk -v s="$whole" 'BEGIN { exit !(s >= 0.998) }' && line H5 PASS "the page over a plain App's scene (classbg frame 30, SSIM $whole against raea)" || line H5 FAIL "the page over a plain App's scene (classbg frame 30, SSIM $whole < 0.998)"
      [ "$title" = 1.000000 ] && line H10 PASS "the page blends as raea's feed (title area SSIM $title)" || line H10 FAIL "the page blends as raea's feed (title area SSIM $title, want 1.000000)"
    fi
    out=$(target/release/examples/page_layers 2>/dev/null | tail -1)
    [ "$out" = "page_layers: PASS" ] && line H5 PASS "world and surface modes: a page as a quad in 3D and on a cube's UVs (page_layers)" || line H5 FAIL "world and surface modes (page_layers: ${out:-no result})"
    out=$(target/release/examples/element_camera 2>/dev/null | tail -1)
    [ "$out" = "element_camera: PASS" ] && line H7 PASS "a camera's texture in an element in a plain App, GPU to GPU (element_camera)" || line H7 FAIL "a camera's texture in an element in a plain App (element_camera: ${out:-no result})"
    out=$(target/release/examples/texture_slot "$WORK/texture.js" 2>/dev/null | tail -1)
    [ "$out" = "texture_slot: PASS" ] && line H5 PASS "a texture element in a plain App's page shows the App's PageTextures (texture_slot)" || line H5 FAIL "a texture element in a plain App (texture_slot: ${out:-no result})"
    out=$(target/release/examples/click_make 2>/dev/null | tail -1)
    [ "$out" = "click_make: PASS" ] && line H3 PASS "a click through Bevy's picking on a host page in a plain App: the handler's new element has its twin in that event (a dom::Click observer finds it), and a click on it reaches that twin (click_make)" || line H3 FAIL "a click that makes an element, live in a plain App (click_make: ${out:-no result})"
    out=$(target/release/examples/planes 2>/dev/null | tail -1)
    [ "$out" = "planes: PASS" ] && line H5 PASS "3D planes in a plain App: the nearer plane over the farther (transform3d-sorting-001), a plane past the viewer drawn up to its w = 0 cut, two planes crossing at 80deg split where they cross (BSP, 9c779dc4) (planes)" || line H5 FAIL "3D planes in a plain App (planes: ${out:-no result})"
    out=$(target/release/examples/page_replay play "$WORK/replay.json" "$WORK" > /dev/null && target/release/examples/page_replay render "$WORK/replay.json" "$WORK" 2>/dev/null | tail -1)
    [ "$out" = "page_replay: PASS" ] && line H10 PASS "a page in a plain App played live with a mouse, recorded, and rendered from the log: hover, :active and plain frames the same bytes (page_replay)" || line H10 FAIL "a page played and rendered from its input log (page_replay: ${out:-no result})"
    # H1 cost: the twins system on a app-sized page (500 elements, nothing changed), timed under load; the gate is 250 µs and 0 allocations a frame
    if cost=$(target/release/examples/twin_cost 2>/dev/null); then g=$(grep -m1 "500 elements.*budget" <<<"$cost" | sed -E "s/.*: ([0-9.]+ µs) and ([0-9.]+) allocations.*/\1, \2 allocations/"); line H1 PASS "a twin's cost on a app-sized page (500 elements, nothing changed): $g a frame (twin_cost)"
    else line H1 FAIL "a twin's cost on a app-sized page: $(grep -m1 "over the budget" <<<"$cost" || echo "no result") (twin_cost)"; fi
    # the clip's frame at the composition's frame 30, against raea's; and not the page's background (a missing video
    # would match too)
    (cd "$WORK" && RAEA_PAGE=blitz "$MOTION" render video.tsx --at 30 -o "$WORK/vref/") > "$WORK/vrender.log" 2>&1
    target/release/examples/plain_page "$WORK/video.js" "$WORK/video.png" 1280 720 30 30 > "$WORK/video.log" 2>&1
    v=$(REF=$WORK/vref/frame-00000.png GOT=$WORK/video.png ssim ",crop=320:568:80:60")
    mean=$(ffmpeg -v error -i "$WORK/video.png" -vf "crop=320:568:80:60,scale=1:1" -f rawvideo -pix_fmt rgb24 - 2>/dev/null | xxd -p)
    if [ -n "$v" ] && [ "$mean" != 1b2230 ] && awk -v s="$v" 'BEGIN { exit !(s >= 0.999) }'; then line H11 PASS "a video playing inside a page in a plain App (its area SSIM $v against raea)"
    else line H11 FAIL "a video playing inside a page in a plain App (SSIM ${v:-none}, mean ${mean:-none})"; fi
    # a <Screen>: the frame against raea's, and its page on it: red at its middle, the card blue, black around
    (cd "$WORK" && RAEA_PAGE=blitz "$MOTION" render screen.tsx --at 30 -o "$WORK/sref/") > "$WORK/srender.log" 2>&1
    target/release/examples/plain_page "$WORK/screen.js" "$WORK/screen.png" 1280 720 30 30 > "$WORK/screen.log" 2>&1
    sc=$(REF=$WORK/sref/frame-00000.png GOT=$WORK/screen.png ssim "")
    at() { ffmpeg -v error -i "$WORK/screen.png" -vf "crop=1:1:$1:$2" -f rawvideo -pix_fmt rgb24 - 2>/dev/null | xxd -p; }
    if [ -n "$sc" ] && awk -v s="$sc" 'BEGIN { exit !(s >= 0.995) }' && [ "$(at 640 360)" = ff3030 ] && [ "$(at 794 424)" = 2040ff ] && [ "$(at 100 100)" = 000000 ]; then
      line H5 PASS "a <Screen>'s nested composition on its screen in a plain App (SSIM $sc against raea)"
    else line H5 FAIL "a <Screen> in a plain App (SSIM ${sc:-none}; middle $(at 640 360), card $(at 794 424), around $(at 100 100))"; fi
    # a layered Canvas3D: the frame exactly raea's (the 3D made straight for the slot as raea's render makes it), and
    # the page under the sphere and over it
    (cd "$WORK" && RAEA_PAGE=blitz "$MOTION" render layers.tsx --at 30 -o "$WORK/lref/") > "$WORK/lrender.log" 2>&1
    target/release/examples/plain_page "$WORK/layers.js" "$WORK/layers.png" 960 540 30 30 > "$WORK/layers.log" 2>&1
    L() { REF=$WORK/lref/frame-00000.png GOT=$WORK/layers.png ssim "$1"; }
    lw=$(L "") lu=$(L ",crop=240:80:20:20") lo=$(L ",crop=900:40:0:250")
    if [ "$lw" = 1.000000 ] && [ "$lu" = 1.000000 ] && [ "$lo" = 1.000000 ]; then
      line H5 PASS "a layered Canvas3D in a plain App: the page under and over its 3D as raea's (SSIM $lw; under $lu, over $lo)"
    else line H5 FAIL "a layered Canvas3D in a plain App (SSIM ${lw:-none}; under ${lu:-none}, over ${lo:-none})"; fi
  else
    line H4 FAIL "the examples didn't build ($WORK/examples.log)"
  fi
  # S8b: a Blitz widget crate declared in the plugin catalog (raea-bench-swatch's "widgets"), drawn by the plugin
  # build of raea: widget.tsx's <x-swatch> fills its 200 x 150 box blue on the red page
  PLUGINS=${PLUGINS:-$ROOT/target/release/raea-bevy-sdk}
  if [ -x "$PLUGINS" ]; then
    cp bench/bridge/widget.tsx "$WORK/"
    (cd "$WORK" && "$PLUGINS" motion render widget.tsx --at 0 -o "$WORK/wref/") > "$WORK/wrender.log" 2>&1
    px() { ffmpeg -hide_banner -nostats -loglevel error -i "$WORK/wref/frame-00000.png" -vf "crop=1:1:$1:$2,format=rgb24" -f rawvideo - 2>/dev/null | od -An -tu1 | tr -s ' ' | sed 's/^ //; s/ $//'; }
    inside=$(px 200 125) outside=$(px 500 300)
    if [ "$inside" = "0 0 255" ] && [ "$outside" = "255 0 0" ]; then
      line S8b PASS "a declared Blitz widget (raea-bench-swatch) drawn by raea's plugin build: its box $inside, the page $outside"
    else line S8b FAIL "the declared <x-swatch> (box ${inside:-none}, want 0 0 255; page ${outside:-none}): a plugin build with the catalog's swatch? ($WORK/wrender.log)"; fi
  else
    line S8b GAP "no plugin build at $PLUGINS (tools/bevy-plugins.sh build sdk/raea/bench/bevy-plugins)"
  fi
  # the whole frame, 3D and page, as raea renders it: classbg's cube and the <Screen>'s quad (raea-render's Engine drew
  # its 3D over its target's tag row too, 0.14% larger and half a pixel low, until tag_row_view)
  [ "${whole:-}" = 1.000000 ] && [ "${sc:-}" = 1.000000 ] && line H10 PASS "the frame entire at 1.000000 (classbg $whole, the <Screen> $sc)" || line H10 FAIL "the frame entire (classbg ${whole:-none}, the <Screen> ${sc:-none}, want 1.000000)"
else
  line H10 GAP "pixels not run (--pixels)"
fi
[ $FAILS = 0 ]
