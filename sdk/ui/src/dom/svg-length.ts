// SVGGeometryElement's lengths (#16): getTotalLength and getPointAtLength of path, line, polyline, polygon, circle,
// ellipse and rect, from each shape's outline as SVG 2 defines it (a rect from (x + rx, y) clockwise, a circle and an
// ellipse from (cx + rx, cy) clockwise), curves and arcs flattened finely enough that a length is within about 1e-4
// of its exact value. A line-draw animation sets stroke-dasharray to the length, so it's what that needs.

type Pt = [number, number];
/** an outline: its subpaths, each a polyline (a move starts a new one and adds no length) */
type Outline = Pt[][];

/** segments a curve is flattened into, and an arc's per degree */
const CURVE_STEPS = 256;
const ARC_STEP = Math.PI / 180;

/** The outline of an SVG element (`attr` reads an attribute as a number, 0 when absent), or undefined for a tag with
 *  no geometry. */
export function outline(tag: string, attr: (name: string) => number, d: () => string, points: () => string): Outline | undefined {
  switch (tag) {
    case "path":
      return pathOutline(d());
    case "line":
      return [[[attr("x1"), attr("y1")], [attr("x2"), attr("y2")]]];
    case "polyline":
    case "polygon": {
      const n = numbers(points());
      const pts: Pt[] = [];
      for (let i = 0; i + 1 < n.length; i += 2) pts.push([n[i], n[i + 1]]);
      if (tag === "polygon" && pts.length) pts.push(pts[0]);
      return [pts];
    }
    case "circle":
      return [ellipse(attr("cx"), attr("cy"), attr("r"), attr("r"))];
    case "ellipse":
      return [ellipse(attr("cx"), attr("cy"), attr("rx"), attr("ry"))];
    case "rect":
      return [rect(attr("x"), attr("y"), attr("width"), attr("height"), attr("rx"), attr("ry"))];
  }
  return undefined;
}

export function totalLength(o: Outline): number {
  let sum = 0;
  for (const sub of o) for (let i = 1; i < sub.length; i++) sum += dist(sub[i - 1], sub[i]);
  return sum;
}

/** The point `at` along the outline, clamped to its ends as a browser clamps it. */
export function pointAtLength(o: Outline, at: number): Pt {
  const subs = o.filter((s) => s.length);
  if (!subs.length) return [0, 0];
  let left = Math.max(0, at);
  for (const sub of subs) {
    for (let i = 1; i < sub.length; i++) {
      const d = dist(sub[i - 1], sub[i]);
      if (left <= d && d > 0) {
        const t = left / d;
        return [sub[i - 1][0] + (sub[i][0] - sub[i - 1][0]) * t, sub[i - 1][1] + (sub[i][1] - sub[i - 1][1]) * t];
      }
      left -= d;
    }
  }
  const last = subs[subs.length - 1];
  return last[last.length - 1];
}

const dist = (a: Pt, b: Pt) => Math.hypot(b[0] - a[0], b[1] - a[1]);

