@spec
Feature: Reviewing a video in raea's player: comments written on the picture, every layer in it selectable
  Shane, 2026-11-09: "i want comments in video player i want to be able to do it all without having to open or look
  at the comment panel i want to leave them inline. also make sure we are able to whene exporting a video form raea we
  allow for the dom sleectable like 3d objects/layers if we want in the video player. right now i cant sleect anything.
  so how do we make sure that actually works and does so consistently across 3d and reacta nd dom objects equally"

  What the player is today (apps/app src/detail), read 11-09:
    - Comment (C) drops a pin on the picture, then opens the notes rail, where the words are written (DetailView.tsx
      opens it on every new pin). Replies, resolve, Edit/Note and sending the edits to the agent are all in the rail.
      Posted pins show on the picture at every moment of the video, not at their own.
    - Select (P) picks from item.parts: one fixed box per part for the whole video. Only the demo fixtures fill it; a
      real render never does (canPick is true, the list is empty), so nothing can be picked.
    - The render manifest (engine/page motion.rs) gives each text run and video a box per frame, and with E2's stamps
      (docs/editing.feature, branch stamps-wip, parked) the JSX that drew it. 3D entities, images, shapes and plain
      elements aren't in it, and 3D JSX isn't stamped (stamp.rs leaves R3F and bevy-react tags alone).

  The model this file sets:
    1. Reviewing happens on the picture. A comment is written where it's pinned, read where it's pinned, answered and
       resolved there; the rail is a list of the same threads, never the only place to act on one.
    2. A layer is anything the composition draws that a person can point at: a 3D mesh, a page element, a run of text,
       an image, a video, a component's use. Each has one key: its source stamp (file:line:col), the uses of the
       edit's components it was drawn in, and its place among what the same JSX drew (a .map()'s key or index). The
       same thing keeps the same key in every frame, in Layers, on the timeline and in a comment.
    3. What is at a pixel is decided once, by the engine, from the frame as drawn: a layer pass gives each pixel the
       key of the frontmost layer there, through the frame's real structure (the page over the 3D, a <Screen>'s page
       on a mesh, a 3D view in an element). The player, the editor and agents all read that pass; none of them guesses
       from boxes, so 3D, React and the page can't disagree.
    4. Layers ride with an export when asked: the render writes the layer pass beside the video, and the player makes
       the video selectable from it. A video without it says so, and how to get it.
    5. "Works" is checked against an oracle that knows nothing of the layer pass: hide one layer, render again, and
       the pixels that change are where that layer shows. A conformance composition holds the same shape as a 3D
       mesh, a React component and a page element, and every layer kind in it must match the oracle.

  Background:
    Given the player's code is apps/app src/detail (Frame.tsx, views/Watch.tsx, player.tsx, blocks/Review.tsx), the
      manifest's is engine/page (motion.rs), the 3D pass's is engine/render (engine/aov.rs is RT1's depth and motion),
      and the stamps' is engine/bundle (stamp.rs, on stamps-wip)
    And Rust changes go through the lanes' build rules (announce, `lane build --`, no edits under a peer's build)

  Rule: R1. Comments are written, read and answered on the picture (app only)
    # Status 11-09: built (raea-feed 281ce00, src/detail/blocks/Inline.tsx; Frame, Timeline, Watch, Look, DetailView).
    # Checked in headless Chrome on the reel fixture: composer beside the pin, rail closed; pins as the author's initial
    # ringed white (readable on bright and dark footage); hover reads, click opens to reply and resolve; marks show their
    # first line and open the comment; "Send 2 edits to Cutter" in the bar. e2e smoke: 5 pass (two new cases). Not yet
    # checked in raea.app's own runtime (next repackage).

    Scenario: R1a. A comment is written where it's pinned
      Given a video open in the player
      When I press C and click the picture
      Then the video pauses, a composer opens beside the pin, and the notes rail stays as it was
      When I type and press Enter
      Then the comment is posted with that place and moment, its pin stays on the picture, and Comment stays on for
        the next one; Esc closes the composer without posting

    Scenario: R1b. Edit or Note is chosen in place
      Given the video was made by an agent
      Then the inline composer offers Edit and Note, Edit first, as the rail's does

    Scenario: R1c. A comment shows at its own moment
      Given a comment pinned at 0:12
      Then its pin shows from 0:12 for 2 s while playing, and whenever the playhead is paused within 0.5 s of it
      When I hover the pin
      Then its words, author and replies show beside it
      When I click it
      Then it stays open with Reply, Resolve and Edit/Note, and a reply is written in it

    Scenario: R1d. The scrubber's marks open their comments
      When I hover a comment's mark on the scrubber
      Then its first line shows above the mark
      When I click the mark
      Then the video seeks to it, pauses, and its comment opens on the picture

    Scenario: R1e. The edits go to the agent from the bar
      Given three comments marked Edit and not sent
      Then the player's bar has "Send 3 edits" naming the agent, and pressing it sends them as one round

  Rule: R2. Every layer is selectable, the same way for 3D, React and the page (engine and app)
    # Status 11-09: the player side (R2d, R2f) is built (raea-feed 79f58c5, cc9e064, d331572): host/layers.ts reads
    # NAME.layers (v1: gzip, RAEALAYR, header JSON, per frame u32 N runs of u16 value + u16 count, N=0 repeats; a
    # parent as its key or index; "at" for a render of chosen moments), Select outlines a layer by its mask shape (text
    # by its box, and preferred within 2 mask px since its mask is glyph-exact), a pin follows its layer. The engine's
    # first file (issues lane, stamps-wip: systems/layers, bench/layers/compare.sh) has the 12 page layers of the
    # conformance composition at IoU >= 0.977 against the hide-and-diff oracle; the 3D half is next. `--layers` and its
    # manual entry land with stamps-wip; the app's export passes it once that raea ships (R2c's toggle).

    Scenario: R2a. Every drawn thing has its layer key
      Given a composition with a 3D mesh, a component used twice, a page element, text, an image and a video
      Then each carries its source stamp and use chain to what draws it: page elements as E2's data-raea-loc and
        data-raea-use, 3D entities as a component on the entity (never a prop the 3D kinds read)
      And a .map() drawing five cards gives five keys that differ only in their index

    Scenario: R2b. The engine gives each pixel its frontmost layer
      When a frame is rendered with the layer pass on
      Then a mask at a quarter of the frame's size holds, per pixel, the index of the frontmost layer in the frame's
        layer table, or none: 3D from the drawn meshes (skinned and morphed as drawn), the page in its paint order,
        a <Screen>'s page through its mesh, a 3D view through its element
      And the table gives each index its key, kind (mesh, element, text, image, video), name and parent

    Scenario: R2c. An export can carry its layers
      When I render with `raea render --layers`, or export from the app with "Selectable layers" on
      Then NAME.layers sits beside NAME.mp4 (the table and every frame's mask, run-length coded and compressed), and
        render.json names it
      And a render without it plays as before

    Scenario: R2d. The player selects from the layers
      Given a video exported with its layers
      When I press P and hover the picture
      Then the frontmost layer under the pointer is outlined by its own shape (a mesh's silhouette, not a box) with
        its name, at any moment, playing or paused
      When I click it
      Then it's selected, its path is shown (each step selects that parent), and E opens the editor at its JSX
      When I comment with it selected
      Then the comment is pinned to its key, and its pin follows the layer as it moves

    Scenario: R2e. 3D, React and the page agree with the oracle
      Given the conformance composition: one shape drawn as a 3D mesh, as a React component and as a page element,
        overlapping, with text, an image, a video, a <Screen> on a mesh, a rotated element, a skinned character and
        a .map() list
      When each layer is hidden in turn and the frame rendered again
      Then the pixels that change and the layer pass's pixels for it overlap with IoU 0.95 or better, for every layer,
        at three moments
      And mutants fail it: two keys swapped, the 3D mask shifted 2 px, occlusion ignored, the page drawn under the 3D

    Scenario: R2f. A video without layers says how to get them
      Given a raea render exported without layers
      Then Select is disabled and its tooltip says "Rendered without layers: export again with Selectable layers on"
