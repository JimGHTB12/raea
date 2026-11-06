// gap: SH1
// A shader over a lit standard surface (<MeshStandardMaterial shader>, raea-sys-shader): a white
// sphere lit from its left only. Its `surface` hook makes the base colour red before Bevy's lighting, and its `lit` hook
// adds the uniform `blue` after it, so the red is lit (bright on the left, dark on the right) and the blue is even.
// probe: 22 32 r>110 g<60 b>150
// probe: 43 32 r<70 g<60 b>150
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { ToneMapping, Sphere } from "raea/3d";
import { Mesh3d } from "raea/mesh";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { ExtendedMaterial, ShaderExtension, StandardMaterial } from "raea/pbr";

const wesl = `
fn surface(in: VertexOutput, pbr: ptr<function, PbrInput>) {
  (*pbr).material.base_color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
fn lit(in: VertexOutput, pbr: PbrInput, color: vec4<f32>) -> vec4<f32> {
  return vec4<f32>(color.r, color.g, u_blue(), 1.0);
}`;

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 0, 4).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 1.0471975511966 } }} />
        <ToneMapping mode="none" />
        <AmbientLight brightness={0} />
        <DirectionalLight transform={Transform.fromXyz(-5, 0, 0.5).lookingAt([0, 0, 0])} illuminance={8000} />
        <Mesh3d>
          <Sphere radius={1.2} />
          <ExtendedMaterial base={<StandardMaterial base_color="#ffffff" perceptual_roughness={0.9} />} extension={<ShaderExtension {...({ wesl, uniforms: { blue: 1 } })} />} />
        </Mesh3d>
      </>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
