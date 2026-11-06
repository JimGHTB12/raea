// The head held steady (docs/locomotion.md, U4): a walker's head stays nearly level in the world whatever the trunk
// does under it, the vestibulo-collic reflex, pitch held within a few degrees in walking (Pozzo et al. 1990). Each frame
// the head's tilt in the world (its roll about the way the hips face, its pitch about their side) follows a critically
// damped second-order spring at `hz`; the head keeps `keep` of what the tilt does faster than that, and the spring's
// slow part all of it, so a lean into a turn or a look down stays and the sway on top goes.
import type { Vec3 } from "raea/3d";
import { B, joint, pose, qaxis, qmul, qrot, setWorld, type Pose, type Skeleton } from "raea/character";

export interface SteadyHead { hz?: number; keep?: number; keepPitch?: number }

export function steadyHead(skel: Skeleton, poses: Pose[], fps = 30, o: SteadyHead = {}): Pose[] {
  const head = joint(skel, B.head), n = poses.length;
  if (n < 3) return poses;
  const hz = o.hz ?? Number(process.env.RAEA_STEADY_HZ ?? 1.5), keep = o.keep ?? Number(process.env.RAEA_STEADY_KEEP ?? 0.35);
  const keepP = o.keepPitch ?? Number(process.env.RAEA_STEADY_KEEP_PITCH ?? 0.6);
  const W = poses.map((p) => pose(skel, p));
  const frame = (f: number) => { const v = qrot(W[f].rot[0], [0, 0, 1]), l = Math.hypot(v[0], v[2]) || 1; const fw: Vec3 = [v[0] / l, 0, v[2] / l]; return { fw, side: [fw[2], 0, -fw[0]] as Vec3 }; };
  const tilt = (f: number) => { const q = W[f].rot[head]; return [Math.asin(Math.max(-1, Math.min(1, qrot(q, [1, 0, 0])[1]))), Math.asin(Math.max(-1, Math.min(1, qrot(q, [0, 0, 1])[1])))]; };
  // (t3ssel8r's second-order system with z = 1, r = 0: k1 = 1/(pi f), k2 = 1/(2 pi f)^2, stepped twice a frame)
  const k1 = 1 / (Math.PI * hz), k2 = 1 / (2 * Math.PI * hz) ** 2, dt = 1 / fps / 2;
  const T = poses.map((_, f) => tilt(f)), y = [...T[0]], v = [0, 0], out = poses.slice();
  for (let f = 0; f < n; f++) {
    for (let s = 0; s < 2; s++) for (let c = 0; c < 2; c++) { v[c] += (dt * (T[f][c] - y[c] - k1 * v[c])) / k2; y[c] += dt * v[c]; }
    const dr = (keep - 1) * (T[f][0] - y[0]), dp = (keepP - 1) * (T[f][1] - y[1]), { fw, side } = frame(f);
    if (Math.abs(dr) + Math.abs(dp) < 1e-4) continue;
    // (pitch measured up as the face's forward rises: a turn about the side axis (+x for a body facing +z) lowers it)
    out[f] = setWorld(skel, poses[f], W[f], head, qmul(qmul(qaxis(fw, dr), qaxis(side, -dp)), W[f].rot[head]));
  }
  return out;
}
