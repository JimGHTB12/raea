@spec
Feature: Preview: anything in a project opens at once, in its space, with its props, and can be pointed at
  Shane, 2026-11-09: "all components and compositions have a really hard time loading. we should have the ability to
  recognize how/what it is and view it in 3d space with all that is included and see and mess with props. This will
  require us to be comprehensive with any type of media, if time is a factor or not or can be seen in playback, if we
  need to proxy it or not ... a complete proxy management solution and LOD solution. as well we should be turning on
  selected components for scenes so they're pointable. i have yet to see a single one actually make it so i can point
  and click on objects in 3d space. and within components, assets, compositions being able to have that level of
  control is also good. i might even have notes for a 3d asset"

  What there is today (read 11-09):
    - A component's page shows a still the host renders per change (host/props.ts preview: a throwaway composition and
      a `raea render`, about 2 s each after a 450 ms wait). No time, no 3D view, nothing in it can be picked.
    - A composition shows its last render's mp4; with no render it shows nothing until one is made. Live (the bolt)
      puts the engine's stage in its place, off by default.
    - A render the host makes carries its layers (review.feature R2c, on by default), so the player's Select picks 3D
      parts in a finished video. The live stage's pick (stage.pick) is the page's hit test only: over 3D it answers
      the element holding the scene, never the part. So a game, a Live composition and every component have no
      pointable 3D.
    - Proxies are a render's 540-line copy only (host/proxy.ts). Footage, stills, sound and models in a project have
      none; an asset's card loads the file itself. Nothing lists, sizes or clears them.
    - An asset opens in the browser or its Mac app; a model has no viewer and nothing can carry a note.

  The model this file sets:
    1. One description. Any file in a project has one answer to "what is this": its kind (composition, component,
       image, video, sound, model, font, document), whether time is part of it (length, rate) and it plays, whether it
       is 3D, 2D or both, whether it takes input (a game), its props, what it includes (media, components, models),
       and its weight (bytes, pixels, triangles), from which whether it needs a proxy follows.
    2. One proxy store. Every heavy thing has light copies made once and kept by what it was made from: a poster for
       anything drawn, a 540-line copy for video, peaks for sound, lower levels of detail for a model. The store says
       what it holds, what each copy is of, its size and state, and is cleared or rebuilt from one place.
    3. One preview. A component, a composition and an asset open in the same surface: the poster at once, then the
       engine's stage playing it live (in the raea app), with the time bar only when time is part of it, a free camera
       when it is 3D, the outline of what it includes, and its props as controls that change the picture as they move.
    4. Everything drawn is pointable. On the live stage as in a render: hover outlines the frontmost thing (a 3D part,
       an element, a text), a click selects it, and the selection is a place in the code with its props.
    5. A note goes on anything: a moment, a part, a point on a model's surface.

  Background:
    Given the app's code is apps/app (raea-feed) src/compositions, src/detail, host; the engine's is engine/page/stage,
      runtime/live, systems/frame and systems/layers
    And plain `raea render FILE -o OUT` output stays as it is: nothing here changes a render's defaults

  Rule: PV1. The live stage picks 3D parts (engine)
    # Status 11-09: built. systems/frame pick.rs (the nearest mesh along the pointer's ray from the frame's camera,
    # skinned meshes as posed; locate: where parts and world points are on the frame now) and view.rs (FreeView, an
    # orbit held over the composition's camera and given back); runtime/live pick3d.rs (pick: the page's element or
    # the 3D's part as one picture, a part answered with its JSX's own props; set_view, locate, set_props);
    # engine/page/stage: stage.pick, stage.view, stage.locate, stage.props, and what it opens is stamped. Checked with
    # `raea play examples/games/shapes.tsx --offscreen --seek 10 OUT.png --pick 640,330` (shape-2 at
    # shapes.tsx:28:11#2 with its position and quaternion; the floor; the div where there is no mesh) and
    # `--view 1.2,0.9,9,0,0.5,0` (drawn from above, the same part under the same point). raea-sys-frame: 3 tests.

    @done
    Scenario: PV1a. A part under the pointer
      Given a composition with 3D meshes playing on the stage
      When stage.pick is asked a point over a mesh
      Then it answers that part: its layer key (file:line:col, its list index, its uses), name, kind "mesh", its box
        on the stage, the point hit in the world and its distance, the nearest mesh along the pointer's ray as the
        frame was drawn (skinned meshes as posed)
      And a page element in front of the 3D is answered instead, and the 3D where the page over it is see-through

    @done
    Scenario: PV1b. The stage's compositions are stamped
      Then a composition the stage opens is bundled with its source stamps, so every part and element has its key

    @done
    Scenario: PV1c. A free camera
      When stage.view is given an orbit (yaw, pitch, distance, target) and the scene is 3D
      Then the frame is drawn from there until stage.view null puts the composition's own camera back
      And picking uses the camera the frame was drawn from

  Rule: PV2. What a thing is (app host)
    # Status 11-09: built (raea-feed host/preview describe.ts, glb.ts; POST /host/describe). A model's facts come from
    # its glb's JSON chunk alone (triangles, nodes, clips, its box). Checked on a scratch project: a component (3D,
    # timed, its four props), a composition (both, 3 s), a model (27,420 triangles), a video, a sound. 5 tests.

    @done
    Scenario: PV2a. One description of any file
      When the host is asked what a project's file is
      Then it answers kind, timed (seconds, fps), plays, space (2d, 3d, both), input, props, includes and weight, and
        whether a proxy is needed, for a composition, a component, an image, a video, a sound, a model and a font

  Rule: PV3. Proxies and levels of detail (app host)
    # Status 11-09: built (raea-feed host/preview proxies.ts; GET /host/proxy, POST /host/proxies; Settings › General ›
    # Light copies). thumb, light and peaks are raea's own verbs (media still --width, new; media proxy; media audio);
    # a model's still is its view rendered; lod is `raea media lod` (new, apps/media lod.rs: meshoptimizer on the
    # model's own vertices, so skins and UVs are untouched; embedded pictures at most 1024 px; what nothing uses
    # leaves the file). A render's light copy (editing.feature E4) is the same store's now. Checked: body.glb 27,420
    # triangles -> 6,854 at level 2, rendered and looked at; a 1280x720 take's still in 0.28 s and light copy in
    # 0.9 s; a sound's peaks in 0.15 s; Settings said "318 KB used: 3 stills, 1 videos, 1 sounds". 8 + 2 tests.
    # 11-09 later: the level follows the model's size on screen (light under a third of the frame's height, full from
    # just over). An FBX, OBJ, USD or .blend gets a glb copy (raea blender run blender/convert.py, from a scratch
    # folder of links with the script's licence note written there); a font a full still of its specimen. Checked:
    # crate.obj converted and drawn in 2.8 s. The store, bundled as raea.app bundles its host (esbuild, run outside
    # the repo), answered describe, view, proxy (thumb, still, peaks, the obj), jsx and file-notes.

    @done
    Scenario: PV3a. Every medium has its light copies
      Then an image has a thumbnail, a video a poster and a 540-line copy, a sound its peaks, a model a poster and
        lower levels of detail, a composition and a component a poster, each made once and found again by its source's
        path, size and time

    @done
    Scenario: PV3b. The store is managed in one place
      Then Settings lists the store by project and kind with sizes, and offers Clear and Rebuild; copies whose source
        is gone are dropped; the store keeps under its limit by dropping the least recently used

    @done
    Scenario: PV3c. A model's levels of detail
      Then a heavy model gets lower levels (triangles halved per level, textures at most 1024 px), and its preview
        opens on one and draws the full model once the view has stood a moment

  Rule: PV4. One preview (app)
    # Status 11-09: built in the raea app, checked in its headless window on a scratch project (raea-feed
    # src/preview: PreviewStage, stage.ts, Selected, LivePick, AssetPreview). A component's page plays it on the stage:
    # a click on the cylinder of a five-shape row answered shape-2 at row.tsx:26 item 2 with its props, dragging
    # `lift` raised the shapes as it moved, Orbit turned the scene. A composition never rendered played on opening,
    # and a click on its title answered the div at shapes.tsx:8. A model, a video, a still and a sound each opened on
    # their page from Assets, whose cards are the store's stills. In Chrome there is no stage: the still stays.
    # 11-09 later: the selection's card changes the code (raea-feed host/preview jsx.ts, POST /host/jsx): the picked
    # element's own attributes as written, a literal a control written where it stands with Undo, a computed one
    # shown as code; on a component's page, a composition never rendered (the first change starts NAME-vN.tsx) and
    # the editor's Live view. Checked: the floor's name set from its card changed row.tsx line 21 and Undo put it
    # back, the stage live through both. Every kind opens: a font's specimen, a document's or code's text, a splat on
    # the stage (turned upright when it has no placement beside it, with Flip), a converted model, and anything
    # else in its Mac app. Composition and component rows have their still. 3 tests (jsx).
    # In a browser there is no stage, so a preview there is the still; the composition editor's own Select tool
    # (the move, scale and fade of a part) still reads a render's layers, and Live's pointing is the card's.

    @done
    Scenario: PV4a. It opens at once
      When I open a component, a composition or an asset
      Then its poster shows in the first frame the page draws, and the live stage takes over when it is ready

    @done
    Scenario: PV4b. Props change the picture as they move
      When I drag a component's number prop
      Then the stage shows each value without reopening the composition and without a render

    @done
    Scenario: PV4c. Time only where there is time
      Then a still component or an image has no time bar; a timed one plays, loops and scrubs

    @done
    Scenario: PV4d. Pointing and selecting
      When I hover and click a 3D part or an element in the preview
      Then it is outlined, selected, shown in the outline tree, and its props are the panel's

  Rule: PV5. Notes on anything (app)
    # Status 11-09: built (raea-feed host/preview notes.ts, POST /host/file-notes; kept in the project's
    # .raea/notes/). Checked: a click on body.glb's chest and a note wrote pin.point [0.00, 1.23, 0.13], and its pin
    # stayed on the chest through an orbit (stage.locate). 1 test. A note's Ask an agent starts a task in the
    # project's feed with the note, where it is pinned and where the notes are kept (not run against a live agent).

    @done
    Scenario: PV5a. A note on a model
      When I click a point on a model in its preview and write a note
      Then the note is kept with the asset, pinned to that point on that part, and shown there from any camera
