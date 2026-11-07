// Bevy v0.20.0 examples/3d/atmospheric_fog.rs in raea's TSX (docs/control.feature FC0a): the camera at (-1, 0.1, 1)
// looking at the origin with Bevy's DistanceFog, a <Fog> inside the <Camera3d> (colour srgba(0.35, 0.48, 0.66, 1),
// directional light colour srgba(1, 0.95, 0.85, 0.5) at exponent 30, FogFalloff::from_visibility_colors(15,
// srgb(0.35, 0.5, 0.66), srgb(0.8, 0.844, 1))), a shadowed DirectionalLight of srgb(0.98, 0.95, 0.82) from the origin
// towards (-0.15, -0.05, 0.25) with cascades to 3 m (first 0.3 m), models/terrain/Mountains.gltf, a 2 x 1 x 1 unlit
// #888888 box scaled 20 around it all drawn both sides and casting no shadow, and the instructions in Bevy's default
// font at bottom 12, left 12, in Bevy's default 1280 x 720 window. The fog's sun term (bevy_pbr's apply_fog, scaled by
// the directional shadow) changes no pixel of these frames: a black or alpha-0 sun colour renders the same, so this
// port checks the falloff and colour, not the sun's glow.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D, Entity, Fog, Model, Shadows } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d } from "raea/mesh";
import { FogFalloff, StandardMaterial } from "raea/pbr";
import { DirectionalLight } from "raea/light";
import { Srgba } from "raea/color";

const FONT = `@font-face { font-family: "Fira Mono"; src: url("${staticFile("fonts/FiraMono-Medium.ttf")}"); }`;

function Shot() {
  return (
    <AbsoluteFill>
      <style>{FONT}</style>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        <Camera3d transform={Transform.fromXyz(-1, 0.1, 1).lookingAt([0, 0, 0])}>
          <Fog
            color={new Srgba(0.35, 0.48, 0.66, 1)}
            directional_light_color={new Srgba(1, 0.95, 0.85, 0.5)}
            directional_light_exponent={30}
            falloff={FogFalloff.from_visibility_colors(15, new Srgba(0.35, 0.5, 0.66, 1), new Srgba(0.8, 0.844, 1, 1))}
          />
        </Camera3d>
        <Shadows mapSize={2048} distance={3} firstCascade={0.3} cascades={4} overlap={0.2} />
        <DirectionalLight color="rgb(249.9, 242.25, 209.1)" shadow_maps_enabled transform={Transform.fromXyz(0, 0, 0).lookingAt([-0.15, -0.05, 0.25])} />
        <Entity transform={Transform.IDENTITY}>
          <Model src={staticFile("models/terrain/Mountains.gltf")} />
        </Entity>
        <Mesh3d transform={Transform.fromScale([20, 20, 20])} castShadow={false}>
          <Cuboid half_size={[1, 0.5, 0.5]} />
          <StandardMaterial base_color="#888888" unlit cull_mode={null} />
        </Mesh3d>
      </Canvas3D>
      <div style={{ position: "absolute", bottom: 12, left: 12, fontFamily: "Fira Mono", fontSize: 20, lineHeight: 1.2, color: "#fff", whiteSpace: "pre" }}>{"Press Spacebar to Toggle Atmospheric Fog.\nPress S to Toggle Directional Light Fog Influence."}</div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
