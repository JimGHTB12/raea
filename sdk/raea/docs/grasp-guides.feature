Feature: Grasp guides: simple shapes on a thing that say how a hand takes it
  Set by Shane, 2026-10-22: "use cylinder guide shapes placed in places or something with a tag... take an interactive
  object and determine it by making the agent [define it]. in my head the cylinder is the bend of the hand around a
  thing at whatever degrees and the something for the thumb?"

  Before this, items.ts put each hand on a thing by a table of places per grip kind (handsOn), and the grips bench only
  judged how far skin went into the thing. A friction check showed most can and mug grips touched only one side of the
  thing: nothing went in, but nothing could have held it up either (closure on about 0 of the can and mug frames). That
  comes from where the hand is put, not from how the fingers close. Guides put the hand so that the palm, the fingers
  and the thumb push from opposed sides, after the grasp-primitives literature (Miller et al. 2003, "Automatic grasp
  planning using shape primitives"): a thing is held as the primitive it is nearest, and each primitive has its own few
  ways of being taken.

  Code: sdk/raea/src/grasp.ts (the guides, autoGuides, pickGuide, handOn, contactsOf, holds), sdk/raea/src/things.ts
  (Thing.guides), sdk/raea/src/items.ts (guided: a pick's hand on its guide; the thing riding that hand rigidly).
  Bench: sdk/raea/bench/character/grips.ts (closure per scene). Tests: sdk/raea/conformance/grasp.test.ts.

  Rule: A guide is a shape in the thing's frame (its bottom's middle at the origin, x across, y up, z its front, metres)
    Scenario: The four kinds
      Then a cylinder is `at` (its axis's middle), `axis`, `radius` and `length`: the fingers wrap round it, the palm on
        one side; `thumb: "wrap"` (the default) closes the thumb over the fingers, `"along"` lays it along the axis (a
        knife, a racket)
      And a pinch is `at` (between the pads), `across` (from the thumb's pad to the fingers') and `width` (the thing's
        thickness there); `fingers` is "pinch" (thumb and index), "tripod" (a pencil) or "stem" (a wine glass's)
      And a sphere is `at` and `radius`: the palm on it, the fingers spread over it
      And a slab is `at` (between the faces), `normal` (across them), `width` (their distance apart), `span` (the
        faces' extent along `up` and across), `hands` (two on opposite faces, or one hand's thumb on one face and its
        fingers on the other: a book)
    Scenario: Tags
      Then every guide may carry tags ("body", "handle", "stem", "bowl", "lid", "rim", "side", anything)
      And a pick asks for one with `{ pick: "mug", tag: "handle" }`; without a tag the guide the grip calls for is taken
        (pickGuide: a handle's cylinder for "handle", a stem's pinch for "stem", a body's cylinder for "wrap" and "top",
        a sphere for "palm", a slab for "edges" and "sides")

  Rule: A thing with no guides of its own gets them from its shape and size (autoGuides)
    Scenario: The defaults
      Then a "round" thing gets a cylinder round its body (tag "body"), its radius half its across and its length its
        height
      And a grip with kind "stem" adds a pinch on the stem (first, tag "stem"), and the body's cylinder becomes the bowl
        above it (tags "body", "bowl")
      And any other grip (a handle) adds a cylinder along its `along` (first, tag "handle", radius 6 mm)
      And a "ball" gets a sphere; a "pole" a cylinder along z
      And a box gets a slab across each pair of its upright faces (normal x, then z), two hands where the faces are
        further apart than a hand spans (18 cm by default), else one
    Scenario: Guides of its own replace the defaults
      Given a Thing with `guides`
      Then those are its guides and nothing is fitted; a scene author or an agent places them by hand:
        const mug: Thing = { at, size: [0.085, 0.1, 0.085], shape: "round", guides: [
          { kind: "cylinder", at: [0.06, 0.05, 0], axis: [0, 1, 0], radius: 0.006, length: 0.06, tags: ["handle"] },
          { kind: "cylinder", at: [0, 0.05, 0], axis: [0, 1, 0], radius: 0.0425, length: 0.1, tags: ["body"] }] };
      And the old `grips` still work as they did (a handle, a stem); they only add guides when there are none

  Rule: The hand is planned on the guide in its own frame (handOn), so the same guide is taken from beside or above
    Scenario: A cylinder from the side (a can, a mug's body: grip "wrap")
      Given the way from the guide toward the hand (a one-hand carry: the hand's own side and a little behind)
      Then the palm goes flat on that side of the axis, its middle a little toward the wrist from where it touches
      And the axis lies diagonally across the palm (0.8 rad, the fingers turned toward the thumb's end of it), so the
        thumb, closing, goes round the far way from the fingers; square across the palm the thumb lay on the can's top
        and the fingers reached a third of the way round, one side
      And the hand sits down the axis from its middle by 0.28 of its length (at most 0.4 of the cylinder's), so the
        thumb's root is on the side and not past the end: a man's hand is longer than a can
    Scenario: A cylinder from its end (a can from above: grip "top", a patient pick)
      Then the palm goes on the end facing down, its middle back from the end's middle by 1.2 radii, the fingers over
        the far edge and down that side
      And the hand is turned about the axis toward the thumb (0.6 rad), so the thumb goes down the near side across
        from the fingers (square, they went down sides a quarter turn apart and pushed at right angles)
    Scenario: The fingers
      Then closeOn (character.tsx) closes the fingers and the thumb onto the thing's distance field, as for every grip
    Scenario: The thing rides the hand
      Given a thing in one hand placed on a guide
      Then it rides that hand rigidly, turned with it as well as moved with it (a hand half given back to the walk's
        swing left a can a centimetre and a half off the palm, held by its fingertips)
      And the hand keeps its turn on it however much of the arm is the walk's (the wrist takes up the swing): a can rode
        a half-free hand tipped 45 degrees
      And the thing is pushed out of the palm as the hand last closed on it (laid straight, the knuckles' pads stand off
        the palm and pushed a can out of the palm's reach)

  Rule: Where guides place the hand today, and where the old places stay
    Scenario: Guided
      Then a round thing's body wrapped ("wrap": the can, the adults' mug) and a round thing taken from above ("top" on
        a cylinder: the can looked at)
    Scenario: Not yet guided (the old per-grip places, kept because a guide didn't beat them on the bench, or isn't
      wired)
      Then a ball in the palm and from above: the sphere held on about as many frames as the old places (the kid's
        ball 81% against 84, the woman's ball-look 86% against 100), so the old places stay
      And a mug's handle: its bar isn't in the thing's distance field (thingSdf draws the mug as a plain cylinder), so
        the fingers close on the body and no grip on it can be judged
      And the pinch, the tripod and the stem: the stem already holds (97-100%); the pencil is 8 mm across and its
        closure is near 0 either way
      And two hands (edges, sides, cradle, hug): bigHold's search places them; slabs are fitted but not yet used there

  Rule: Closure is measured, frame by frame (holds, bench/character/grips.ts)
    Scenario: What holds means
      Given the skin touching the thing (each part's sample nearest it, within 3 mm: the hands', the arms', the trunk's)
      Then each is a point contact with friction 0.5 (its cone as six edges), pushing along the surface's inward normal
      And it holds when some pushes along them add up to the thing's weight upward with no turn about its middle (a turn
        over 5 cm counts as a force): the nonnegative least-squares fit (Lawson and Hanson) leaves under a tenth of the
        weight
      And the grips bench reports it per scene, the share of held frames that hold; it is reported, never a pass/fail
    Scenario: The earlier check
      Then the first closure script fitted the same pushes by projected gradient, which didn't converge with a dozen
        contacts: it said the kid's shoebox held on 0 of 81 frames where the exact fit says 70

  Rule: Results (2026-10-22, against main at 59ceeb01, paired in one worktree)
    Scenario: Closure, kid / man / woman, main to guides (parkour.json; the full databases)
      Then the can 1 / 0 / 2% to 98 / 98 / 97 (0 / 5 / 7 to 96 / 100 / 98)
      And the adults' mug, wrapped 0 / 2 to 98 / 99 (21 / 15 to 99 / 96)
      And the can taken from above and looked at 31 / 66 / 28 to 100 / 96 / 99 (36 / 34 / 6 to 99 / 98 / 96)
      And every scene a guide doesn't place (ball, pencil, glass, the kid's mug by its handle, shoebox, ball-look,
        bigball, bigbox) identical in closure and clip frames
    Scenario: Clips and the bars
      Then clip frames fall or stay on every guided scene but the woman's database can-look (3 to 4); the man's
        database mug 11 to 0 (his thumb's middle bone had been up to a centimetre in)
      And grips 27/30 either way; the databases kid 10/10, man 7 to 8/10, woman 9/10 (before the box change below); parkour identical; conformance
        279 to 286 pass (grasp.test.ts), 36 skip
    Scenario: Two hands on a box's edges (bigHold, after the guides)
      Then the edges' places along a box's side count down by how far from its middle they are (all of it at its end,
        was a third): a man took a shoe box lying lengthwise by its near end, its middle 14 cm past his hands
      And each palm is drawn back out of the box a few times over (a reach lands within a centimetre)
      And the shoebox's closure 89 / 25 / 37% to 99 / 90 / 90 (databases 95 / 50 / 53 to 100 / 88 / 95); the man's
        database shoebox 43 clip frames to 0 and the woman's bigbox 18 to 6 (both passed), her database bigbox 44 to 13
        (still failing); grips 28/30, the databases 10 / 9 / 9
    Scenario: What's left
      Then bigball 0 / 0 / 3 (cradled by the sides); slabs aren't used by bigHold yet
      And a mug's handle needs its bar in thingSdf (and drawn) before a guide can be judged on it
      And the glass's stem fails for the adults on the head and arm into the bowl, not the hand; the kid's other hand
        3.6 cm into the glass at frame 145
