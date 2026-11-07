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
        42 drawn, 12 near (a note says what differs), 8 with nothing to draw (a bone, a camera that isn't drawn with,
        a sound, a base class)
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
      And three/kinds.ts has a row for every kind raea's SDK renders (58): 27 drawn, 11 near, 4 settings, 9 effects
        and 7 that are Bevy's own (a WESL shader or frame pass, a compute pass, a Bevy component or resource,
        volumetric fog and fog volumes)
      And an ECS world (<Entities>) is drawn: its meshes and their transforms come from raea/ecs's own store in
        JavaScript (sdk/app's scene(), the engine's composition of a child under its parent), each mesh's shape by
        three's geometry or core/shape's wasm
      And the 7 that are Bevy's own aren't three.js's to draw: WESL and WGSL run on Bevy's renderer, components and
        resources are Bevy's data, and the fogs are raymarched through Bevy's shadow maps and depth. A page that needs
        them uses raea-web, raea's engine itself in the browser on WebGPU (R3F10); Scene3d names each one it leaves out
        (a warning, and its row)
      And a Model with `at` is a copy of the model at each placement, as raea's engine draws it
      And raea's default tone mapping, TonyMcMapface, is drawn with Bevy's own LUT (sdk/playable's tony.bin) in a
        pass in place of three's output pass; ACES, AgX, Reinhard and none are three's own

    @done
    Scenario: R3F5. raea's solids on the web
      Given a Mesh whose shape is a solid (minus, hull, extrude, Text3D: core/shape's expression)
      Then engine/shape-web compiles core/shape to wasm (tools/shape-wasm.sh, 2.6 MB) and loadShapes() brings it in:
        the solid's own triangles, the same as raea's engine meshes
      And until it loads, a solid waits (it isn't drawn as something else)
      And the public repository's releases carry the module, so an app needn't build Rust: tools/release-web.sh builds
        raea_shape_web.js and raea_shape_web_bg.wasm (and raea-web's raea_web.js and raea_web_bg.wasm, with SHA256SUMS)
        from a commit and attaches them to its GitHub release: every engine release (tag vVERSION; 0.1.0-canary.1 and
        canary.2 were given their own shape code's module on 11-06, without raea-web, which came after them), and a
        pre-release web-SHA for a public commit between them (SHA the public commit's short hash), at
        https://github.com/shinyobjectz/raea/releases/download/TAG/raea_shape_web.js. GitHub serves them as
        downloads (no CORS), so an app serves the two files itself and calls loadShapes("/raea_shape_web.js"), or
        imports the glue and passes the module: loadShapes(await import("./raea_shape_web.js"))

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
      And it fails on a kind three/kinds.ts calls "engine" that has no raea-web parity case (R3F10)
      And `bun test` in sdk/r3f runs these with the input, round-trip and source-export tests (27 tests)
      And a ShadowMaterial is raea's <Mesh shadow="catch">: a surface that shows only the shadows falling on it, in
        its material's colour and opacity (systems/material shadow_catch.rs in the engine, three's ShadowMaterial in
        Scene3d)

    @done
    Scenario: R3F8. Each way held to the other renderer
      Given test/parity/fixtures.tsx: R3F scenes (in-*), raea compositions (out-*) and the same compositions written
        as R3F source (src-*, R3F9), at 640x360
      When tools/parity.sh renders each with `raea render` (native.tsx) and with three.js in headless Chrome
        (test/parity/page.tsx, which tools/serve.ts builds and serves itself, with the wasm tools/shape-wasm.sh builds),
        and compares the two by SSIM
      Then every case is at least 0.92 (measured 11-07)
        | case        | what                                                                 | SSIM   |
        | in-basic    | a spinning box (useFrame), PBR and clearcoat, shadows, a point light | 0.9917 |
        | in-lights   | spot, hemisphere and area lights, fog, Lambert and flat Phong        | 0.9795 |
        | in-things   | InstancedMesh, multi-material, edge lines, a sprite                  | 0.9917 |
        | in-catch    | a ShadowMaterial floor: only the box's shadow over the background    | 0.9966 |
        | in-lettra   | lettra's MSDF text (TSL) and a TSL gradient on its clock             | 0.9994 |
        | in-lettrafx | lettra's wipe and scramble (an attribute, a uniform array)           | 0.9992 |
        | in-shader   | ShaderMaterials: a rippled sphere, points sized by an attribute      | 0.9989 |
        | in-post     | <EffectComposer>: HueSaturation, Vignette, ToneMapping ACES          | 0.9451 |
        | in-drei     | drei's Text, Environment preset, ContactShadows, Float, useGLTF      | 0.9932 |
        | in-approx   | matcap, normal, Phong specular, dashes, clipping, hemisphere, probe  | 0.9848 |
        | out-basic   | a raea scene, ACES, a moving box                                     | 0.9958 |
        | out-tony    | the same scene in raea's default tone mapping, TonyMcMapface         | 0.9961 |
        | out-lights  | spot and area lights, fog, Text3D (a solid)                          | 0.9633 |
        | out-bloom   | <Bloom intensity={2}>                                                | 0.9295 |
        | out-grade   | <Grade> saturation, contrast, vignette                               | 0.9738 |
        | out-lens    | <LensDistortion k1={0.15}>                                           | 0.9922 |
        | out-catch   | <Mesh shadow="catch">, three's ShadowMaterial                        | 0.9972 |
        | out-entities| <Entities>: a raea/ecs world, a ship and its gun turned by a system  | 0.9972 |
        | src-basic   | out-basic exported as R3F source, drawn by a plain <Canvas>          | 0.9958 |
        | src-tony    | out-tony exported (Tony through raea-effects.ts)                     | 0.9961 |
        | src-lights  | out-lights exported as R3F source (Text3D as its triangles)          | 0.9633 |
        | src-bloom   | out-bloom exported (raea-effects.ts)                                 | 0.9295 |
        | src-grade   | out-grade exported                                                   | 0.9738 |
        | src-lens    | out-lens exported                                                    | 0.9922 |
        | src-catch   | out-catch exported                                                   | 0.9972 |
        | src-entities| out-entities exported                                                | 0.9972 |
      And three r181 rewrote UnrealBloomPass (mrdoob/three.js#31528): three/effects.ts puts r180's kernels, composite
        and blend back on three's pass, so out-bloom holds to the bloom raea's matches (three's frame equals r180's at
        SSIM 0.999998)
      And three's ACES scales its input by 1/0.6 where Bevy's doesn't: the host's exposure takes 0.6 under ACES, and
        an R3F canvas's ACES reads as 0.74 stops more (measured: out-lights 0.927 to 0.963, in-lights 0.927 to 0.979)

  Rule: raea written as R3F

    @done
    Scenario: R3F9. A raea composition exported as React Three Fiber source
      Given a raea composition
      When `bun sdk/r3f/tools/export.ts EDIT.tsx COMPOSITION OUT.tsx [--from N] [--frames N | --all]` runs, or
        toR3FSource(component, options) from @raea/r3f/export
      Then OUT.tsx is a TSX module that needs only react, three and @react-three/fiber: the scene as JSX (meshes with
        three's geometry and materials, lights, fog, background, environment, models by url), what moves as a table a
        frame (position, rotation, scale, visibility, materials, lights, the camera) applied in useFrame, and a default
        export that is a <Canvas> with raea's camera, tone mapping, exposure and shadows
      And a solid (minus, hull, extrude, Text3D) is written as its triangles, meshed by core/shape's wasm
      And when the scene has frame effects or raea's Tony tone mapping, raea-effects.ts is written beside it: the
        composer <Raea> draws them with (three/effects.ts and Bevy's Tony table, needing only three), run by the module
        after each frame's updates; effects that only raea's engine draws (three/kinds.ts "engine") are named in the
        header
      And test/export.test.tsx checks what is written; the src-* cases (R3F8) draw it against raea's render

  Rule: What three.js can't draw, raea-web does

    @done
    Scenario: R3F10. The engine's own kinds held to `raea render` in the browser
      Given sdk/web/test/parity/cases.ts: a case for each kind three/kinds.ts calls "engine", and <Entities>
      When sdk/web/tools/parity.sh renders each with `raea render` and with raea-web (sdk/web, raea's engine on WebGPU)
        in headless Chrome (tools/shoot.ts), and compares the two by SSIM
      Then every case is at least 0.92, and a blank web frame fails whatever its SSIM (measured 11-06)
        | case           | kinds                     | SSIM   |
        | surface-shader | SurfaceShader             | 0.9998 |
        | frame-pass     | FramePass                 | 1.0000 |
        | compute        | Compute                   | 1.0000 |
        | component      | BevyComponent             | 1.0000 |
        | resource       | BevyResource              | 0.9999 |
        | entities       | Entities                  | 0.9760 |
        | fog            | VolumetricFog, FogVolume  | 0.9993 |
      And the first five are sdk/raea/bench/bevy-ts's cases (shader-scene, frame-pass, compute, reflect-on-camera,
        reflect-keyed-resource-still, 64x64); entities and fog are 320x180
      And a page's files (a glTF, a picture) are fetched from assets/ beside the page, Bevy's asset folder on the web
      And measuring it fixed raea-web: a model's file is fetched rather than looked for on disk (runtime/scene3d
        model.rs), a black page background is the 3D's clear rather than a cover over it (rgb(0, 0, 0) was read as
        see-through, here and in Scene3d), and a layered <Canvas3D>'s empty slot isn't drawn

  Rule: What R3F apps are made of draws in raea

    Goal, 2026-11-06: "finish the r3f": custom GLSL shaders, @react-three/postprocessing, the materials and lights that
    were approximate, and a drei-heavy scene, each read by <R3F> and held to three.js by parity (R3F8).

    @done
    Scenario: R3F11. three.js shaders run in raea's engine
      Given a ShaderMaterial or RawShaderMaterial in an <R3F> scene (written by hand, or drei's shaderMaterial())
      When raea reads it (src/input/shader.ts)
      Then its GLSL goes as three would compile it: the #includes expanded from three's ShaderChunk, its defines in
        front, the #ifdef blocks its defines settle decided, its uniforms as this frame's numbers and its textures as
        pictures (a picture made in code as a PNG data: URL)
      And raea-sys-three-shader translates it with naga and runs it with three's built-ins (the matrices, the camera,
        a colour and uvs on every vertex, up to 4 attributes of its own, 64 vec4 uniforms, 4 textures)
      And a shader that includes three's tone mapping is tone-mapped by Bevy's, one that doesn't is shown as three's
        canvas shows it
      And in-shader (a sphere rippled by a uniform set in useFrame, points sized by their own attribute): SSIM 0.9989

    @done
    Scenario: R3F12. @react-three/postprocessing's <EffectComposer> as raea's frame effects
      Given an <EffectComposer> in an <R3F> scene
      When raea reads its effects (src/input/post.ts)
      Then each becomes the raea effect that does the same thing: Bloom; Vignette, ChromaticAberration,
        HueSaturation and BrightnessContrast in one Grade; ToneMapping; DepthOfField and Bokeh; SMAA and FXAA; SSAO
        and N8AO; Pixelation; LensDistortion
      And one raea has nothing for (noise, glitch, god rays ...) is named once and left out (kinds.ts's EFFECTS
        decides every effect postprocessing exports, tested)
      And a composer's frame is tone-mapped only by its ToneMapping effect, as in three
      And in-post (HueSaturation, Vignette, ToneMapping ACES): SSIM 0.945

    @done
    Scenario: R3F13. A drei-heavy scene: <Text>, <Environment preset>, <ContactShadows>, <Float>, useGLTF
      Given test/parity's in-drei case: a knot loaded by useGLTF in a <Float>, troika's <Text> in Inter, drei's
        <ContactShadows> under it and <Environment preset="city"> lighting it
      When raea renders it
      Then three's loaders read their files where a native render has no fetch: FileLoader by the `asset.read` native
        call (a file, or an http(s) URL through raea's cache), so a glTF is parsed in JS; a DataTextureLoader (HDR,
        EXR) hands back the cached file, which raea's engine loads as <Environment src>
      And what an app draws into a render target is done on the CPU where it's a pass raea knows (src/input/targets.ts):
        a MeshDepthMaterial override from an orthographic camera and three's 9-tap blurs, which are drei's
        <ContactShadows>; anything else drawn into a target is named and left empty
      And a mesh whose transform mirrors it is drawn with its triangles rewound (three flips the front face; Bevy
        culls as wound)
      And raea's bundler gives a composition troika-three-text as sdk/r3f/src/input/troika.ts (engine/bundle's
        STAND_INS): its Text holds troika's settings and draws nothing, and src/input/text.ts reads any troika Text as
        raea's <RichText> (size, the face's family from its name table, weight, colour, anchors, maxWidth)
      And a root whose parts mounted late (they suspended) starts again from frame 0 once its loads are cached, and
        Math.random is seeded per root, so drei's <Float> and three's uuids are the same however often a seek restarts
        it (render equals play)
      And in-drei: SSIM 0.993 (Float at speed 0 in the case: three's page draws its phase at random)

    @done
    Scenario: R3F14. The approximate kinds made exact or closer
      Given test/parity's in-approx case
      When raea renders it
      Then MeshMatcapMaterial and MeshNormalMaterial run three's own ShaderLib programs (R3F11), so they look as three
        draws them
      And Phong's specular colour is its F0 (three's Blinn-Phong): Bevy's reflectance and specular tint
      And a LineDashedMaterial is cut into its dashes by the line's distances (computeLineDistances), as three's shader
        cuts it
      And clipping planes (a material's with localClippingEnabled, the renderer's, a ClippingGroup's) cut the mesh's
        triangles, so what three discards isn't drawn (src/input/clip.ts; clipIntersection keeps what is inside any one)
      And drei's MeshTransmissionMaterial draws with Bevy's transmission (its own uniform's, as three's is 0 under it)
      And a LightProbe is an ambient light of its constant band and a light along its linear band (exact for surfaces
        facing that light or sideways, as the hemisphere light is), and its constant band is scaled as three's
        irradiance is (0.886, where 0.282 made it a third as bright)
      And in-approx: SSIM 0.9848 (Phong's F0 is held at Bevy's highest, 0.16, so its orange highlight glints less)
