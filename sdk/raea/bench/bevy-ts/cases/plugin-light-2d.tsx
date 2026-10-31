// gap: PL7
// bevy_light_2d through raea/plugins/light_2d (docs/plugins.feature PL7): a white sprite filling the 2D overlay
// (pixels from the frame's top left, +y up) under no ambient light, lit only by a red point light of radius 20 on
// its left: the sprite is red at the light and black on the right, past the light's reach.
// build: plugins
// frame: 1
// probe: 14 32 r>120 g<60 b<60
// probe: 56 32 r<20 g<20 b<20
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { BevyComponent, Canvas3D, Entity } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Transform } from "raea/transform";
import { Lighting2d, PointLight2d } from "raea/plugins/light_2d";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.785398 } }} />
        <Entity position={[32, -32, 0]}>
          <BevyComponent type="raea_sys_sprite::Draws2d" value={{}} />
          <BevyComponent type="bevy_sprite::sprite::Sprite" value={{ color: { Srgba: { red: 1, green: 1, blue: 1, alpha: 1 } }, custom_size: [64, 64] }} />
        </Entity>
        <Entity position={[14, -32, 1]}>
          <PointLight2d value={{ color: { LinearRgba: { red: 1, green: 0, blue: 0, alpha: 1 } }, intensity: 3, radius: 20, falloff: 0, cast_shadows: false }} />
        </Entity>
        <Entity>
          <Lighting2d value={{ color: [1, 1, 1], brightness: 0 }} />
        </Entity>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
