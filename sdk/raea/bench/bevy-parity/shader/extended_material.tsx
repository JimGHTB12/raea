// Bevy v0.20.0 examples/shader/extended_material.rs in raea's TSX (docs/control.feature FC0a, FC2b): a red unit sphere
// whose ExtendedMaterial runs the example's own shader file, assets/shaders/extended_material.wesl, as written: its
// own @binding(100) uniform struct (quantize_steps: u32, 1), its imports and its deferred branch. It copies red into
// blue before lighting, quantizes the lit colour and doubles it after post-processing, under a DirectionalLight turning
// about Y at 1 rad/s, in Bevy's default 1280 x 720 window.
import { Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Mesh3d, Sphere } from "raea/mesh";
import { ExtendedMaterial, ShaderExtension, StandardMaterial } from "raea/pbr";
import { DirectionalLight } from "raea/light";

function Shot() {
  // rotate_things: rotate_y by each update's delta_secs (none on the first)
  const a = useCurrentFrame() / 60;
  const light = Transform.fromXyz(1, 1, 1).lookingAt([0, 0, 0]);
  return (
    <>
      <Mesh3d transform={Transform.fromXyz(0, 0.5, 0)}>
        <Sphere radius={1} />
        <ExtendedMaterial base={<StandardMaterial base_color="#ff0000" />} extension={<ShaderExtension fragment={staticFile("shaders/extended_material.wesl")} uniforms={{ quantize_steps: 1 }} />} />
      </Mesh3d>
      <DirectionalLight transform={{ translation: light.translation, rotation: Quat.mul(Quat.fromRotationY(a), light.rotation), scale: [1, 1, 1] }} />
      <Camera3d transform={Transform.fromXyz(-2, 2.5, 5).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={61} fps={60} width={1280} height={720} />);
