import { AbsoluteFill, useCurrentFrame } from "raea";
import { Canvas3D, Entity, ShaderMesh, ToneMapping } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Transform } from "raea/transform";
// #26's scene: 200 small spheres orbiting, each with a per-frame uniform (scene.tsx sets a 360-frame timeline)
const W = `fn shade(in: VertexOutput) -> vec4<f32> { return vec4<f32>(u_level(), 0.6, 0.3, 1.0); }`;
const N = 200;
export function Scene() {
  const f = useCurrentFrame();
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D>
        <Camera3d transform={Transform.fromXyz(0, 0, 12).lookingAt([0, 0, 0], [0, 1, 0])} />
        <ToneMapping mode="none" />
        {Array.from({ length: N }, (_, i) => {
          const a = (i / N) * Math.PI * 2 + f * 0.01;
          const r = 2 + (i % 10) * 0.4;
          return (
            <Entity key={i} transform={Transform.fromXyz(Math.cos(a) * r, Math.sin(a) * r, 0)}>
              <ShaderMesh name={`s${i}`} shape={{ primitive: "Sphere", radius: 0.12 }} wgsl={W} uniforms={{ level: ((f + i) % 60) / 60 }} />
            </Entity>
          );
        })}
      </Canvas3D>
    </AbsoluteFill>
  );
}
