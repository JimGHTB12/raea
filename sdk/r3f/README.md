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

In a project, three and @react-three/fiber are its own: `raea add three@0.180.0 @react-three/fiber@9.8.2`.

**raea in R3F.** Draw a raea composition inside any R3F `<Canvas>`:

```tsx
import { Canvas } from "@react-three/fiber";
import { Raea, loadShapes } from "@raea/r3f";

loadShapes(await import("./raea_shape_web.js")); // solids, Text3D: tools/shape-wasm.sh builds it
<Canvas shadows><Raea component={Edit} fps={30} /></Canvas>
```

Its camera, tone mapping, fog, environment and effects become the canvas's. Without React, `Scene3d` (`@raea/r3f/three`)
draws a raea page into any three.js scene.

**Measured.** `bun test` checks that every raea kind and every three.js class has a row in `src/three/kinds.ts` and
`src/input/kinds.ts`, which say what is drawn exactly, near (and what differs) or left to raea's engine.
`tools/parity.sh` renders each case in `test/parity` with raea and with three.js in Chrome and compares them by SSIM
(at least 0.92).
