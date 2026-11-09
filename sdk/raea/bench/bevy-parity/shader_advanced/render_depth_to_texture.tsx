// Bevy v0.20.0 examples/shader_advanced/render_depth_to_texture.rs in raea's TSX (docs/control.feature FC2d): a
// depth-only camera (order -1, RenderTarget::None of 256 x 256, Msaa off, DepthPrepass) at (-4, -5, 5) looking at the
// origin, its depth copied after its prepass into a 256 x 256 Depth32Float image (a <DepthCopy>), which a 4 m plane
// facing +Z at (10, 4, 0), scaled 2.5, shows through assets/shaders/show_depth_texture_material.wesl as written (its
// texture_depth_2d and sampler_comparison). A 3 m white cube at the origin turned each update by rotate_x(1.5 dt),
// rotate_y(1.1 dt) and rotate_z(-1.3 dt) (dt 1/60 s, none on the first update), Bevy's default PointLight at (5, 6, 7),
// the main camera at (5, 2, 30) looking at (5, 2, 0) with Msaa off, a LIME cone gizmo (radius 1, height 3, Bevy's
// default resolution of 5) at the depth camera turned a quarter about its X, and "Use WASD to move the secondary camera"
// 12 px from the top left, in Bevy's default 1280 x 720 window. The camera moves only under input.
import { Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { BevyComponent, DepthCopy, Gizmo } from "raea/3d";
import { Camera3d } from "raea/camera";
import { PointLight } from "raea/light";
import { Cuboid, Mesh3d, Plane3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { Quat, Transform } from "raea/transform";
import { Text, px } from "raea/ui";
import { Material } from "@raea/render/bevy3d";

const f = Math.fround;
const DT = f(1 / 60);
const DEPTH = "render:depth:256x256:depth";
const LIME = "#00ff00";

/** glam's f32 Quat::from_rotation_{x,y,z} and Quat * Quat */
const axis = (i: number, a: number): Quat => {
  const h = f(a * 0.5);
  const q: Quat = [0, 0, 0, f(Math.cos(h))];
  q[i] = f(Math.sin(h));
  return q;
};
const mul = (a: Quat, b: Quat): Quat => Quat.mul(a, b).map(f) as Quat;

/** rotate_cube: each update after the first, rotate_x, rotate_y then rotate_z by its delta (each premultiplied) */
function cubeAt(frame: number): Quat {
  let q: Quat = [0, 0, 0, 1];
  for (let n = 0; n < frame; n++) for (const [i, w] of [[0, 1.5], [1, 1.1], [2, -1.3]] as const) q = mul(axis(i, f(w * DT)), q);
  return q;
}

/** Bevy's primitive_3d(&Cone { radius: 1, height: 3 }) at the depth camera: its base circle's 5 points closed, each
 * point but the first joined to the apex, then the circle's 5 sides */
function cone(): [[number, number, number], [number, number, number]][] {
  const cam = Transform.fromXyz(-4, -5, 5).lookingAt([0, 0, 0]);
  const rot = Quat.mul(cam.rotation, Quat.fromRotationX(Math.PI / 2));
  const at = (v: [number, number, number]) => Quat.mulVec3(rot, v).map((x, i) => x + cam.translation[i]) as [number, number, number];
  const apex = at([0, 1.5, 0]);
  const circle = [0, 1, 2, 3, 4, 5].map((p) => at([Math.sin((p * 2 * Math.PI) / 5), -1.5, Math.cos((p * 2 * Math.PI) / 5)]));
  return [...circle.slice(1).map((c) => [c, apex] as [typeof c, typeof c]), ...circle.slice(0, 5).map((c, i) => [c, circle[i + 1]] as [typeof c, typeof c])];
}

function Shot() {
  const frame = useCurrentFrame();
  return (
    <>
      <Mesh3d transform={{ translation: [0, 0, 0], rotation: cubeAt(frame) }}>
        <Cuboid half_size={[1.5, 1.5, 1.5]} />
        <StandardMaterial base_color="#ffffff" />
      </Mesh3d>
      <Mesh3d transform={Transform.fromXyz(10, 4, 0).withScale([2.5, 2.5, 2.5])}>
        <Plane3d normal={[0, 0, 1]} half_size={[2, 2]} />
        <Material fragment={staticFile("shaders/show_depth_texture_material.wesl")} textures={{ depth_texture: DEPTH }} />
      </Mesh3d>
      <PointLight transform={Transform.fromXyz(5, 6, 7)} />
      <Camera3d order={-1} transform={Transform.fromXyz(-4, -5, 5).lookingAt([0, 0, 0])}>
        <BevyComponent type="bevy_camera::camera::RenderTarget" value={{ None: { size: [256, 256] } }} />
        <BevyComponent type="bevy_render::view::Msaa" value="Off" />
        <BevyComponent type="bevy_core_pipeline::prepass::DepthPrepass" value={{}} />
        <DepthCopy image={DEPTH} />
      </Camera3d>
      <Camera3d transform={Transform.fromXyz(5, 2, 30).lookingAt([5, 2, 0])}>
        <BevyComponent type="bevy_render::view::Msaa" value="Off" />
      </Camera3d>
      <Gizmo lines={cone()} color={LIME} />
      <Text position_type="Absolute" top={px(12)} left={px(12)}>Use WASD to move the secondary camera</Text>
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={61} fps={60} width={1280} height={720} />);
