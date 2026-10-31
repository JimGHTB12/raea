// gap: G20
// A material written in WGSL from TSX (<ShaderMesh>, raea-render's generic ShaderMaterial: one AsBindGroup of uniform slots
// and textures, its fragment shader picked per material in specialize): a 3 m square whose red is its uv.x times the
// uniform `gain` and whose blue is the uniform colour `tint`'s; a 2-pixel picture made in code as the texture `pic`
// puts green on its top half only.
// probe: 10 44 r<100 b>150 g<80
// probe: 54 44 r>150 b>150 g<80
// probe: 32 20 g>150
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D, ShaderMesh } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const wgsl = `
fn shade(in: VertexOutput) -> vec4<f32> {
  let pic = textureSample(tex_pic, samp_pic, in.uv);
  return vec4<f32>(in.uv.x * u_gain(), pic.g, u_tint().b, 1.0);
}`;

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <ShaderMesh
          shape={{ primitive: "Rectangle", half_size: [1.5, 1.5] }}
          wgsl={wgsl}
          uniforms={{ gain: 1, tint: "#0000ff" }}
          textures={{ pic: { width: 1, height: 2, pixels: ["#00ff00", "#000000"], nearest: true } }}
        />
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
