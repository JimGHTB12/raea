# raea/r3f

React Three Fiber both ways (docs/r3f.feature).

**R3F in raea.** Put an R3F scene in a composition and raea's engine draws it, frame by frame:

```tsx
import { R3F } from "raea/r3f";

export const Edit = () => (
  <R3F camera={{ position: [3, 2.5, 4], fov: 45 }} shadows>
    <ambientLight intensity={0.4} />
    <directionalLight position={[3, 5, 2]} intensity={2.5} castShadow />
    <Spinner /> {/* useFrame steps 1/fps a frame */}
  </R3F>
);
```

In a project, three and @react-three/fiber are its own: `raea add three@0.185.0 @react-three/fiber@9.8.2`.

TSL node materials (`three/webgpu`: a `MeshBasicNodeMaterial` with a `colorNode`, or any library built on TSL) run in
raea too. three's own `WGSLNodeBuilder` compiles the material's graph to WGSL, and raea's engine runs that WGSL
(`src/input/tsl.ts`). The camera and transform come from raea, and every uniform takes this frame's value: TSL's `time`
is R3F's clock, so a uniform you change in `useFrame` moves with it. A node material with no graph of its own (a plain
`MeshStandardNodeMaterial`) is drawn as its classic material. A graph raea can't carry is named in a warning and drawn
flat. That covers a graph needing more than 4 textures or 64 vec4 uniforms, a storage buffer, a render target, or a
vertex attribute other than position, normal, uv and color.

**lettra** (MSDF text on TSL, [lettra.dev](https://lettra.dev/docs)) works in `<R3F>` as plain text.
`test/parity` has cases for it: `in-lettra` for plain text and `in-lettrafx` for `wipe` and `scramble`. Those two read
a vertex attribute of their own (`layoutX`) and a uniform array, which raea carries. In a plain
R3F `<Canvas>`, lettra needs three's `WebGPURenderer` (`gl={async (p) => { const r = new WebGPURenderer(p); await
r.init(); return r; }}`). Inside `<R3F>` it doesn't, because raea draws. Set the atlas's settings (`flipY: false`,
`NoColorSpace`) when you create its texture, as `configureFontTexture` does. Call `configureFontTexture` itself in the
loader's `onLoad`, or WebGPURenderer uploads the picture before it has loaded.

Baking an atlas on an Apple Silicon Mac (`lettra bake`, lettra 0.3.0 with msdf-bmfont-xml 2.7) needs two fixes, both
in lettra's dependencies:
- The `msdfgen` msdf-bmfont-xml ships for macOS is x86-only ("Bad CPU type in executable"). Install the arm64 build with
  `brew install msdfgen`, then link it over the bundled one:
  `ln -sf /opt/homebrew/bin/msdfgen node_modules/msdf-bmfont-xml/bin/darwin/msdfgen.osx`.
- The nested `maxrects-packer` 2.7.4 (`node_modules/msdf-bmfont-xml/node_modules/maxrects-packer`, an ES module) fails
  with "MaxRectsPacker is not a constructor". Move it aside so the top-level CommonJS copy is used:
  `bun add -d maxrects-packer@2.7.3`, then remove or rename the nested folder.
- Then bake with `node`: `node node_modules/lettra/dist/cli.mjs bake Font.ttf --charset "..." --out fonts/Font`.
  Bake a static font, or the kerning is dropped. `test/parity/fonts` holds Inter, under the OFL (`Inter-OFL.txt`).

**three.js shaders.** A `ShaderMaterial` or `RawShaderMaterial` (written by hand, or drei's `shaderMaterial()`) runs in
raea's engine as written. Its GLSL is expanded from three's chunks, translated by naga and run with three's built-ins
(`src/input/shader.ts`, docs/r3f.feature R3F11). A shader can have up to 4 vertex attributes of its own, 64 vec4
uniforms and 4 textures. `MeshMatcapMaterial` and `MeshNormalMaterial` run three's own programs this way.

**Effects.** `@react-three/postprocessing`'s `<EffectComposer>` is read as raea's frame effects (`src/input/post.ts`,
R3F12): Bloom, Vignette, ChromaticAberration, HueSaturation, BrightnessContrast, ToneMapping, DepthOfField, Bokeh,
SMAA, FXAA, SSAO, N8AO, Pixelation and LensDistortion. An effect raea has nothing for (noise, glitch, god rays) is
named in a warning and left out.

**drei.** A typical drei scene draws (`test/parity`'s `in-drei`, R3F13):
- `<Text>`: raea's bundler swaps troika-three-text for a stand-in, and raea lays the words out as its own `<RichText>`.
  The face is found by its family name.
- `<Environment preset>` and `useGLTF`: three's loaders read files and URLs through raea's cache.
- `<ContactShadows>`: its depth and blur passes are done on the CPU.
- `<Float>`: `Math.random` is seeded for each `<R3F>`, so a render equals its play.

**What still differs.**
- `onBeforeCompile` patches aren't run, so drei's `MeshWobbleMaterial` and `MeshDistortMaterial` draw as their base
  material. `MeshTransmissionMaterial` draws with Bevy's transmission, without its chromatic aberration or distortion.
- A render target an app fills itself is drawn empty, unless it is one of the passes `src/input/targets.ts` knows.
- `<Text>` outlines, strokes and `curveRadius` are left out.
- Phong's highlight has GGX's shape, not Blinn-Phong's, and a specular colour brighter than Bevy's highest F0 (0.16)
  glints less.
- HemisphereLight and LightProbe are exact only for surfaces facing their light or side-on.

`src/input/kinds.ts` lists every three.js class and what raea draws for it.

**raea in R3F.** Draw a raea composition inside any R3F `<Canvas>`:

```tsx
import { Canvas } from "@react-three/fiber";
import { Raea, loadShapes } from "@raea/r3f";

