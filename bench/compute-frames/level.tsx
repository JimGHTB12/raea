// bench/compute-frames: an inline <Compute> fills a 64 x 64 render image with the grey its uniform `level` gives,
// f / 20 at frame f, and a Sprite shows it over the whole 128 x 128 frame (check.sh).
import { Composition, registerRoot, useCurrentFrame } from "raea";
import { Canvas3D, Compute, Sprite } from "raea/3d";

const FILL = `
@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
  let l = u_level();
  textureStore(out, vec2<i32>(id.xy), vec4<f32>(l, l, l, 1.0));
}`;

function Shot() {
  const f = useCurrentFrame();
  return (
    <Canvas3D layered clearColor="#000000">
      <Compute wesl={FILL} workgroups={[8, 8, 1]} uniforms={{ level: f / 20 }} images={{ out: "render:level:64x64" }} />
      <Sprite src="render:level:64x64" x={64} y={64} width={64} height={64} scale={2} />
    </Canvas3D>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={12} fps={30} width={128} height={128} />);
