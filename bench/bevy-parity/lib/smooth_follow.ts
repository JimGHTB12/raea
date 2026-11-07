// examples/movement/smooth_follow.rs's two systems in f32, on the harness's clock (an update a frame, none of it on the
// first): move_target walks the blue sphere at 5 m/s to a point Cuboid(4 m).sample_interior draws from the example's
// ChaCha8Rng (seed 68941654987813521), drawing the next once it's there (Dir3::new fails on a zero vector), and
// move_follower smooth_nudges the red one towards it at decay rate 2 (bevy_math's VectorSpace::lerp, self * (1 - t) +
// rhs * t). Checked against bevy_math's own run (lib/smooth_follow.test.ts).
import { ChaCha8Rng } from "./rand";

export type V3 = [number, number, number];
const f = Math.fround;
const sub = (a: V3, b: V3): V3 => [f(a[0] - b[0]), f(a[1] - b[1]), f(a[2] - b[2])];
/** glam's Vec3::length: sqrt of the dot, summed x, y, z in order */
const length = (v: V3) => f(Math.sqrt(f(f(f(v[0] * v[0]) + f(v[1] * v[1])) + f(v[2] * v[2]))));

/** the spheres after the updates up to frame `frame`: [target, follower, the point it's heading for] */
export function smoothFollow(frame: number): [V3, V3, V3] {
  const rng = ChaCha8Rng.seed_from_u64(68941654987813521n);
  const dtFull = f(16_666_667e-9);
  let target: V3 = [0, 0, 0];
  let pos: V3 = [0, 0, 0];
  let follow: V3 = [0, -2, 0];
  for (let k = 0; k <= frame; k++) {
    const dt = k === 0 ? 0 : dtFull;
    const d = sub(pos, target);
    const len = length(d);
    if (Number.isFinite(len) && len > 0) {
      const dir: V3 = [f(d[0] / len), f(d[1] / len), f(d[2] / len)];
      const m = Math.min(len, f(dt * 5));
      target = [f(target[0] + f(dir[0] * m)), f(target[1] + f(dir[1] * m)), f(target[2] + f(dir[2] * m))];
    } else {
      pos = [rng.random_range(-2, 2), rng.random_range(-2, 2), rng.random_range(-2, 2)];
    }
    const t = f(1 - f(Math.exp(f(-2 * dt))));
    const keep = f(1 - t);
    follow = [0, 1, 2].map((i) => f(f(follow[i]! * keep) + f(target[i]! * t))) as V3;
  }
  return [target, follow, pos];
}
