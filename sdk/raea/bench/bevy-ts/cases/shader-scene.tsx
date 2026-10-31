// gap: SH4
// One shader over every lit standard surface in a scene (<SurfaceShader>): two white boxes and lit-card.gltf, none with
// a shader of its own; the `lit` hook keeps only the lit green, so all three show green. A fourth box is unlit
// (MeshBasicMaterial) and stays white.
// probe: 12 20 g>100 r<40 b<40
// probe: 52 20 g>100 r<40 b<40
// probe: 32 46 g>100 r<40 b<40
// probe: 32 12 r>180 g>180 b>180
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { ToneMapping, Canvas3D, Model, SurfaceShader } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const wesl = `
fn lit(in: VertexOutput, pbr: PbrInput, color: vec4<f32>) -> vec4<f32> {
  return vec4<f32>(0.0, color.g, 0.0, 1.0);
}`;

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D>
        <Camera3d transform={Transform.fromXyz(0, 0, 6).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.872664625997165 } }} />
        <ToneMapping mode="none" />
        <AmbientLight brightness={120} />
        <DirectionalLight transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} illuminance={8000} />
        <SurfaceShader wesl={wesl} />
        <Mesh3d transform={Transform.fromXyz(-1.6, 0.9, 0)}><Cuboid half_size={[0.5, 0.5, 0.5]} /><StandardMaterial base_color="#ffffff" /></Mesh3d>
        <Mesh3d transform={Transform.fromXyz(1.6, 0.9, 0)}><Cuboid half_size={[0.5, 0.5, 0.5]} /><StandardMaterial base_color="#ffffff" /></Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0, 1.6, 0)}><Cuboid half_size={[0.3, 0.3, 0.3]} /><StandardMaterial base_color="#ffffff" unlit /></Mesh3d>
        <Model src="lit-card.gltf" position={[0, -1.2, 0]} scale={0.6} />
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
