// Bevy v0.20.0 examples/shader_advanced/fullscreen_material.rs in raea's TSX (docs/control.feature FC2d): a
// FullscreenMaterial, assets/shaders/fullscreen_effect.wesl as written in a <FramePass file> (a chromatic aberration):
// its screen_texture the frame as drawn so far, its texture_sampler Bevy's SamplerDescriptor::default(), its uniform
// `settings` { intensity }. The pass runs in Core3d's PostProcess before tonemapping (FullscreenMaterial's default
// schedule_configs). update_intensity sets it each update from Time::elapsed_secs t: its phase offset stays 0 (the
// effect starts at 0, as its Local does), so intensity = (sin(2 t) + 1) / 2 * 0.015. A default Cuboid of srgb(0.8,
// 0.7, 0.6) at the origin, a DirectionalLight of 1000 lux facing -Z, the camera at (0, 0, 5) looking at the origin
// over Bevy's default clear colour, and the text "(T) FullscreenEffect: On" 12 px from the top left, in Bevy's default
// 1280 x 720 window.
import { Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { FramePass } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Srgba } from "raea/color";
import { DirectionalLight } from "raea/light";
import { Cuboid, Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { Transform } from "raea/transform";
import { Text, px } from "raea/ui";

const DT = Math.fround(0.016666667);

function Shot() {
  const f = useCurrentFrame();
  const t = Math.fround(f * DT);
  const phase = Math.fround(t * 2);
  const intensity = Math.fround(Math.fround((Math.fround(Math.sin(phase)) + 1) / 2) * Math.fround(0.015));
  return (
    <>
      <Mesh3d transform={Transform.IDENTITY}>
        <Cuboid half_size={[0.5, 0.5, 0.5]} />
        <StandardMaterial base_color={new Srgba(0.8, 0.7, 0.6)} />
      </Mesh3d>
      <DirectionalLight illuminance={1000} />
      <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} />
      <FramePass file={staticFile("shaders/fullscreen_effect.wesl")} frame="screen_texture" at="before-tonemap" uniforms={{ settings: { intensity } }} />
      <Text position_type="Absolute" top={px(12)} left={px(12)}>(T) FullscreenEffect: On</Text>
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={61} fps={60} width={1280} height={720} />);
