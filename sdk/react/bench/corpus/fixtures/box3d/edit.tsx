// BOX3D: a synthetic 3D scene for the corpus harness (sdk/react/bench/corpus/run.sh): a box and a torus that spring in and
// turn under a camera and two lights, with a DOM title and lower third over the 3D, as an agent's product shot would
// put them. Square, 4 s, no footage.
import { AbsoluteFill, Composition, interpolate, registerRoot, spring, useCurrentFrame } from "raea";
import { Torus } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Quat, Transform } from "raea/transform";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Camera3d } from "raea/camera";

const FPS = 30;

function Shot() {
  const f = useCurrentFrame();
  const s = spring({ frame: f, fps: FPS, config: { damping: 12 } });
  const turn = interpolate(f, [0, 119], [0, Math.PI]);
  const o = interpolate(f, [20, 40], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  return (
    <AbsoluteFill style={{ background: "linear-gradient(180deg, #0e1726, #1d2b44)" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 0.6, 6).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.785398163397448 } }} />
        <AmbientLight brightness={240} />
        <DirectionalLight transform={Transform.fromXyz(3, 4, 5).lookingAt([0, 0, 0])} illuminance={9600} shadow_maps_enabled />
        <Mesh3d transform={Transform.fromXyz(-1.1, 0, 0).withRotation(Quat.fromEuler("XYZ", 0.4, turn, 0)).withScale([s, s, s])}>
          <Cuboid half_size={[0.7, 0.7, 0.7]} />
          <StandardMaterial base_color="#f59e0b" perceptual_roughness={0.35} metallic={0.1} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(1.3, 0, 0).withRotation(Quat.fromEuler("XYZ", turn, 0.3, 0)).withScale([s, s, s])}>
          <Torus major_radius={0.8} minor_radius={0.3} />
          <StandardMaterial base_color="#38bdf8" perceptual_roughness={0.2} metallic={0.6} />
        </Mesh3d>
      </>
      <div style={{ position: "absolute", top: 90, width: 1080, textAlign: "center", fontFamily: "Helvetica", fontWeight: 800, fontSize: 96, color: "#ffffff", opacity: o }}>NEW SHAPES</div>
      <div style={{ position: "absolute", left: 80, bottom: 90, padding: "18px 32px", borderRadius: 16, background: "rgba(255,255,255,0.12)", fontFamily: "Helvetica", fontSize: 44, color: "#e2e8f0", opacity: o }}>Spring collection · in stores now</div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="box3d" component={Shot} width={1080} height={1080} fps={FPS} durationInFrames={120} />);
