// A boat's wake, built from raea/3d's public parts (docs/examples.feature EX5): copy it and make it yours.
import { useCurrentFrame, useVideoConfig } from "raea";
import { toHex, type Vec3 } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Quat, Transform } from "raea/transform";
import { Cuboid, Mesh3d } from "raea/mesh";

/** small seeded random numbers, the same for the same seed */
function rng(seed: number) {
  let s = (Math.floor(seed) * 2654435761 + 1013904223) >>> 0 || 1;
  return () => {
    s ^= s << 13;
    s >>>= 0;
    s ^= s >>> 17;
    s ^= s << 5;
    s >>>= 0;
    return s / 4294967296;
  };
}

export interface WakeProps {
  name: string;
  /** where the boat is at a time (seconds) and which way it's heading (a unit [x, z]), on the water */
  path: (t: number) => { at: Vec3; heading: [number, number] };
  /** the water's height (the path's y by default) */
  y?: number;
  /** seconds of wake behind the boat (4) in this many segments an arm (14) */
  length?: number;
  segments?: number;
  /** how fast the arms open sideways (0.5 m/s) and how wide a segment's foam is when new (0.12 m) and old (0.6 m) */
  spread?: number;
  width?: number;
  oldWidth?: number;
  color?: string;
  time?: number;
}

/** A boat's wake: two arms of broken white foam opening behind it in a V, and the churn at its stern, laid flat on the
 * water. Segments are matched by name across frames; the look's rag breaks their edges. */
export function Wake(props: WakeProps) {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const t = props.time ?? frame / fps;
  const n = props.segments ?? 14;
  const len = props.length ?? 4;
  const spread = props.spread ?? 0.5;
  const w0 = props.width ?? 0.12;
  const w1 = props.oldWidth ?? 0.6;
  const color = toHex("Wake", "color", props.color ?? "#ffffff");
  const r = rng(n * 31 + 5);
  const jit = Array.from({ length: n * 2 }, () => r());
  const parts = [];
  for (let arm = -1; arm <= 1; arm += 2) {
    for (let i = 0; i < n; i++) {
      const a0 = (i / n) * len;
      const a1 = ((i + 0.8) / n) * len;
      const p0 = props.path(t - a0);
      const p1 = props.path(t - a1);
      const sx = (p: { heading: [number, number] }, age: number) => [-p.heading[1] * arm * (0.25 + spread * age), p.heading[0] * arm * (0.25 + spread * age)];
      const o0 = sx(p0, a0);
      const o1 = sx(p1, a1);
      const x0 = p0.at[0] + o0[0];
      const z0 = p0.at[2] + o0[1];
      const x1 = p1.at[0] + o1[0];
      const z1 = p1.at[2] + o1[1];
      const y = props.y ?? p0.at[1];
      const age = (a0 + a1) / 2 / len;
      const w = w0 + (w1 - w0) * age * (0.6 + 0.8 * jit[i + (arm > 0 ? n : 0)]);
      // the oldest foam breaks up: fewer, shorter pieces
      const keep = age < 0.55 || jit[i + (arm > 0 ? n : 0)] > age * 0.9;
      const l = Math.hypot(x1 - x0, z1 - z0) * (keep ? 1 : 0.001);
      parts.push(
        <Mesh3d key={`${arm}-${i}`} transform={Transform.fromXyz((x0 + x1) / 2, y + 0.01, (z0 + z1) / 2).withRotation(Quat.fromEuler("XYZ", 0, -Math.atan2(z1 - z0, x1 - x0), 0)).withScale([Math.max(l, 0.001), 1, keep ? w : 0.001])} name={`${props.name}-${arm}-${i}`}>
          <Cuboid half_size={[0.5, 0.01, 0.5]} />
          <StandardMaterial base_color={color} unlit />
        </Mesh3d>,
      );
    }
  }
  // the churn at the stern
  const s = props.path(t - 0.05);
  parts.push(
    <Mesh3d key="stern" transform={Transform.fromXyz(s.at[0] - s.heading[0] * 0.2, (props.y ?? s.at[1]) + 0.012, s.at[2] - s.heading[1] * 0.2).withRotation(Quat.fromEuler("XYZ", 0, -Math.atan2(s.heading[1], s.heading[0]), 0)).withScale([1.1, 1, 0.5])} name={`${props.name}-stern`}>
      <Cuboid half_size={[0.5, 0.01, 0.5]} />
      <StandardMaterial base_color={color} unlit />
    </Mesh3d>,
  );
  return <>{parts}</>;
}
