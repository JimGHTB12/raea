// gap: SH10
// <Compute> (docs/shaders.feature SH10): a compute shader run before the cameras draw, writing a "render:NAME" image
// that a <ShaderMesh>'s texture reads by name. Its one 4 x 4 workgroup fills the 4 x 4 image: the left half from the
// uniform `red`, the right half from its buffer `cells` ([0, 1] at the start: green) and its buffer `count`, which it
// adds one to each frame (kept from frame to frame), as blue: by frame 1 it has run at least once before, so blue is full.
//   left box: the image's left texel, red; right box: its right texel, cyan.
// frame: 1
// probe: 17 32 r>200 g<40 b<40
// probe: 47 32 r<40 g>200 b>200
// probe: 32 4 r<40 g<40 b<40
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D, Compute, ShaderMesh, ToneMapping } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const fill = `
@compute @workgroup_size(4, 4, 1)
fn main(@builtin(local_invocation_index) i: u32, @builtin(global_invocation_id) id: vec3<u32>) {
  let n = count[0];
  storageBarrier();
  if i == 0u { count[0] = n + 1.0; }
  var c = vec4<f32>(u_red(), 0.0, 0.0, 1.0);
  if id.x >= 2u { c = vec4<f32>(cells[0], cells[1], min(n, 1.0), 1.0); }
  textureStore(out, vec2<i32>(id.xy), c);
}`;

const read = (u: number) => `
fn shade(in: VertexOutput) -> vec4<f32> {
  return textureSampleLevel(tex_pic, samp_pic, vec2<f32>(${u}, 0.5), 0.0);
}`;
const pic = { src: "render:cgrid:4x4", sampler: { filter: "nearest" } };

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D>
        <Camera3d transform={Transform.fromXyz(0, 0, 6).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.872664625997165 } }} />
        <ToneMapping mode="none" />
        <Compute wesl={fill} workgroups={[1]} uniforms={{ red: 1 }} buffers={{ cells: [0, 1], count: { size: 1 } }} images={{ out: "render:cgrid:4x4" }} />
        <ShaderMesh shape="box(1, 1, 1)" position={[-1.4, 0, 0]} wesl={read(0.125)} textures={{ pic }} />
        <ShaderMesh shape="box(1, 1, 1)" position={[1.4, 0, 0]} wesl={read(0.875)} textures={{ pic }} />
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={3} fps={30} width={64} height={64} />);
