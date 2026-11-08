// Bevy v0.20.0 examples/shader/pipeline_constants.rs in raea's TSX (docs/control.feature FC0a, FC2a): three 200 x 150
// Rectangles at x = -220, 0, 220 (y 20), each with the example's own Material2d, assets/shaders/
// pipeline_constants.wesl as written, its `override LEVELS` set to 2, 4 and 8 in its pipeline (as the example's
// specialize pushes it into descriptor.fragment.constants): one gradient posterized into that many steps. Under each a
// Text2d "N levels" at y -65 (Bevy's default font, 20 px, centred), the clear colour Bevy's default.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { Transform } from "raea/transform";
import { Mesh2d, Rectangle } from "raea/sprite";
import { Text2d } from "raea/ui";
import { Material } from "@raea/render/bevy3d"; // raea/pbr once scene-loop lands (its pbr.ts commit)

const SHADER = staticFile("shaders/pipeline_constants.wesl");

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        {[2, 4, 8].map((levels, i) => (
          <Mesh2d key={`q${i}`} transform={Transform.fromXyz((i - 1) * 220, 20, 0)}>
            <Rectangle half_size={[100, 75]} />
            <Material fragment={SHADER} constants={{ LEVELS: levels }} />
          </Mesh2d>
        ))}
      </Canvas3D>
      {[2, 4, 8].map((levels, i) => (
        <Text2d key={`t${i}`} transform={Transform.fromXyz((i - 1) * 220, -65, 0)}>
          {`${levels} levels`}
        </Text2d>
      ))}
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
