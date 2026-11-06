// avian3d through the generated SDK (sdk/raea/docs/bevy-plugins.md): balls and crates dropped on a tilted floor, a
// gravity resource, bounciness and friction, and a crate kicked sideways at frame 60 (a component row from a frame).
// Every physics component comes from raea/plugins/avian (sdk/plugins/src/avian.gen.tsx), written by raea-bevy-sdk
// from avian's TypeRegistry; no raea-render system is per component. Render and play
// must agree: sdk/raea/bench/bevy/check.sh with RAEA_BIN=bevy-plugins/motion.sh.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Sphere } from "raea/3d";
import { AngularVelocity, ColliderConstructor, Friction, GravityResource, LinearVelocity, Restitution, RigidBody } from "raea/plugins/avian";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const FPS = 30;
const FRAMES = 150;

const balls = [
  { x: -1.6, y: 3.0, r: 0.35, c: "#e11d48", bounce: 0.8 },
  { x: -0.6, y: 4.2, r: 0.3, c: "#f59e0b", bounce: 0.5 },
  { x: 0.5, y: 3.6, r: 0.4, c: "#10b981", bounce: 0.2 },
  { x: 1.5, y: 5.0, r: 0.25, c: "#3b82f6", bounce: 0.9 },
];
const crates = [
  { x: -1.0, y: 6.0, s: 0.6, c: "#8b5cf6" },
  { x: 1.0, y: 7.0, s: 0.5, c: "#d946ef", kick: true },
];

function Drop() {
  return (
    <AbsoluteFill style={{ background: "#0f1720" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 2.5, 9).lookingAt([0, 1, 0])} projection={{ Perspective: { fov: 0.785398163397448 } }} />
        <AmbientLight brightness={200} />
        <DirectionalLight transform={Transform.fromXyz(3, 6, 4).lookingAt([0, 0, 0])} illuminance={10400} shadow_maps_enabled />
        <GravityResource value={[0, -9.81, 0]} />
        <Mesh3d transform={Transform.fromXyz(0, 0, 0).withRotation(Quat.fromEuler("XYZ", 0, 0, -0.12))} name="floor">
          <Cuboid half_size={[4, 0.15, 2]} />
          <StandardMaterial base_color="#334155" perceptual_roughness={0.8} />
          <RigidBody value="Static" />
          <ColliderConstructor value={{ Cuboid: { x_length: 8, y_length: 0.3, z_length: 4 } }} />
          <Friction value={{ dynamic_coefficient: 0.6, static_coefficient: 0.6, combine_rule: "Average" }} />
        </Mesh3d>
        {balls.map((b, i) => (
          <Mesh3d key={`ball${i}`} transform={Transform.fromXyz(b.x, b.y, 0)} name={`ball${i}`} driven={true}>
            <Sphere radius={b.r} />
            <StandardMaterial base_color={b.c} perceptual_roughness={0.4} />
            <RigidBody value="Dynamic" />
            <ColliderConstructor value={{ Sphere: { radius: b.r } }} />
            <Restitution value={{ coefficient: b.bounce, combine_rule: "Max" }} />
          </Mesh3d>
        ))}
        {crates.map((c, i) => (
          <Mesh3d key={`crate${i}`} transform={Transform.fromXyz(c.x, c.y, 0).withRotation(Quat.fromEuler("XYZ", 0.3, 0.2, 0.4))} name={`crate${i}`} driven={true}>
            <Cuboid half_size={[c.s / 2, c.s / 2, c.s / 2]} />
            <StandardMaterial base_color={c.c} perceptual_roughness={0.5} />
            <RigidBody value="Dynamic" />
            <ColliderConstructor value={{ Cuboid: { x_length: c.s, y_length: c.s, z_length: c.s } }} />
            <AngularVelocity value={[0, 2, 1]} />
            {c.kick ? <LinearVelocity value={[-4, 3, 0]} from={60} /> : null}
          </Mesh3d>
        ))}
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="drop" component={Drop} width={1280} height={720} fps={FPS} durationInFrames={FRAMES} />);
