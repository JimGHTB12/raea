// Bevy v0.20.0 examples/3d/pbr.rs in raea's TSX (docs/control.feature FC0a): 11 x 5 spheres of #ffd891, roughness
// rising to the right and metal upwards, an unlit one below, a 1500 lx DirectionalLight from (50, 50, 50), Pisa's
// prefiltered EnvironmentMapLight at 900 on an orthographic camera (ScalingMode::WindowSize at 0.01: 7.2 m tall), and
// its two labels in Bevy's default font at 30 px (the "Loading" label is gone once the maps load), in Bevy's default
// 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Mesh3d, Sphere } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { DirectionalLight, EnvironmentMapLight } from "raea/light";

const FONT = `@font-face { font-family: "Fira Mono"; src: url("${staticFile("fonts/FiraMono-Medium.ttf")}"); }`;
const label = { position: "absolute", fontFamily: "Fira Mono", fontSize: 30, color: "#fff" } as const;

function Shot() {
  const balls = [];
  for (let y = -2; y <= 2; y++) {
    for (let x = -5; x <= 5; x++) {
      balls.push(
        <Mesh3d key={`${x},${y}`} transform={Transform.fromXyz(x, y + 0.5, 0)}>
          <Sphere radius={0.45} />
          <StandardMaterial base_color="#ffd891" metallic={(y + 2) / 4} perceptual_roughness={(x + 5) / 10} />
        </Mesh3d>,
      );
    }
  }
  return (
    <AbsoluteFill>
      <style>{FONT}</style>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        {balls}
        <Mesh3d transform={Transform.fromXyz(-5, -2.5, 0)}>
          <Sphere radius={0.45} />
          <StandardMaterial base_color="#ffd891" unlit />
        </Mesh3d>
        <DirectionalLight illuminance={1500} transform={Transform.fromXyz(50, 50, 50).lookingAt([0, 0, 0])} />
        <Camera3d projection={{ Orthographic: { scaling_mode: { FixedVertical: { viewport_height: 7.2 } } } }} transform={Transform.fromXyz(0, 0, 8).lookingAt([0, 0, 0])} />
        <EnvironmentMapLight diffuse_map={staticFile("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2")} specular_map={staticFile("environment_maps/pisa_specular_rgb9e5_zstd.ktx2")} intensity={900} />
      </Canvas3D>
      <div style={{ ...label, top: 20, left: 100 }}>Perceptual Roughness</div>
      <div style={{ ...label, top: 130, right: 0, transform: "rotate(90deg)" }}>Metallic</div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={61} fps={60} width={1280} height={720} />);
