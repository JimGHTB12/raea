// record: bevy-camera-camera-orbit, bevy-math-smooth-follow
// Bevy's "camera orbit" and "smooth follow" examples in raea's SDK. The camera is a function of the frame: it orbits
// the scene, and a marker follows a moving target with exponential smoothing, worked out in closed form per frame
// (a follower that lags by tau seconds behind a target moving on a circle) so any frame renders alone.
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame, useVideoConfig } from "raea";
import { Sphere } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Orbit() {
  const f = useCurrentFrame();
  const { fps } = useVideoConfig();
  const t = f / fps;
  const yaw = t * 0.6;
  const cam: [number, number, number] = [Math.sin(yaw) * 8, 3.5, Math.cos(yaw) * 8];
  const w = 1.5; // the target's angular speed (rad/s)
  const target: [number, number, number] = [Math.cos(t * w) * 2.5, 0.4, Math.sin(t * w) * 2.5];
  // steady state of x' = (target - x) / tau for circular motion: the same circle, smaller and phase-lagged
  const tau = 0.4;
  const gain = 1 / Math.sqrt(1 + (w * tau) ** 2);
  const lag = Math.atan(w * tau);
  const follow: [number, number, number] = [Math.cos(t * w - lag) * 2.5 * gain, 0.4, Math.sin(t * w - lag) * 2.5 * gain];
  return (
    <AbsoluteFill style={{ background: "#0f1720" }}>
      <>
        <Camera3d transform={Transform.fromTranslation(cam).lookingAt([0, 0.4, 0])} projection={{ Perspective: { fov: 0.785398163397448 } }} />
        <AmbientLight brightness={160} />
        <DirectionalLight transform={Transform.fromXyz(4, 8, 3).lookingAt([0, 0, 0])} illuminance={9600} shadow_maps_enabled />
        <Mesh3d transform={Transform.fromXyz(0, -0.05, 0)}>
          <Cuboid half_size={[5, 0.05, 5]} />
          <StandardMaterial base_color="#c4bfb2" perceptual_roughness={0.9} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromTranslation(target)} name="target">
          <Sphere radius={0.3} />
          <StandardMaterial base_color="#ef476f" />
        </Mesh3d>
        <Mesh3d transform={Transform.fromTranslation(follow)} name="follower">
          <Cuboid half_size={[0.225, 0.225, 0.225]} />
          <StandardMaterial base_color="#06d6a0" />
        </Mesh3d>
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="orbit" component={Orbit} width={1280} height={720} fps={30} durationInFrames={150} />);
