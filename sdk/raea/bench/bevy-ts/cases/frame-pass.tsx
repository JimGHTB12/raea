// gap: SH6
// Two <FramePass>es in order over a white box 4.5 m away. The first, before tone mapping with the prepasses, writes red
// where the depth is under 10 m, green where the normal faces the camera, and blue where nothing was drawn. The second,
// after, swaps red and blue through frame(uv): so the box ends cyan-ish (blue, green) and the clear red.
// probe: 32 32 b>200 g>200 r<40
// probe: 4 4 r>200 g<40 b<40
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { ToneMapping, FramePass } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const probe = `
fn shade(in: FullscreenVertexOutput) -> vec4<f32> {
  if depth(in.uv) > 1e20 { return vec4<f32>(0.0, 0.0, 1.0, 1.0); }
  return vec4<f32>(step(depth(in.uv), u_far()), step(0.5, normal(in.uv).z), 0.0, 1.0);
}`;

const swap = `
fn shade(in: FullscreenVertexOutput) -> vec4<f32> {
  return frame(in.uv).bgra;
}`;

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.872664625997165 } }} />
        <ToneMapping mode="none" />
        <AmbientLight brightness={400} />
        <Mesh3d><Cuboid half_size={[0.5, 0.5, 0.5]} /><StandardMaterial base_color="#ffffff" /></Mesh3d>
        <FramePass wesl={probe} at="before-tonemap" prepass={{ depth: true, normal: true }} uniforms={{ far: 10 }} />
        <FramePass wesl={swap} />
      </>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
