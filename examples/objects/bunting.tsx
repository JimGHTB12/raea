// Bunting, built from raea/3d's public parts (docs/examples.feature EX5): copy it and make it yours.
import { manifold, toHex, type Solid, type Vec3 } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Mesh3d } from "raea/mesh";

function vec(owner: string, k: string, v: unknown): Vec3 {
  if (!Array.isArray(v) || v.length !== 3 || !v.every((n) => typeof n === "number" && Number.isFinite(n))) throw new TypeError(`${owner}: ${k} must be three finite numbers [x, y, z], got ${JSON.stringify(v)}`);
  return v as Vec3;
}

export interface BuntingProps {
  name: string;
  /** where the line's tied at each end, world metres */
  from: Vec3;
  to: Vec3;
  /** how far it sags at the middle (0.12 of its length) */
  sag?: number;
  /** flags along it (12), their colours in turn, their size (0.35 m tall) */
  flags?: number;
  colors?: string[];
  size?: number;
  /** the line's colour */
  line?: string;
}

const buntingCache = new Map<string, { line: Solid; flags: (Solid | null)[] }>();

/** A string of triangular flags hung between two points on a catenary. Its solids are made once per set of props. */
export function Bunting(props: BuntingProps) {
  const from = vec("Bunting", "from", props.from);
  const to = vec("Bunting", "to", props.to);
  const colors = (props.colors ?? ["#d8443a", "#f2c84b", "#3f7cc4", "#f4f1ea"]).map((c) => toHex("Bunting", "colors", c));
  const n = props.flags ?? 12;
  if (!(Number.isInteger(n) && n >= 1 && n <= 64)) throw new RangeError(`Bunting: flags must be a whole number from 1 to 64, got ${n}`);
  const size = props.size ?? 0.35;
  const L = Math.hypot(to[0] - from[0], to[1] - from[1], to[2] - from[2]);
  const sag = (props.sag ?? 0.12) * L;
  const key = JSON.stringify([from, to, sag, n, size, colors.length]);
  let made = buntingCache.get(key);
  if (!made) {
    // a catenary through both ends, dropping `sag` at the middle (a cosh shape, close enough to a hung line)
    const at = (u: number): Vec3 => {
      const c = 3;
      const drop = sag * (1 - (Math.cosh(c * (2 * u - 1)) - 1) / (Math.cosh(c) - 1));
      return [from[0] + (to[0] - from[0]) * u, from[1] + (to[1] - from[1]) * u - drop, from[2] + (to[2] - from[2]) * u];
    };
    const segs: Solid[] = [];
    const k = 24;
    const r = Math.max(0.01, size * 0.04);
    for (let i = 0; i < k; i++) {
      const a = at(i / k);
      const b = at((i + 1) / k);
      const pts: Vec3[] = [];
      for (const q of [a, b]) for (const sx of [-r, r]) for (const sy of [-r, r]) for (const sz of [-r, r]) pts.push([q[0] + sx, q[1] + sy, q[2] + sz]);
      segs.push(manifold.hull(pts));
    }
    const dx = (to[0] - from[0]) / L;
    const dz = (to[2] - from[2]) / L;
    const nz = [-dz, dx];
    const byColour: Solid[][] = colors.map(() => []);
    for (let i = 0; i < n; i++) {
      const u0 = (i + 0.15) / n;
      const u1 = (i + 0.85) / n;
      const a = at(u0);
      const b = at(u1);
      const m = at((u0 + u1) / 2);
      const tip: Vec3 = [m[0], m[1] - size, m[2]];
      const th = 0.01;
      const pts: Vec3[] = [];
      for (const s of [-th, th]) for (const p of [a, b, tip]) pts.push([p[0] + nz[0] * s, p[1], p[2] + nz[1] * s]);
      byColour[i % colors.length].push(manifold.hull(pts));
    }
    made = { line: manifold.compose(...segs), flags: byColour.map((f) => (f.length ? manifold.compose(...f) : null)) };
    buntingCache.set(key, made);
  }
  return (
    <>
      <Mesh3d name={`${props.name}-line`} shape={made.line}>
        <StandardMaterial base_color={toHex("Bunting", "line", props.line ?? "#3a3430")} />
      </Mesh3d>
      {made.flags.map((f, i) =>
        f === null ? null : (
          <Mesh3d key={i} name={`${props.name}-flags-${i}`} shape={f}>
            <StandardMaterial base_color={colors[i]} />
          </Mesh3d>
        ),
      )}
    </>
  );
}
