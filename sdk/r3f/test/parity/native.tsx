// The parity cases as raea compositions, for `raea render native.tsx --composition in-basic --frame 24`: an R3F scene
// through <R3F>, a raea scene as it is.
import { Composition, registerRoot } from "raea";
import { R3F } from "raea/r3f";
import { INPUTS, OUTPUTS, SIZE } from "./fixtures";

registerRoot(() => (
  <>
    {Object.entries(INPUTS).map(([k, c]) => (
      <Composition key={`in-${k}`} id={`in-${k}`} component={() => <R3F {...c.canvas}>{c.scene()}</R3F>} width={SIZE.width} height={SIZE.height} fps={SIZE.fps} durationInFrames={c.frame + 1} />
    ))}
    {Object.entries(OUTPUTS).map(([k, c]) => (
      <Composition key={`out-${k}`} id={`out-${k}`} component={c.component} width={SIZE.width} height={SIZE.height} fps={SIZE.fps} durationInFrames={c.frame + 1} />
    ))}
  </>
));
