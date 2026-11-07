// Bevy v0.20.0 examples/stress_tests/many_foxes.rs in raea's TSX (docs/control.feature FC1a), run --sync (every fox
// starts its clip at 0; without it each starts at its entity's index / 10, which is Bevy's allocator's): 1000 animated
// foxes (models/animated/Fox.glb, its Run clip, Animation(2)) in rings 2 m apart, each ring turning at 2 m/s the other
// way to the last, on a 5000 m green plane under a shadow-casting sun, in a 1920 x 1080 window.
import { Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Entity, Model } from "raea/3d";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Mesh3d, Plane3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { DirectionalLight, Shadows } from "raea/light";

const COUNT = 1000;
const RING_SPACING = 2;
const FOX_SPACING = 2;
const SPEED = 2;
const FOX = staticFile("models/animated/Fox.glb");

type Fox = { x: number; z: number; rotation: [number, number, number, number] };
type Ring = { radius: number; sign: number; foxes: Fox[] };

// the example's setup: rings out from 2 m, as many foxes as fit 2 m apart, until 1000 are placed
const rings: Ring[] = [];
let radius = RING_SPACING;
for (let left = COUNT, i = 0; left > 0; i++) {
  const base = i % 2 === 0 ? Quat.fromRotationY(Math.PI) : Quat.IDENTITY;
  const circumference = Math.PI * 2 * radius;
  const n = Math.min(Math.floor(Math.fround(circumference / FOX_SPACING)), left);
  const step = circumference / (n * radius);
  const foxes = Array.from({ length: n }, (_, k) => {
    const a = k * step;
    return { x: radius * Math.cos(a), z: radius * Math.sin(a), rotation: Quat.mul(base, Quat.fromRotationY(-a)) };
  });
  rings.push({ radius, sign: i % 2 === 0 ? 1 : -1, foxes });
  left -= n;
  radius += RING_SPACING;
}
const zoom = 0.8;
const eye: [number, number, number] = [radius * 1.25 * zoom, radius * 0.5 * zoom, radius * 1.5 * zoom];

function Shot() {
  const f = useCurrentFrame();
  // update_fox_rings turns each ring by the update's delta_secs (none on the first), and each fox's AnimationPlayer
  // advances its clip by the same, from the first update (offsets of -1 to 8 frames scored 0.95-0.974 against 0.990)
  const t = f / 60;
  return (
    <>
      <Camera3d transform={Transform.fromTranslation(eye).lookingAt([0.2 * eye[0], 0, 0.2 * eye[2]])} />
      {rings.map((r, i) => (
        <Entity key={i} transform={{ rotation: Quat.fromRotationY(r.sign * (SPEED / r.radius) * t) }}>
          {r.foxes.map((x, k) => (
            <Entity key={k} transform={{ translation: [x.x, 0, x.z], rotation: x.rotation, scale: [0.01, 0.01, 0.01] }}>
              <Model src={FOX} animation={2} time={t} />
            </Entity>
          ))}
        </Entity>
      ))}
      <Mesh3d transform={Transform.IDENTITY}>
        <Plane3d half_size={[2500, 2500]} />
        <StandardMaterial base_color="rgb(76.5, 127.5, 76.5)" />
      </Mesh3d>
      {/* Bevy's default 2048-texel maps; its CascadeShadowConfigBuilder's 4 cascades, 0.2 overlap */}
      <Shadows mapSize={2048} firstCascade={0.9 * radius} distance={2.8 * radius} cascades={4} overlap={0.2} />
      <DirectionalLight shadow_maps_enabled transform={{ translation: [0, 0, 0], rotation: Quat.fromEuler("ZYX", 0, 1, -Math.PI / 4), scale: [1, 1, 1] }} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={61} fps={60} width={1920} height={1080} />);