loadShapes(await import("./raea_shape_web.js")); // solids, Text3D: from the release, or tools/shape-wasm.sh
<Canvas shadows><Raea component={Edit} fps={30} /></Canvas>
```

Its camera, tone mapping (raea's default, TonyMcMapface, drawn with Bevy's own table), fog, environment and effects
become the canvas's. Without React, `Scene3d` (`@raea/r3f/three`) draws a raea page into any three.js scene.

The shape module, `raea_shape_web.js` and `raea_shape_web_bg.wasm`, is attached to the GitHub releases: every engine
release (`vVERSION`, each with its own shape code) and a `web-SHA` pre-release for public commits between them
(`https://github.com/shinyobjectz/raea/releases/download/TAG/raea_shape_web.js`). Serve the two files with
your app: GitHub sends them as downloads, so a page can't import them from there.

What only raea's engine draws (WESL and WGSL shaders, Bevy components and resources, volumetric fog) isn't three.js's
to draw: `Scene3d` names each one it leaves out. A page that needs them plays the composition with `@raea/web`, raea's
engine in the browser on WebGPU (`sdk/web/tools/parity.sh` holds each to `raea render`).

**raea as R3F source.** Write a composition out as a plain R3F module, with no raea needed to run it:

```sh
bun sdk/r3f/tools/export.ts edit.tsx intro Intro.tsx --all   # or toR3FSource() from @raea/r3f/export
```

`Intro.tsx` needs only react, three and @react-three/fiber: the scene as JSX, what moves as a table a frame, and a
default export that is a `<Canvas>`. Solids are written as their triangles. When the scene has frame effects or Tony,
`raea-effects.ts` is written beside it (the composer `<Raea>` uses, three only).

**Measured.** `bun test` checks that every raea kind and every three.js class has a row in `src/three/kinds.ts` and
`src/input/kinds.ts`, which say what is drawn exactly, near (and what differs) or left to raea's engine.
`tools/parity.sh` renders each case in `test/parity` with raea and with three.js in headless Chrome (it serves the page
itself, tools/serve.ts) and compares them by SSIM (at least 0.92): R3F scenes in raea, raea in R3F, and raea exported
as R3F source.
