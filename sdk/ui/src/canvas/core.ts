// Matrices and colours.

export type Mat = [number, number, number, number, number, number];

export type RGBA = [number, number, number, number];

export const ID: Mat = [1, 0, 0, 1, 0, 0];

export function mul(m: Mat, n: Mat): Mat {
  return [m[0] * n[0] + m[2] * n[1], m[1] * n[0] + m[3] * n[1], m[0] * n[2] + m[2] * n[3], m[1] * n[2] + m[3] * n[3], m[0] * n[4] + m[2] * n[5] + m[4], m[1] * n[4] + m[3] * n[5] + m[5]];
}

export function apply(m: Mat, x: number, y: number): [number, number] {
  return [m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5]];
}

function invert(m: Mat): Mat | null {
  const det = m[0] * m[3] - m[1] * m[2];
  if (!det) return null;
  const a = m[3] / det, b = -m[1] / det, c = -m[2] / det, d = m[0] / det;
  return [a, b, c, d, -(a * m[4] + c * m[5]), -(b * m[4] + d * m[5])];
}

// ---------------------------------------------------------------- colours

const NAMED: Record<string, string> = {
  black: "#000000", white: "#ffffff", red: "#ff0000", green: "#008000", blue: "#0000ff", yellow: "#ffff00", cyan: "#00ffff", aqua: "#00ffff",
  magenta: "#ff00ff", fuchsia: "#ff00ff", gray: "#808080", grey: "#808080", silver: "#c0c0c0", maroon: "#800000", olive: "#808000", lime: "#00ff00",
  navy: "#000080", purple: "#800080", teal: "#008080", orange: "#ffa500", pink: "#ffc0cb", brown: "#a52a2a", gold: "#ffd700", indigo: "#4b0082",
  violet: "#ee82ee", coral: "#ff7f50", salmon: "#fa8072", tomato: "#ff6347", crimson: "#dc143c", khaki: "#f0e68c", lavender: "#e6e6fa",
  beige: "#f5f5dc", ivory: "#fffff0", tan: "#d2b48c", chocolate: "#d2691e", darkgray: "#a9a9a9", lightgray: "#d3d3d3", darkblue: "#00008b",
  darkgreen: "#006400", darkred: "#8b0000", lightblue: "#add8e6", lightgreen: "#90ee90", skyblue: "#87ceeb", steelblue: "#4682b4",
  slategray: "#708090", whitesmoke: "#f5f5f5", rebeccapurple: "#663399",
};

function hue(h: number, s: number, l: number): RGBA {
  const k = (n: number) => (n + h / 30) % 12;
  const a = s * Math.min(l, 1 - l);
  const f = (n: number) => l - a * Math.max(-1, Math.min(k(n) - 3, 9 - k(n), 1));
  return [f(0) * 255, f(8) * 255, f(4) * 255, 1];
}

/** A CSS colour as RGBA (0-255, alpha 0-1), or null if it doesn't parse (the style is then left as it was). */
export function parseColor(input: string): RGBA | null {
  let s = String(input).trim().toLowerCase();
  if (s === "transparent") return [0, 0, 0, 0];
  if (NAMED[s]) s = NAMED[s];
  if (s[0] === "#") {
    const h = s.slice(1);
    const x = h.length <= 4 ? h.split("").map((c) => c + c).join("") : h;
    if (!/^[0-9a-f]{6}([0-9a-f]{2})?$/.test(x)) return null;
    const n = (i: number) => parseInt(x.slice(i, i + 2), 16);
    return [n(0), n(2), n(4), x.length === 8 ? n(6) / 255 : 1];
  }
  const m = s.match(/^(rgba?|hsla?)\((.*)\)$/);
  if (!m) return null;
  const parts = m[2].split(/[\s,/]+/).filter(Boolean);
  const num = (p: string, scale: number) => (p.endsWith("%") ? (parseFloat(p) / 100) * scale : parseFloat(p));
  if (parts.length < 3) return null;
  const alpha = parts[3] !== undefined ? num(parts[3], 1) : 1;
  if (m[1].startsWith("rgb")) return [num(parts[0], 255), num(parts[1], 255), num(parts[2], 255), alpha];
  const c = hue(((parseFloat(parts[0]) % 360) + 360) % 360, num(parts[1], 1) > 1 ? num(parts[1], 1) / 100 : num(parts[1], 1), num(parts[2], 1) > 1 ? num(parts[2], 1) / 100 : num(parts[2], 1));
  c[3] = alpha;
  return c;
}
