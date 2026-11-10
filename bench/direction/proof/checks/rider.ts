// A rider turns with its seat and is in no part of it (DR4's fifth proof: a 3D scene). The parts are named in the
// rule's values (`rider`; `seat`, and every part whose name starts with it is the seat's); their rotations are
// compared, and each part's own box, turned as it is, is tested against the rider's. That the rider rides the seat
// is DR16's Rides; until then the rule names them.
import { check, found, type Part } from "raea/check";

type V = number[];
const dot = (u: V, v: V) => u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
const cross = (u: V, v: V) => [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];

/** The angle between two rotations (x, y, z, w), in degrees. */
const turn = (a: V, b: V) => (2 * Math.acos(Math.min(1, Math.abs(a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]))) * 180) / Math.PI;

/** A part as a box turned as it is: its centre, its axes and its half sizes along them, in world metres. A part with
 * no box of its own is its world box. */
function box(p: Part): { c: V; axes: V[]; half: V } | null {
  if (!p.own) return p.min && p.max ? { c: [0, 1, 2].map((k) => (p.min![k] + p.max![k]) / 2), axes: [[1, 0, 0], [0, 1, 0], [0, 0, 1]], half: [0, 1, 2].map((k) => (p.max![k] - p.min![k]) / 2) } : null;
  const [lo, hi] = p.own;
  const [x, y, z, w] = p.rotation;
  // v turned by the part's rotation
  const rot = (v: V) => {
    const t = [2 * (y * v[2] - z * v[1]), 2 * (z * v[0] - x * v[2]), 2 * (x * v[1] - y * v[0])];
    return [v[0] + w * t[0] + (y * t[2] - z * t[1]), v[1] + w * t[1] + (z * t[0] - x * t[2]), v[2] + w * t[2] + (x * t[1] - y * t[0])];
  };
  const c = rot([0, 1, 2].map((k) => ((lo[k] + hi[k]) / 2) * p.scale[k])).map((v, k) => v + p.at[k]);
  return { c, axes: [[1, 0, 0], [0, 1, 0], [0, 0, 1]].map(rot), half: [0, 1, 2].map((k) => (Math.abs(hi[k] - lo[k]) / 2) * Math.abs(p.scale[k])) };
}

/** How far two turned boxes are in each other: the least overlap along the axes that could part them (0: apart). */
function depth(a: NonNullable<ReturnType<typeof box>>, b: NonNullable<ReturnType<typeof box>>): number {
  const d = [0, 1, 2].map((k) => b.c[k] - a.c[k]);
  const axes = [...a.axes, ...b.axes, ...a.axes.flatMap((u) => b.axes.map((v) => cross(u, v)))];
  let least = Infinity;
  for (const ax of axes) {
    const len = Math.hypot(ax[0], ax[1], ax[2]);
    if (len < 1e-9) continue;
    const n = ax.map((v) => v / len);
    const reach = (o: typeof a) => o.half.reduce((s, h, k) => s + h * Math.abs(dot(o.axes[k], n)), 0);
    const overlap = reach(a) + reach(b) - Math.abs(dot(d, n));
    if (overlap <= 0) return 0;
    least = Math.min(least, overlap);
  }
  return least;
}

export default check({ id: "rider", text: "a rider turns with its seat and is in no part of it" }, (world, _frame, values) => {
  const parts = world.find({ with: ["Part"] });
  const part = (id: number) => world.component<Part>(id, "Part")!;
  const rider = parts.find((p) => part(p).name === String(values.rider));
  const seat = parts.find((p) => part(p).name === String(values.seat));
  // a piece with neither has no rider to hold; one without the other is a name gone wrong
  if (rider == null && seat == null) return;
  if (rider == null || seat == null) return found({ says: `there's no part named ${rider == null ? values.rider : values.seat} on this frame` });
  const deg = turn(part(rider).rotation, part(seat).rotation);
  const most = Number(values.turn ?? 0.5);
  if (deg > most) found({ about: [rider, seat], says: `the rider is turned ${deg.toFixed(1)} degrees from its seat`, measure: { name: "turn", value: Math.round(deg * 10) / 10, unit: "deg", limit: most } });
  const into = Number(values.into ?? 0.01);
  const r = box(part(rider));
  for (const s of parts.filter((p) => p !== rider && part(p).name.startsWith(String(values.seat)))) {
    const q = box(part(s));
    if (!r || !q) continue;
    const cm = depth(r, q);
    if (cm > into) found({ about: [rider, s], says: `the rider is ${Math.round(cm * 100)} cm into ${part(s).name}`, measure: { name: "into", value: Math.round(cm * 100), unit: "cm", limit: Math.round(into * 100) } });
  }
});
