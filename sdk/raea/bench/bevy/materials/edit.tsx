// Materials fixture: a glowing emissive ring with bloom, a see-through glass sphere fading in, a clearcoat car-paint
// sphere, an unlit sign, a sphere whose colour animates, Text3D letters; distance fog and ACES tone mapping.
import { AbsoluteFill, Composition, interpolate, registerRoot, useCurrentFrame } from "raea";
import { Bloom, Fog, Sphere, Text3D, ToneMapping, Torus } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { LinearRgba, Srgba } from "raea/color";

const FPS = 30;
const FRAMES = 60;

function mix(a: number[], b: number[], t: number) {
  return "#" + a.map((x, i) => Math.round(x + (b[i] - x) * t).toString(16).padStart(2, "0")).join("");
}

function Shot() {
  const f = useCurrentFrame();
  const t = f / (FRAMES - 1);
  const glow = interpolate(f, [0, FRAMES - 1], [1, 6]);
  const glass = interpolate(f, [0, FRAMES - 1], [0.15, 0.6]);
  return (
    <AbsoluteFill style={{ background: "#10131a" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 1.5, 7).lookingAt([0, 0.3, 0])} projection={{ Perspective: { fov: 0.785398163397448 } }} />
        <AmbientLight brightness={120} />
        <DirectionalLight transform={Transform.fromXyz(3, 5, 4).lookingAt([0, 0, 0])} illuminance={11200} shadow_maps_enabled />
        <Bloom intensity={1.5} threshold={0.8} />
        <Fog color="#10131a" near={6} far={16} />
        <ToneMapping mode="aces" />
        <Mesh3d transform={Transform.fromXyz(-2.6, 0.4, 0).withRotation(Quat.fromEuler("XYZ", Math.PI / 2, 0, f * 0.03))}>
          <Torus major_radius={0.6} minor_radius={0.12} />
          <StandardMaterial base_color="#1a1a1a" emissive={LinearRgba.from(Srgba.hex("#ff3c78")).mul(glow)} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(-0.9, 0.4, 0)}>
          <Sphere radius={0.6} />
          <StandardMaterial base_color={Srgba.hex("#bfe6ff").with_alpha(glass)} perceptual_roughness={0.05} alpha_mode="Blend" />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0.8, 0.4, 0)}>
          <Sphere radius={0.6} />
          <StandardMaterial base_color="#8a0f1e" perceptual_roughness={0.5} metallic={0.6} clearcoat={1} clearcoat_perceptual_roughness={0.05} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(2.5, 0.4, 0)}>
          <Sphere radius={0.6} />
          <StandardMaterial base_color={mix([40, 200, 120], [250, 200, 40], t)} perceptual_roughness={0.4} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0, 1.9, -1)}>
          <Cuboid half_size={[1.6, 0.35, 0.025]} />
          <StandardMaterial base_color="#f5f0e1" unlit />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(-1.2, -0.9, 0.5)}>
          <Text3D size={0.6} depth={0.2}>SALE</Text3D>
          <StandardMaterial base_color="#ffd166" perceptual_roughness={0.25} metallic={0.8} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0, -1.3, -6)}>
          <Cuboid half_size={[15, 0.05, 15]} />
          <StandardMaterial base_color="#2a2f3a" perceptual_roughness={0.9} />
        </Mesh3d>
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="materials" component={Shot} width={960} height={540} fps={FPS} durationInFrames={FRAMES} />);
