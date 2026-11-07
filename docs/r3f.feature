@spec
Feature: React Three Fiber both ways: an R3F scene drawn by raea, a raea composition drawn in R3F
  Goal, 2026-11-06: "complete r3f support so that raea can also render to r3f or use r3f as input in raea, two way
  complete r3f support, and measure so that it is complete and nothing is missing". The code is sdk/r3f (raea/r3f for
  authors). Both ways go through raea's scene contract, the `data-3d` elements a frame renders: an R3F scene is read
  into those elements (input), and those elements are drawn as a three.js scene (output). The same units, shapes and
  tone mapping hold on both sides, and both are measured: every kind and class has a row (R3F7) and every case is
  held to the other renderer by SSIM (R3F8).

  Rule: R3F as raea's input

    @done
    Scenario: R3F1. An R3F scene in a raea composition
      Given an R3F scene, the JSX a <Canvas> holds (meshes, lights, cameras, useFrame, refs, primitives)
      When a composition puts it inside <R3F camera={...} shadows> from raea/r3f
      Then raea's engine draws it: each frame R3F's reconciler builds the three.js scene off screen (a stand-in
        renderer, no WebGL), raea steps R3F's clock to the frame and reads the scene into raea's parts
      And useFrame runs once a frame, 1/fps a step, from frame 0; a seek back starts again from 0, so a render, a
        seek and a play give the same frame (test/input.test.tsx "the clock")
      And raea's own hooks work inside it (useCurrentFrame, a context from outside the canvas): the context bridge
        re-provides the composition's contexts to R3F's tree

    @done
    Scenario: R3F2. What is read from three.js
      Then input/kinds.ts has a row for every Object3D and Material class three exports (62 in three 0.180):
        41 drawn, 12 near (a note says what differs), 9 with nothing to draw (a bone, a camera that isn't drawn with,
        a sound, a base class, ShadowMaterial)
      And meshes keep their triangles, normals, uvs (v flipped: Bevy counts from the top), vertex colours, geometry
        groups (a mesh a material); skinned and morphed meshes are posed on the CPU each frame; an InstancedMesh is
        Instances while its copies hold still; Points, Lines, Sprites and three's helpers are drawn
      And lights convert by units.ts, so a three light and the raea light it becomes put the same light on screen; a
        HemisphereLight is an ambient light of its colours' average plus half their difference from straight up
      And textures keep their url; a DataTexture or a canvas becomes a PNG; repeat, offset, rotation and samplers hold

  Rule: raea drawn in R3F and three.js

    @done
    Scenario: R3F3. A raea composition inside an R3F <Canvas>
      Given a raea composition's component
      When an R3F app puts <Raea component={Edit} /> inside its <Canvas>
      Then its 3D is drawn by three.js in that canvas: the component renders as under raea (into a page off screen,
        at its frame), and Scene3d draws the page's parts into a three.js scene R3F holds
      And its camera becomes R3F's, its tone mapping, exposure, shadows, fog, environment and background the canvas's
        (camera={false} and settings={false} leave them to the app)
      And its frame follows R3F's clock at its fps, or `frame` pins it

    @done
    Scenario: R3F4. Scene3d, the three.js host, for any three.js app
      Then sdk/r3f/src/three draws a raea page into a three.js scene without React Three Fiber (sdk/playable's host
        is built on it): Scene3d.sync(page, seconds), fit(width, height), apply(renderer)
      And three/kinds.ts has a row for every kind raea's SDK renders (58): 26 drawn, 11 near, 4 settings, 9 effects
        and 8 that are Bevy's own (a WESL shader or frame pass, a compute pass, a Bevy component or resource, an ECS
        world read from the engine's tables, volumetric fog and fog volumes)

    @done
    Scenario: R3F5. raea's solids on the web
      Given a Mesh whose shape is a solid (minus, hull, extrude, Text3D: core/shape's expression)
      Then engine/shape-web compiles core/shape to wasm (tools/shape-wasm.sh, 2.6 MB) and loadShapes() brings it in:
        the solid's own triangles, the same as raea's engine meshes
      And until it loads, a solid waits (it isn't drawn as something else)

    @done
    Scenario: R3F6. Frame effects
      Then three/effects.ts draws raea's effects with three's EffectComposer in Bevy's order: ambient occlusion,
        outlines, depth of field, bloom (energy-conserving, as Bevy's) and motion blur in the scene's light; Bevy's
        grade (fringes, vignette, white balance, levels) before tone mapping; saturation, looks and lenses after
      And <Raea> runs it whenever the composition has effects; effects={false} hands them to the app's own composer

  Rule: Measured

    @done
    Scenario: R3F7. Nothing is left without a row
      Then test/coverage.test.tsx reads the SDK's sources for every kind it renders and fails on one missing from
        three/kinds.ts, draws every kind three.js draws and fails on a warning, and fails on an effect row the
        composer doesn't draw
      And it walks three's exports and fails on an Object3D or Material class missing from input/kinds.ts
      And `bun test` in sdk/r3f runs these with the input and round-trip tests (22 tests)

    @done
    Scenario: R3F8. Each way held to the other renderer
      Given test/parity/fixtures.tsx: R3F scenes (in-*) and raea compositions (out-*) at 640x360
      When tools/parity.sh renders each with `raea render` (native.tsx) and with three.js in headless Chrome
        (test/parity/page.tsx, served by a Vite host), and compares the two by SSIM
      Then every case is at least 0.92 (measured 11-06)
        | case        | what                                                                 | SSIM   |
        | in-basic    | a spinning box (useFrame), PBR and clearcoat, shadows, a point light | 0.9911 |
        | in-lights   | spot, hemisphere and area lights, fog, Lambert and flat Phong        | 0.9793 |
        | in-things   | InstancedMesh, multi-material, edge lines, a sprite                  | 0.9908 |
        | out-basic   | a raea scene, ACES, a moving box                                     | 0.9958 |
        | out-lights  | spot and area lights, fog, Text3D (a solid)                          | 0.9627 |
        | out-bloom   | <Bloom intensity={2}>                                                | 0.9444 |
        | out-grade   | <Grade> saturation, contrast, vignette                               | 0.9748 |
        | out-lens    | <LensDistortion k1={0.15}>                                           | 0.9933 |
      And three's ACES scales its input by 1/0.6 where Bevy's doesn't: the host's exposure takes 0.6 under ACES, and
        an R3F canvas's ACES reads as 0.74 stops more (measured: out-lights 0.927 to 0.963, in-lights 0.927 to 0.979)
