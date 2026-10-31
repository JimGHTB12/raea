// gap: SH2
// A shader over a Model's own lit glTF material (<Model> with a <MeshStandardMaterial shader> child): lit-card.gltf is a
// 2 m white card facing the camera (lit, material "card"); the `lit` hook keeps only the lit green, so the card shows
// green where it was white, and the black around it stays black.
// probe: 32 32 g>100 r<40 b<40
// probe: 4 4 g<20
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { ToneMapping, Canvas3D, Model } from "raea/3d";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { ExtendedMaterial, ShaderExtension, StandardMaterial } from "raea/pbr";

const wesl = `
fn lit(in: VertexOutput, pbr: PbrInput, color: vec4<f32>) -> vec4<f32> {
  return vec4<f32>(0.0, color.g, 0.0, 1.0);
}`;

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D>
        <Camera3d transform={Transform.fromXyz(0, 0, 4).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <ToneMapping mode="none" />
        <AmbientLight brightness={120} />
        <DirectionalLight transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} illuminance={8000} />
        <Model src="lit-card.gltf">
          <ExtendedMaterial base={<StandardMaterial material="card" />} extension={<ShaderExtension {...({ wesl })} />} />
        </Model>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
