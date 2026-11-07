// record: bevy-3d-rendering-pbr
// Bevy's "PBR" example in raea's SDK: a grid of spheres, metalness rising left to right and roughness rising bottom
// to top. Without an environment map fully metal surfaces read dark, so metalness tops out at 0.8 (bevy/limits).
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Sphere } from "raea/3d";
import { AmbientLight, DirectionalLight, PointLight } from "raea/light";
import { Camera3d } from "raea/camera";
import { StandardMaterial } from "raea/pbr";
import { Transform } from "raea/transform";
import { Mesh3d } from "raea/mesh";

const N = 5;

function Pbr() {
  const cells = [];
  for (let r = 0; r < N; r++) {
    for (let c = 0; c < N; c++) {
      cells.push(
        <Mesh3d key={`${r}-${c}`} transform={Transform.fromXyz((c - 2) * 1.2, (r - 2) * 1.2, 0)}>
          <Sphere radius={0.48} />
          <StandardMaterial base_color="#c8553d" perceptual_roughness={0.08 + (r / (N - 1)) * 0.9} metallic={(c / (N - 1)) * 0.8} />
        </Mesh3d>,
      );
    }
  }
  return (
    <AbsoluteFill style={{ background: "#202329" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 0, 11).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.663225115757845 } }} />
        <AmbientLight brightness={160} />
        <DirectionalLight transform={Transform.fromXyz(3, 4, 6).lookingAt([0, 0, 0])} illuminance={9600} shadow_maps_enabled />
        <PointLight color="#bfd7ff" transform={Transform.fromXyz(-4, 3, 4)} intensity={1350000} />
        {cells}
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="pbr" component={Pbr} width={1280} height={720} fps={30} durationInFrames={30} />);
