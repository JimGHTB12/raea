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

    Scenario: PV1a. A part under the pointer
      Given a composition with 3D meshes playing on the stage
      When stage.pick is asked a point over a mesh
      Then it answers that part: its layer key (file:line:col, its list index, its uses), name, kind "mesh", its box
        on the stage, the point hit in the world and its distance, the nearest mesh along the pointer's ray as the
        frame was drawn (skinned meshes as posed)
      And a page element in front of the 3D is answered instead, and the 3D where the page over it is see-through

    Scenario: PV1b. The stage's compositions are stamped
      Then a composition the stage opens is bundled with its source stamps, so every part and element has its key

    Scenario: PV1c. A free camera
      When stage.view is given an orbit (yaw, pitch, distance, target) and the scene is 3D
      Then the frame is drawn from there until stage.view null puts the composition's own camera back
      And picking uses the camera the frame was drawn from

  Rule: PV2. What a thing is (app host)

    Scenario: PV2a. One description of any file
      When the host is asked what a project's file is
      Then it answers kind, timed (seconds, fps), plays, space (2d, 3d, both), input, props, includes and weight, and
        whether a proxy is needed, for a composition, a component, an image, a video, a sound, a model and a font

  Rule: PV3. Proxies and levels of detail (app host)

    Scenario: PV3a. Every medium has its light copies
      Then an image has a thumbnail, a video a poster and a 540-line copy, a sound its peaks, a model a poster and
        lower levels of detail, a composition and a component a poster, each made once and found again by its source's
        path, size and time

    Scenario: PV3b. The store is managed in one place
      Then Settings lists the store by project and kind with sizes, and offers Clear and Rebuild; copies whose source
        is gone are dropped; the store keeps under its limit by dropping the least recently used

    Scenario: PV3c. A model's levels of detail
      Then a heavy model gets lower levels (triangles halved per level, textures at most 1024 px) and the preview
        draws the level its size on screen calls for, the full model when still

  Rule: PV4. One preview (app)

    Scenario: PV4a. It opens at once
      When I open a component, a composition or an asset
      Then its poster shows in the first frame the page draws, and the live stage takes over when it is ready

    Scenario: PV4b. Props change the picture as they move
      When I drag a component's number prop
      Then the stage shows each value without reopening the composition and without a render

    Scenario: PV4c. Time only where there is time
      Then a still component or an image has no time bar; a timed one plays, loops and scrubs

    Scenario: PV4d. Pointing and selecting
      When I hover and click a 3D part or an element in the preview
      Then it is outlined, selected, shown in the outline tree, and its props are the panel's

  Rule: PV5. Notes on anything (app)

    Scenario: PV5a. A note on a model
      When I click a point on a model in its preview and write a note
      Then the note is kept with the asset, pinned to that point on that part, and shown there from any camera