function numbers(s: string): number[] {
  return (s.match(/[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?/g) ?? []).map(Number);
}

function ellipse(cx: number, cy: number, rx: number, ry: number): Pt[] {
  if (rx <= 0 || ry <= 0) return [];
  const pts: Pt[] = [];
  const n = 360;
  for (let i = 0; i <= n; i++) {
    const a = (i / n) * 2 * Math.PI;
    pts.push([cx + rx * Math.cos(a), cy + ry * Math.sin(a)]);
  }
  return pts;
}

function rect(x: number, y: number, w: number, h: number, rx0: number, ry0: number): Pt[] {
  if (w <= 0 || h <= 0) return [];
  // SVG 2: an absent rx or ry takes the other's value, each at most half its side
  let rx = rx0 > 0 ? rx0 : ry0 > 0 ? ry0 : 0;
  let ry = ry0 > 0 ? ry0 : rx;
  rx = Math.min(rx, w / 2);
  ry = Math.min(ry, h / 2);
  const pts: Pt[] = [];
  const corner = (cx: number, cy: number, from: number) => {
    for (let i = 1; i <= 90; i++) {
      const a = from + (i / 90) * (Math.PI / 2);
      pts.push([cx + rx * Math.cos(a), cy + ry * Math.sin(a)]);
    }
  };
  pts.push([x + rx, y], [x + w - rx, y]);
  if (rx > 0) corner(x + w - rx, y + ry, -Math.PI / 2);
  pts.push([x + w, y + h - ry]);
  if (rx > 0) corner(x + w - rx, y + h - ry, 0);
  pts.push([x + rx, y + h]);
  if (rx > 0) corner(x + rx, y + h - ry, Math.PI / 2);
  pts.push([x, y + ry]);
  if (rx > 0) corner(x + rx, y + ry, Math.PI);
  return pts;
}

/** A path's d as polylines: every command (absolute and relative) of SVG 2's path grammar. */
export function pathOutline(d: string): Outline {
  const tokens = d.match(/[MmLlHhVvCcSsQqTtAaZz]|[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?/g) ?? [];
  const out: Outline = [];
  let sub: Pt[] = [];
  let cur: Pt = [0, 0];
  let start: Pt = [0, 0];
  let ctrl: Pt | undefined; // the last curve's second control point, for S and T
  let prev = "";
  let i = 0;
  let cmd = "";
  const num = () => Number(tokens[i++]);
  // an arc's flags may be written run together ("a1 1 0 00 10 10"): one digit each
  const flag = () => {
    const t = tokens[i];
    if (t.length > 1 && (t[0] === "0" || t[0] === "1")) {
      tokens[i] = t.slice(1);
      return t[0] === "1";
    }
    i++;
    return Number(t) !== 0;
  };
  const lineTo = (p: Pt) => {
    if (!sub.length) sub.push(cur);
    sub.push(p);
    cur = p;
  };
  const curve = (pts: Pt[]) => {
    if (!sub.length) sub.push(cur);
    const p0 = cur;
    for (let k = 1; k <= CURVE_STEPS; k++) sub.push(bezier([p0, ...pts], k / CURVE_STEPS));
    cur = pts[pts.length - 1];
  };
  while (i < tokens.length) {
    if (/[a-zA-Z]/.test(tokens[i])) cmd = tokens[i++];
    else if (!cmd) break;
    const rel = cmd === cmd.toLowerCase();
    const at = (x: number, y: number): Pt => (rel ? [cur[0] + x, cur[1] + y] : [x, y]);
    const C = cmd.toUpperCase();
    if (C !== "Z" && i >= tokens.length) break;
    switch (C) {
      case "M": {
        const p = at(num(), num());
        if (sub.length) out.push(sub);
        sub = [p];
        cur = start = p;
        cmd = rel ? "l" : "L"; // further pairs are lines
        break;
      }
      case "L":
        lineTo(at(num(), num()));
        break;
      case "H": {
        const x = num();
        lineTo([rel ? cur[0] + x : x, cur[1]]);
        break;
      }
      case "V": {
        const y = num();
        lineTo([cur[0], rel ? cur[1] + y : y]);
        break;
      }
      case "C": {
        const c1 = at(num(), num()), c2 = at(num(), num()), p = at(num(), num());
        curve([c1, c2, p]);
        ctrl = c2;
        break;
      }
      case "S": {
        const c1: Pt = ctrl && /[CS]/.test(prev) ? [2 * cur[0] - ctrl[0], 2 * cur[1] - ctrl[1]] : cur;
        const c2 = at(num(), num()), p = at(num(), num());
        curve([c1, c2, p]);
        ctrl = c2;
        break;
      }
      case "Q": {
        const c = at(num(), num()), p = at(num(), num());
        curve([c, p]);
        ctrl = c;
        break;
      }
      case "T": {
        const c: Pt = ctrl && /[QT]/.test(prev) ? [2 * cur[0] - ctrl[0], 2 * cur[1] - ctrl[1]] : cur;
        const p = at(num(), num());
        curve([c, p]);
        ctrl = c;
        break;
      }
      case "A": {
        const rx = num(), ry = num(), rot = num(), large = flag(), sweep = flag();
        const p = at(num(), num());
        if (!sub.length) sub.push(cur);
        for (const q of arc(cur, p, rx, ry, rot, large, sweep)) sub.push(q);
        cur = p;
        break;
      }
      case "Z":
        if (sub.length) {
          sub.push(start);
          out.push(sub);
          sub = [];
        }
        cur = start;
        break;
      default:
        i = tokens.length;
    }
    prev = C;
  }
  if (sub.length) out.push(sub);
  return out;
}

function bezier(p: Pt[], t: number): Pt {
  let q = p;
  while (q.length > 1) {
    const n: Pt[] = [];
    for (let k = 1; k < q.length; k++) n.push([q[k - 1][0] + (q[k][0] - q[k - 1][0]) * t, q[k - 1][1] + (q[k][1] - q[k - 1][1]) * t]);
    q = n;
  }
  return q[0];
}

/** An elliptical arc from p0 to p1 as points after p0 (SVG 2's implementation notes: endpoint to center form). */
function arc(p0: Pt, p1: Pt, rx0: number, ry0: number, rotDeg: number, large: boolean, sweep: boolean): Pt[] {
  if (p0[0] === p1[0] && p0[1] === p1[1]) return [];
  let rx = Math.abs(rx0), ry = Math.abs(ry0);
  if (!rx || !ry) return [p1];
  const phi = (rotDeg * Math.PI) / 180;
  const cos = Math.cos(phi), sin = Math.sin(phi);
  const dx = (p0[0] - p1[0]) / 2, dy = (p0[1] - p1[1]) / 2;
  const x1 = cos * dx + sin * dy, y1 = -sin * dx + cos * dy;
  const lambda = (x1 * x1) / (rx * rx) + (y1 * y1) / (ry * ry);
  if (lambda > 1) {
    rx *= Math.sqrt(lambda);
    ry *= Math.sqrt(lambda);
  }
  const num = rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1;
  const den = rx * rx * y1 * y1 + ry * ry * x1 * x1;
  let k = Math.sqrt(Math.max(0, num / den));
  if (large === sweep) k = -k;
  const cx1 = (k * rx * y1) / ry, cy1 = (-k * ry * x1) / rx;
  const cx = cos * cx1 - sin * cy1 + (p0[0] + p1[0]) / 2, cy = sin * cx1 + cos * cy1 + (p0[1] + p1[1]) / 2;
  const angle = (ux: number, uy: number, vx: number, vy: number) => Math.atan2(ux * vy - uy * vx, ux * vx + uy * vy);
  const t1 = angle(1, 0, (x1 - cx1) / rx, (y1 - cy1) / ry);
  let dt = angle((x1 - cx1) / rx, (y1 - cy1) / ry, (-x1 - cx1) / rx, (-y1 - cy1) / ry);
  if (!sweep && dt > 0) dt -= 2 * Math.PI;
  if (sweep && dt < 0) dt += 2 * Math.PI;
  const n = Math.max(1, Math.ceil(Math.abs(dt) / ARC_STEP));
  const pts: Pt[] = [];
  for (let s = 1; s < n; s++) {
    const t = t1 + (dt * s) / n;
    const ex = rx * Math.cos(t), ey = ry * Math.sin(t);
    pts.push([cos * ex - sin * ey + cx, sin * ex + cos * ey + cy]);
  }
  pts.push(p1);
  return pts;
}
