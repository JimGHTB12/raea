// raea-web parity (docs/r3f.feature R3F11): a three.js ShaderMaterial's GLSL run by raea's engine. naga translates the
// vertex shader (a sphere rippled along x) and the fragment shader (its normal lit by hand, a uniform colour, a stripe
// from its uv) and raea-sys-three-shader runs them with three's built-ins.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Mesh3d, Sphere } from "raea/mesh";
import { ThreeShaderMaterial } from "raea/pbr";
import { Transform } from "raea/transform";

const vertex = /* glsl */ `
uniform float uTime;
varying vec3 vNormal;
varying vec2 vUv;
void main() {
  vec3 p = position + normal * sin(position.y * 8.0 + uTime) * 0.08;
  vNormal = normalize(normalMatrix * normal);
  vUv = uv;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(p, 1.0);
}`;
const fragment = /* glsl */ `
uniform vec3 uColor;
varying vec3 vNormal;
varying vec2 vUv;
void main() {
  float light = 0.25 + 0.75 * max(dot(normalize(vNormal), normalize(vec3(0.4, 0.8, 0.6))), 0.0);
  float stripe = step(0.5, fract(vUv.x * 8.0));
  gl_FragColor = vec4(mix(uColor, vec3(1.0), stripe * 0.35) * light, 1.0);
}`;

function Shader() {
  return (
    <AbsoluteFill style={{ background: "#0b0d12" }}>
      <Canvas3D>
        <Camera3d transform={Transform.fromXyz(0, 0, 4).lookingAt([0, 0, 0])} />
        <Mesh3d name="ball">
          <Sphere radius={1} />
          <ThreeShaderMaterial glsl={{ vertex, fragment }} uniforms={{ uTime: 1.2, uColor: [0.9, 0.35, 0.15] }} />
        </Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shader} durationInFrames={1} fps={30} width={320} height={180} />);
