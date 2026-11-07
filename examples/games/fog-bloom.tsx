// record: bevy-3d-rendering-fog, bevy-3d-rendering-bloom-3d
// Bevy's "fog" and "bloom 3d" examples in raea's SDK: a corridor of pillars fading into fog that matches the
// background, with glowing orbs blooming as they pulse.
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { Bloom, Fog, Sphere } from "raea/3d";
import { LinearRgba, Srgba } from "raea/color";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const BG = "#1d2433";

function FogBloom() {
  const f = useCurrentFrame();
  const z = -f * 0.06;
  return (
    <AbsoluteFill style={{ background: BG }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 1.6, 4 + z).lookingAt([0, 1.2, -6 + z])} projection={{ Perspective: { fov: 0.959931088596881 } }} />
        <AmbientLight brightness={120} />
        <DirectionalLight transform={Transform.fromXyz(2, 6, 3).lookingAt([0, 0, 0])} illuminance={6400} shadow_maps_enabled />
        <Fog color={BG} near={4} far={22} />
        <Bloom intensity={1.4} threshold={0.8} />
        <Mesh3d transform={Transform.fromXyz(0, -0.05, -15)}>
          <Cuboid half_size={[4, 0.05, 25]} />
          <StandardMaterial base_color="#3b4458" perceptual_roughness={0.9} />
        </Mesh3d>
        {Array.from({ length: 12 }, (_, i) => (
          <Mesh3d key={`p${i}`} transform={Transform.fromXyz(i % 2 ? 2.2 : -2.2, 1.5, -i * 3)}>
            <Cuboid half_size={[0.25, 1.5, 0.25]} />
            <StandardMaterial base_color="#c9c2b2" perceptual_roughness={0.7} />
          </Mesh3d>
        ))}
        {Array.from({ length: 6 }, (_, i) => (
          <Mesh3d key={`o${i}`} transform={Transform.fromXyz(0, 1.2 + 0.2 * Math.sin(f * 0.1 + i), -2 - i * 5)}>
            <Sphere radius={0.25} />
            <StandardMaterial base_color="#000000" emissive={LinearRgba.from(Srgba.hex("#58e1ff")).mul(4 + 3 * Math.sin(f * 0.15 + i))} />
          </Mesh3d>
        ))}
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="fogbloom" component={FogBloom} width={1280} height={720} fps={30} durationInFrames={120} />);
