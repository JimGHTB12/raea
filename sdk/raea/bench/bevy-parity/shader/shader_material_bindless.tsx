// Bevy v0.20.0 examples/shader/shader_material_bindless.rs in raea's TSX (docs/control.feature FC0a, FC2a): a 1 m cube
// at (-2, 0.5, 0) and a default Cylinder at (2, 0.5, 0) with the example's own Material, assets/shaders/
// bindless_material.wesl as written: its color uniform (LinearRgba::BLUE, ::RED) times a texture (Bevy's dark and
// light logos). raea's Material isn't bindless, so the shader's @if(BINDLESS) set compiles out and its @else set binds,
// as Bevy's does on a GPU without bindless. The camera at (-2, 2.5, 5), in Bevy's default 1280 x 720 window.
import { Composition, registerRoot, staticFile } from "raea";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Cylinder, Mesh3d } from "raea/mesh";
import { Material } from "@raea/render/bevy3d"; // raea/pbr once scene-loop lands (its pbr.ts commit)

const SHADER = staticFile("shaders/bindless_material.wesl");

function Shot() {
  return (
    <>
      <Mesh3d transform={Transform.fromXyz(-2, 0.5, 0)}>
        <Cuboid half_size={[0.5, 0.5, 0.5]} />
        <Material fragment={SHADER} uniforms={{ material_color: { base_color: [0, 0, 1, 1] } }} textures={{ material_color_texture: staticFile("branding/bevy_logo_dark.png") }} />
      </Mesh3d>
      <Mesh3d transform={Transform.fromXyz(2, 0.5, 0)}>
        <Cylinder radius={0.5} half_height={0.5} />
        <Material fragment={SHADER} uniforms={{ material_color: { base_color: [1, 0, 0, 1] } }} textures={{ material_color_texture: staticFile("branding/bevy_logo_light.png") }} />
      </Mesh3d>
      <Camera3d transform={Transform.fromXyz(-2, 2.5, 5).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
