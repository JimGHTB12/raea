// bevy_hanabi through the generated SDK (sdk/raea/docs/bevy-plugins.md): two fountains of GPU particles, one opaque and
// one alpha-blended, on small pedestals. `Particles` is raea's hand-written reflected component (engine/bevy-plugins/src/
// hanabi.rs builds the effect asset from it); its TSX wrapper is generated like any other (raea/plugins/hanabi,
// from tools/bevy-plugins.sh gen). Render and play are compared by sdk/raea/bench/bevy/check.sh with RAEA_BIN=bevy-plugins/motion.sh.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { Particles } from "raea/plugins/hanabi";
import { StandardMaterial } from "raea/pbr";
import { ConicalFrustum, Mesh3d } from "raea/mesh";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const FPS = 30;
const FRAMES = 120;

function Fountains() {
  return (
    <AbsoluteFill style={{ background: "#0b1020" }}>
      <Canvas3D>
        <Camera3d transform={Transform.fromXyz(0, 1.5, 7).lookingAt([0, 1.2, 0])} projection={{ Perspective: { fov: 0.785398163397448 } }} />
        <AmbientLight brightness={200} />
        <DirectionalLight transform={Transform.fromXyz(2, 5, 4).lookingAt([0, 0, 0])} illuminance={9600} shadow_maps_enabled />
        <Mesh3d transform={Transform.fromXyz(-1.5, 0, 0)} name="left">
          <ConicalFrustum radius_top={0.25} radius_bottom={0.35} height={0.3} />
          <StandardMaterial base_color="#475569" />
          <Particles value={{ rate: 300, lifetime: 1.6, speed: 4, spread: 0.3, size: 0.06, color: [1, 0.55, 0.1, 1], gravity: 5, seed: 7, opaque: true }} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(1.5, 0, 0)} name="right">
          <ConicalFrustum radius_top={0.25} radius_bottom={0.35} height={0.3} />
          <StandardMaterial base_color="#475569" />
          <Particles value={{ rate: 300, lifetime: 1.6, speed: 4, spread: 0.3, size: 0.08, color: [0.3, 0.7, 1, 0.5], gravity: 5, seed: 7, opaque: false }} />
        </Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="fountains" component={Fountains} width={1280} height={720} fps={FPS} durationInFrames={FRAMES} />);
