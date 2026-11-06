// gap: PL7
// bevy_smud through raea/plugins/smud (docs/plugins.feature PL7): one signed-distance circle drawn by smud in the
// scene's 2D overlay (Draws2d brings its camera up; pixels from the frame's top left, +y up), its SDF a .wesl file
// beside the case and its fill smud's simple one. A green circle of radius 16 at the frame's centre, black corners.
// build: plugins
// frame: 1
// probe: 32 32 g>150 r<80 b<80
// probe: 2 2 r<20 g<20 b<20
// probe: 61 61 r<20 g<20 b<20
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { BevyComponent, Entity } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Transform } from "raea/transform";
import { SmudShape } from "raea/plugins/smud";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.785398 } }} />
        <Entity position={[32, -32, 0]}>
          <BevyComponent type="raea_sys_sprite::Draws2d" value={{}} />
          <SmudShape
            value={{
              color: { Srgba: { red: 0, green: 1, blue: 0, alpha: 1 } },
              sdf: "smud-circle.wesl",
              fill: { Uuid: "cef2d2c2-1a68-4418-a815-5a8ac361f140" },
              bounds: { half_size: [16, 16] },
              params: [0, 0, 0, 0],
              blend_mode: "Alpha",
              extra_bounds: 2,
            }}
          />
        </Entity>
      </>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
