// Bevy v0.20.0 examples/3d/rotate_environment_map.rs in raea's TSX (docs/control.feature FC0a): a 1.25 m sphere
// (Bevy's Sphere::new(1) mesh, tangents generated) in car paint, StandardMaterial of css GOLD, metallic 0.9, roughness
// 0.1, clearcoat 1 at roughness 0.3 with textures/ScratchedGold-Normal.png as its clearcoat normal map (loaded with
// is_srgb false), a white PointLight of 100000 lm at the origin, and the camera at (0, 0, 10) with a 27 degree
// fov, HDR and AcesFitted, carrying Pisa's Skybox at brightness 5000 and its prefiltered EnvironmentMapLight at 2000.
// rotate_skybox_and_environment_map turns both by Quat::from_rotation_y(0.2 * elapsed) every update. In Bevy's
// default 1280 x 720 window. Its clearcoat normal map needs raea built with raea-sys-material's multi-layer feature
// (Bevy's pbr_multi_layer_material_textures, off by default), so bench/bevy-parity/runs.tsv leaves it out until that
// is the default.
import { Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { ToneMapping } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d, Hdr } from "raea/camera";
import { Mesh3d, Sphere } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { EnvironmentMapLight, PointLight, Skybox } from "raea/light";

const f = Math.fround;
const PISA = "environment_maps/pisa_specular_rgb9e5_zstd.ktx2";

function Shot() {
  // elapsed: f updates of 16_666_667 ns (none on the first), as f32; glam's from_rotation_y in f32
  const t = f(useCurrentFrame() * 16_666_667e-9);
  const half = f(f(f(0.2) * t) * 0.5);
  const rotation: [number, number, number, number] = [0, f(Math.sin(half)), 0, f(Math.cos(half))];
  return (
    <>
      <Mesh3d transform={Transform.fromXyz(0, 0, 0).withScale([1.25, 1.25, 1.25])}>
        <Sphere radius={1} />
        <StandardMaterial
          base_color="rgb(255, 215, 0)"
          metallic={0.9}
          perceptual_roughness={0.1}
          clearcoat={1}
          clearcoat_perceptual_roughness={0.3}
          clearcoat_normal_texture={staticFile("textures/ScratchedGold-Normal.png")}
        />
      </Mesh3d>
      <PointLight color="#ffffff" intensity={100000} />
      <Camera3d transform={Transform.fromXyz(0, 0, 10)} projection={{ Perspective: { fov: (27 / 180) * Math.PI } }}>
        <Hdr />
      </Camera3d>
      <ToneMapping mode="aces" />
      <Skybox image={staticFile(PISA)} brightness={5000} rotation={rotation} />
      <EnvironmentMapLight diffuse_map={staticFile("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2")} specular_map={staticFile(PISA)} intensity={2000} rotation={rotation} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={121} fps={60} width={1280} height={720} />);
