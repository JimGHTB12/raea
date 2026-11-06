// Arms kept out of the trunk in play (docs/locomotion.md, U1). The retarget clears each clip's arms of the body's own
// trunk (retarget-ik.ts clearOfTrunk), but play moves them back in: blends between clips, Froude-quickened cadence, a
// body built wider than the capture's actor (mai's forearms crossed into her trunk on 3% of a run's frames). Each
// frame's arm is cleared again, and the turn that took, at the shoulder about the world, spread over the frames round
// it (the strongest of the neighbours', faded over `spread` frames, then averaged twice), so an arm swings wide of the
// body smoothly instead of hopping off it in a frame.
import type { Vec3 } from "raea/3d";
import { B, SIDES, joint, pose, qinv, qmul, qnorm, setWorld, type Pose, type Skeleton } from "raea/character";
import { qexp, qlog } from "../effects";
import { clearOfTrunk } from "../retarget-ik";

export function clearArms(skel: Skeleton, poses: Pose[], spread = 3): Pose[] {
  const n = poses.length;
  if (!n) return poses;
  const W = poses.map((p) => pose(skel, p));
  const sides = SIDES.map((s) => joint(skel, B.upperarm(s)));
  // each frame's correction per side: the world turn clearOfTrunk put on the upper arm
  const corr: Vec3[][] = sides.map(() => []);
  for (let f = 0; f < n; f++) {
    const nj = poses[f].turns.length, G = clearOfTrunk(skel, W[f].rot.slice(0, nj), nj);
    sides.forEach((u, k) => corr[k].push(qlog(qnorm(qmul(G[u], qinv(W[f].rot[u]))))));
  }
  const mag = (v: Vec3) => Math.hypot(v[0], v[1], v[2]);
  const smoothed = corr.map((c) => {
    if (!c.some((v) => mag(v) > 1e-5)) return null;
    const held = c.map((_, t) => {
      let best: Vec3 = [0, 0, 0], bm = 0;
      for (let k = -spread; k <= spread; k++) {
        const v = c[t + k];
        if (!v) continue;
        const w = 1 - Math.abs(k) / (spread + 1), m = mag(v) * w;
        if (m > bm) { bm = m; best = [v[0] * w, v[1] * w, v[2] * w]; }
      }
      return best;
    });
    for (let pass = 0; pass < 2; pass++) { const h = held.map((v) => [...v] as Vec3); for (let t = 1; t < n - 1; t++) for (let a = 0; a < 3; a++) held[t][a] = 0.25 * h[t - 1][a] + 0.5 * h[t][a] + 0.25 * h[t + 1][a]; }
    // (never less than the frame's own: averaged, a peak came out short of clear)
    return held.map((v, t) => (mag(v) >= mag(c[t]) ? v : c[t]));
  });
  return poses.map((p, f) => {
    let q = p;
    sides.forEach((u, k) => {
      const s = smoothed[k];
      if (!s || mag(s[f]) < 1e-5) return;
      q = setWorld(skel, q, pose(skel, q), u, qmul(qexp(s[f]), pose(skel, q).rot[u]));
    });
    return q;
  });
}
