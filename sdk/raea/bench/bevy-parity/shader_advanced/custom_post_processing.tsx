// Bevy v0.20.0 examples/shader_advanced/custom_post_processing.rs in raea's TSX (docs/control.feature FC2d): a
// chromatic aberration over the frame, assets/shaders/post_processing.wesl as written in a <FramePass file>: its
// screen_texture the frame as drawn so far, its texture_sampler Bevy's SamplerDescriptor::default() (nearest), its
// uniform `settings` { intensity }, which update_settings sets each update from Time::elapsed_secs t:
// (sin(sin(t)) * 0.5 + 0.5) * 0.015. The pass runs in Core3d's PostProcess, after tone mapping. A 1 m cube of
// srgb(0.8, 0.7, 0.6) at (0, 0.5, 0) turns by rotate_x(0.55 dt) then rotate_z(0.15 dt) each update (no delta on the
// first), a DirectionalLight of 1000 lux facing -Z, and the camera at (0, 0, 5) looking at the origin over a white
// clear colour, in Bevy's default 1280 x 720 window.
import { Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { FramePass } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Srgba } from "raea/color";
import { DirectionalLight } from "raea/light";
import { Cuboid, Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { Quat, Transform } from "raea/transform";

const DT = Math.fround(0.016666667);

/** The cube's rotation after f updates of `rotate`, each a delta of DT. */
function turned(f: number) {
  let q = Quat.IDENTITY;
  for (let i = 0; i < f; i++) q = Quat.mul(Quat.fromRotationZ(0.15 * DT), Quat.mul(Quat.fromRotationX(0.55 * DT), q));
  return q;
}

function Shot() {
  const f = useCurrentFrame();
  const t = Math.fround(f * DT);
  const intensity = Math.fround((Math.sin(Math.sin(t)) * 0.5 + 0.5) * 0.015);
  return (
    <>
      <Mesh3d transform={{ translation: [0, 0.5, 0], rotation: turned(f), scale: [1, 1, 1] }}>
        <Cuboid half_size={[0.5, 0.5, 0.5]} />
        <StandardMaterial base_color={new Srgba(0.8, 0.7, 0.6)} />
      </Mesh3d>
      <DirectionalLight illuminance={1000} />
      <Camera3d clear_color={{ Custom: new Srgba(1, 1, 1) }} transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} />
      <FramePass file={staticFile("shaders/post_processing.wesl")} frame="screen_texture" uniforms={{ settings: { intensity } }} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={61} fps={60} width={1280} height={720} />);
