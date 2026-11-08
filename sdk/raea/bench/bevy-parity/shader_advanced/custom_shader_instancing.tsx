// Bevy v0.20.0 examples/shader_advanced/custom_shader_instancing.rs in raea's TSX (docs/control.feature FC2d): one
// Cuboid of 0.5 m drawn 100 times in one draw by assets/shaders/instancing.wesl as written, in the camera's
// Transparent3d phase: a <ShaderInstances> whose instance buffer holds each InstanceData (a position vec3 and scale
// f32 read at location 3, a colour [f32; 4] at location 4: 32 bytes). Instance (x, y) for x and y in 0.1 to 1.0 is at
// (10x - 5, 10y - 5, 0), scale 1, coloured LinearRgba::from(Color::hsla(360x, y, 0.5, 1)). The mesh has no material
// (Bevy's entity is only Mesh3d, the instances and NoFrustumCulling), so it draws only through its instances. The
// camera at (0, 0, 15) looking at the origin, in Bevy's default 1280 x 720 window (its NoIndirectDrawing isn't reflected,
// and raea's draw of the instances is direct either way, as the example's DrawMeshInstanced is).
import { Composition, registerRoot, staticFile } from "raea";
import { BevyComponent, Entity, ShaderInstances } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Hsla, LinearRgba } from "raea/color";
import { Transform } from "raea/transform";

const f = Math.fround;

// InstanceData for x in 1..=10, then y in 1..=10 (the example's flat_map order)
const WORDS: number[] = [];
for (let i = 1; i <= 10; i++) {
  for (let j = 1; j <= 10; j++) {
    const [x, y] = [f(i / 10), f(j / 10)];
    const c = LinearRgba.from(new Hsla(f(x * 360), y, 0.5, 1).to_srgba());
    WORDS.push(f(f(x * 10) - 5), f(f(y * 10) - 5), 0, 1, c.red, c.green, c.blue, c.alpha);
  }
}

function Shot() {
  return (
    <>
      <Entity>
        <BevyComponent type="bevy_mesh::components::Mesh3d" value={{ primitive: "Cuboid", half_size: [0.25, 0.25, 0.25] }} />
        <BevyComponent type="bevy_camera::visibility::NoFrustumCulling" value={{}} />
        <ShaderInstances
          shader={staticFile("shaders/instancing.wesl")}
          stride={32}
          attributes={[{ format: "Float32x4", offset: 0, location: 3 }, { format: "Float32x4", offset: 16, location: 4 }]}
          words={WORDS}
        />
      </Entity>
      <Camera3d transform={Transform.fromXyz(0, 0, 15).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
