// record: bevy-3d-rendering-lighting, bevy-3d-rendering-spotlight
// Bevy's "lighting" and "spotlight" examples in raea's SDK: a dim sun, a coloured bulb orbiting, and a spot sweeping
// across the floor, all moved from the frame.
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { Sphere } from "raea/3d";
import { LinearRgba, Srgba } from "raea/color";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight, DirectionalLight, PointLight, SpotLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Lighting() {
  const f = useCurrentFrame();
  const a = f * 0.05;
  const bulb: [number, number, number] = [Math.cos(a) * 2.5, 1.2, Math.sin(a) * 2.5];
  const sweep = Math.sin(f * 0.04) * 3;
  return (
    <AbsoluteFill style={{ background: "#07080b" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 4, 9).lookingAt([0, 0.5, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <AmbientLight brightness={32} />
        <DirectionalLight transform={Transform.fromXyz(-4, 6, 2).lookingAt([0, 0, 0])} illuminance={2000} shadow_maps_enabled />
        <PointLight color="#ff8a3d" transform={Transform.fromTranslation(bulb)} intensity={2250000} range={10} shadow_maps_enabled />
        <SpotLight color="#9ad1ff" transform={Transform.fromXyz(0, 6, 2).lookingAt([sweep, 0, 0])} intensity={5400000} outer_angle={0.35} inner_angle={0.21} />
        <Mesh3d transform={Transform.fromTranslation(bulb)} name="bulb">
          <Sphere radius={0.08} />
          <StandardMaterial base_color="#000000" emissive={LinearRgba.from(Srgba.hex("#ff8a3d")).mul(4)} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0, -0.05, 0)}>
          <Cuboid half_size={[6, 0.05, 4]} />
          <StandardMaterial base_color="#b9b4aa" perceptual_roughness={0.85} />
        </Mesh3d>
        {[-2, 0, 2].map((x) => (
          <Mesh3d key={x} transform={Transform.fromXyz(x, 0.5, 0)}>
            <Cuboid half_size={[0.5, 0.5, 0.5]} />
            <StandardMaterial base_color="#e9e4da" perceptual_roughness={0.5} />
          </Mesh3d>
        ))}
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="lighting" component={Lighting} width={1280} height={720} fps={30} durationInFrames={120} />);
