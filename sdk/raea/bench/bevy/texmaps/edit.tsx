// Texture maps fixture: Poly Haven CC0 sets (media/, 1k) on a tiled marble floor (a Plane, separate rough and ao
// maps: the grey rough map packed into Bevy's metallic_roughness_texture as it loads; the engine's grey #cccccc under each set), a brick
// wall (a Box, its packed _arm map as both metallic_roughness_texture and occlusion_texture) and a marble plinth, under a
// raking light that sweeps across them so the normal maps catch it. The camera dollies; the plinth's roughness and the
// light move per frame.
import { AbsoluteFill, Composition, interpolate, registerRoot, staticFile, useCurrentFrame } from "raea";
import { StandardMaterial } from "raea/pbr";
import { Affine2 } from "raea/math";
import { Cuboid, Mesh3d, Plane3d } from "raea/mesh";
import { AmbientLight, DirectionalLight, SpotLight } from "raea/light";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const FPS = 30;
const FRAMES = 60;

const marble = {
  base_color: "#cccccc",
  base_color_texture: staticFile("marble_01_diff_1k.jpg"),
  normal_map_texture: staticFile("marble_01_nor_gl_1k.jpg"),
  metallic_roughness_texture: { roughness: staticFile("marble_01_rough_1k.jpg") },
  occlusion_texture: staticFile("marble_01_ao_1k.jpg"),
  perceptual_roughness: 1,
};
const brick = {
  base_color: "#cccccc",
  base_color_texture: staticFile("castle_brick_07_diff_1k.jpg"),
  normal_map_texture: staticFile("castle_brick_07_nor_gl_1k.jpg"),
  metallic_roughness_texture: staticFile("castle_brick_07_arm_1k.jpg"),
  occlusion_texture: staticFile("castle_brick_07_arm_1k.jpg"),
  perceptual_roughness: 1,
};

function Shot() {
  const f = useCurrentFrame();
  const sweep = interpolate(f, [0, FRAMES - 1], [-4, 4]);
  const cx = interpolate(f, [0, FRAMES - 1], [-1.2, 1.2]);
  return (
    <AbsoluteFill style={{ background: "#000000" }}>
      <>
        <Camera3d transform={Transform.fromXyz(cx, 1.7, 5.5).lookingAt([0, 0.9, -1])} projection={{ Perspective: { fov: 0.872664625997165 } }} />
        <AmbientLight brightness={60} />
        <DirectionalLight color="#dfe6ff" transform={Transform.fromXyz(-3, 5, 4).lookingAt([0, 0, 0])} illuminance={4000} shadow_maps_enabled />
        <SpotLight color="#ffe2b8" transform={Transform.fromXyz(sweep, 2.6, 0.6).lookingAt([sweep * 0.3, 1.2, -3])} intensity={2700000} outer_angle={0.7} inner_angle={0.35} shadow_maps_enabled />
        <Mesh3d transform={Transform.IDENTITY.withRotation(Quat.fromEuler("XYZ", -Math.PI / 2, 0, 0))} name="floor">
          <Plane3d normal={[0, 0, 1]} half_size={[6, 6]} />
          <StandardMaterial {...marble} uv_transform={Affine2.fromScale([6, 6])} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0, 2, -3)} name="wall">
          <Cuboid half_size={[6, 2, 0.2]} />
          <StandardMaterial {...brick} uv_transform={Affine2.fromScale([6, 2])} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0, 0.6, -0.5).withRotation(Quat.fromEuler("XYZ", 0, f * 0.01, 0))} name="plinth">
          <Cuboid half_size={[0.45, 0.6, 0.45]} />
          <StandardMaterial {...marble} base_color="#f0ece4" perceptual_roughness={interpolate(f, [0, FRAMES - 1], [1, 0.4])} clearcoat={0.3} />
        </Mesh3d>
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="texmaps" component={Shot} width={960} height={540} fps={FPS} durationInFrames={FRAMES} />);
