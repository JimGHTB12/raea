// gap: PL7
// bevy_sprite3d through raea/plugins/sprite3d (docs/plugins.feature PL7): SpriteQuad, raea's reflected sprite on a
// quad in the 3D scene (sprite3d's `calls`), its image a 2 x 2 checker made in code (white top left, sampled nearest
// as pixel art is) at one pixel a metre, unlit, facing a camera 3 m away with no tone mapping: a 2 m square whose top left and bottom
// right quarters are white and whose top right is black.
// build: plugins
// frame: 1
// probe: 19 19 r>200 g>200 b>200
// probe: 45 45 r>200 g>200 b>200
// probe: 45 19 r<40 g<40 b<40
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D, Entity, ToneMapping } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Transform } from "raea/transform";
import { SpriteQuad } from "raea/plugins/sprite3d";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D>
        <Camera3d transform={Transform.fromXyz(0, 0, 3).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.785398 } }} />
        <ToneMapping mode="none" />
        <Entity position={[0, 0, 0]}>
          <SpriteQuad value={{ image: { width: 2, height: 2, pixels: ["#ffffff", "#000000", "#000000", "#ffffff"], nearest: true }, pixels_per_metre: 1, unlit: true }} />
        </Entity>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
