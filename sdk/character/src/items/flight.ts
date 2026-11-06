// interact's things in the air: let go, falling and flying until they land, then a light ball bounces and rolls.
import type { Vec3 } from "raea/3d";
import { qaxis, qmul, scale3, slerp, type Quat } from "raea/character";
import { handling, shapeOf, toLocal } from "../things";
import { Beat, Flight, G, ItemPlan, Track, uprightNear } from "./all";

/** each thing in the air moved on one frame (`dt`), `now` and `placed` set where it is */
export function fly(flying: Map<string, Flight>, holding: Map<string, Beat>, now: Map<string, Track>, placed: Map<string, { at: Vec3; rot: Quat }>, plan: ItemPlan, h: number, dt: number,
  landing: (p: Vec3, below: number) => { y: number; inside: boolean }, supportAt: (p: Vec3, below: number) => number) {
  // the things let go: falling and flying until they land, then a light ball bounces and rolls to a stop
  for (const [id, fl] of flying) {
    if (holding.has(id) || now.get(id)?.held) { flying.delete(id); continue; }
    const it = plan.items[id], ball = shapeOf(it) === "ball", r = it.size[0] / 2;
    if (!fl.landed) {
      const next: Vec3 = [fl.at[0] + fl.vel[0] * dt, fl.at[1] + fl.vel[1] * dt - 0.5 * G * dt * dt, fl.at[2] + fl.vel[2] * dt];
      const { y: floor, inside } = landing(next, fl.at[1]);
      fl.vel = [fl.vel[0], fl.vel[1] - G * dt, fl.vel[2]];
      const w = Math.hypot(...fl.spin);
      if (w > 0) fl.rot = qmul(qaxis(scale3(fl.spin, 1 / w), w * dt), fl.rot);
      if (next[1] <= floor) {
        next[1] = floor;
        const light = handling(it, h).weight === "light";
        if (ball && light && !inside && fl.vel[1] < -1.2 && fl.bounces < 4) {
          fl.vel = [fl.vel[0] * 0.7, -fl.vel[1] * 0.45, fl.vel[2] * 0.7];
          fl.bounces++;
        } else {
          fl.landed = true;
          fl.vel = ball && !inside ? [fl.vel[0] * 0.7, 0, fl.vel[2] * 0.7] : [0, 0, 0];
        }
      }
      fl.at = next;
    } else {
      // a ball rolls (slowing), turning by the way it goes; anything else settles flat over a few frames
      const sp = Math.hypot(fl.vel[0], fl.vel[2]);
      if (ball && sp > 0.02) {
        const k = Math.max(0, sp - 1.5 * dt) / sp;
        fl.vel = [fl.vel[0] * k, 0, fl.vel[2] * k];
        const next: Vec3 = [fl.at[0] + fl.vel[0] * dt, fl.at[1], fl.at[2] + fl.vel[2] * dt];
        // it stops against anything standing in its way
        if (plan.things.some((t) => t.at[1] < 0.02 && toLocal(t, next)[1] < t.size[1] && Math.abs(toLocal(t, next)[0]) < t.size[0] / 2 + r && Math.abs(toLocal(t, next)[2]) < t.size[2] / 2 + r)) fl.vel = [0, 0, 0];
        else {
          const axis: Vec3 = [fl.vel[2] / sp, 0, -fl.vel[0] / sp];
          fl.rot = qmul(qaxis(axis, (sp * k * dt) / Math.max(0.01, r)), fl.rot);
          fl.at = [next[0], supportAt(next, fl.at[1] + 0.01), next[2]];
        }
      } else if (!ball && fl.settle < 1) {
        fl.settle = Math.min(1, fl.settle + 0.2);
        fl.rot = slerp(fl.rot, uprightNear(fl.rot), fl.settle);
      }
    }
    now.set(id, { at: fl.at, rot: fl.rot, held: false });
    placed.set(id, { at: fl.at, rot: fl.rot });
  }
}
