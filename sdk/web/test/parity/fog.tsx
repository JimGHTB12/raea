// raea-web parity (docs/web-export.md, "raea's own kinds"): volumetric fog, a fog volume lit by a volumetric sun, a
// box's shadow cut through its shafts. Bevy raymarches it through its shadow maps; three.js has nothing for it.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D, FogVolume, VolumetricFog } from "raea/3d";
import { Camera3d } from "raea/camera";
import { DirectionalLight } from "raea/light";
import { Cuboid, Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { Transform } from "raea/transform";

function Fog() {
  return (
    <AbsoluteFill style={{ background: "#05070c" }}>
      <Canvas3D>
        <Camera3d transform={Transform.fromXyz(0, 1.5, 7).lookingAt([0, 1, 0])} />
        <DirectionalLight volumetric illuminance={8000} shadow_maps_enabled transform={Transform.fromXyz(2, 6, -3).lookingAt([0, 0, 0])} />
        <VolumetricFog />
        <FogVolume name="air" position={[0, 2, 0]} scale={[10, 4, 10]} density={0.06} scattering={0.5} absorption={0.1} />
        <Mesh3d name="floor" transform={Transform.fromXyz(0, -0.05, 0)}>
          <Cuboid half_size={[6, 0.05, 6]} />
          <StandardMaterial base_color="#808890" perceptual_roughness={0.9} />
        </Mesh3d>
        <Mesh3d name="box" transform={Transform.fromXyz(0, 2.5, -1)}>
          <Cuboid half_size={[0.8, 0.8, 0.8]} />
          <StandardMaterial base_color="#c04030" />
        </Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Fog} durationInFrames={1} fps={30} width={320} height={180} />);
