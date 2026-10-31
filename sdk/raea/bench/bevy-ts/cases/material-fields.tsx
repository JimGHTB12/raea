// gap: SH7
// StandardMaterial's further Bevy fields (docs/shaders.feature SH7). Every surface is black and glows its
// colour (emissive), so lighting doesn't move the probes. A white <Fog> covers everything past 2 m:
//   top: two red boxes, the left fogged white, the right with fog={false} still red;
//   bottom: checker2.png (a 2 x 2 checker, white top left) as two glowing squares with hard pixels, the right one's
//   uv_transform offset by half in u, so its top left is black where the left's is white.
// The rest (specular_tint, anisotropy, attenuation, parallax, depth_bias, opaque_render_method, diffuse_transmission) reach the
// material as its `fields` column: the rows check it carries them, and raea-sys-material's fields.rs test that each
// lands on the StandardMaterial.
// probe: 16 18 r>200 g>200 b>200
// probe: 48 18 r>200 g<40 b<40
// probe: 11 43 r>150 g>150
// probe: 43 43 r<40 g<40
// rows: specularTint.*anisotropy
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D, Fog, ToneMapping } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Affine2 } from "raea/math";
import { Cuboid, Mesh3d, Plane3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const glow = { base_color: "#000000", perceptual_roughness: 1 } as const;

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D>
        <Camera3d transform={Transform.fromXyz(0, 0, 6).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.872664625997165 } }} />
        <ToneMapping mode="none" />
        <Fog color="#ffffff" near={1} far={2} />
        <Mesh3d transform={Transform.fromXyz(-1.4, 1.2, 0)}><Cuboid half_size={[0.5, 0.5, 0.5]} /><StandardMaterial {...glow} emissive="#ff0000" /></Mesh3d>
        <Mesh3d transform={Transform.fromXyz(1.4, 1.2, 0)}>
          <Cuboid half_size={[0.5, 0.5, 0.5]} />
          <StandardMaterial {...glow} emissive="#ff0000" fog_enabled={false} specular_tint="#00ff00" anisotropy_strength={0.5} anisotropy_rotation={1} attenuation_distance={2} attenuation_color="#ffeecc" parallax_depth_scale={0.05} parallax_mapping_method={{ Relief: { max_steps: 8 } }} depth_bias={1} opaque_render_method="Forward" diffuse_transmission={0.2} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(-1.4, -1.2, 0)}>
          <Plane3d normal={[0, 0, 1]} half_size={[0.8, 0.6]} />
          <StandardMaterial {...glow} emissive="#ffffff" emissive_texture={{ path: "checker2.png", sampler: { filter: "nearest" } }} fog_enabled={false} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(1.4, -1.2, 0)}>
          <Plane3d normal={[0, 0, 1]} half_size={[0.8, 0.6]} />
          <StandardMaterial {...glow} emissive="#ffffff" emissive_texture={{ path: "checker2.png", sampler: { filter: "nearest", wrap: "repeat" } }} fog_enabled={false} uv_transform={Affine2.fromTranslation([0.5, 0])} />
        </Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
