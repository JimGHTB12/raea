// gap: G13
// A reflection probe from Bevy's own components on a bare <Entity> (the generic layer): LightProbe plus an
// EnvironmentMapLight whose diffuse and specular maps are probe-green.ktx2 (a 4 x 4 green cubemap, given by path and
// loaded through the AssetServer), its volume 4 m across around a rough metal cube. Inside the probe the cube takes its
// light from the green cubemap; light-probe-control.tsx, the same probe moved away, leaves it the grey of the default ambient light.
// probe: 32 32 g>150
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { BevyComponent, Canvas3D, Entity } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

export const AT: [number, number, number] = [0, 0, 0];

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(2, 1.5, 3).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <Entity name="probe" position={AT} scale={[4, 4, 4]}>
          <BevyComponent type="bevy_light::probe::LightProbe" value={{}} />
          <BevyComponent type="bevy_light::probe::EnvironmentMapLight" value={{ diffuse_map: "probe-green.ktx2", specular_map: "probe-green.ktx2", intensity: 2000, rotation: [0, 0, 0, 1], affects_lightmapped_mesh_diffuse: true }} />
        </Entity>
        <Mesh3d name="cube"><Cuboid half_size={[0.6, 0.6, 0.6]} /><StandardMaterial base_color="#ffffff" perceptual_roughness={0.4} metallic={1} /></Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
