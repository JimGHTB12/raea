// ctx.filter's CSS filter functions, parsed for the host (systems/canvas filter.rs): each a [name, ...args] with
// lengths in the bitmap's pixels (px, unitless 0), angles in degrees, amounts as numbers (a percentage over 100) and a
// drop-shadow's colour as RGBA. url() isn't drawn yet: a list holding one parses to null, as an invalid one does.
import { RGBA, parseColor } from "./all";

export type FilterFn = [string, ...(number | RGBA)[]];

const AMOUNTS = new Set(["brightness", "contrast", "grayscale", "invert", "opacity", "saturate", "sepia"]);

function amount(s: string | undefined): number | null {
  if (s === undefined) return 1;
  const v = parseFloat(s);
  if (!Number.isFinite(v) || v < 0 || !/^[+]?(\d*\.?\d+(e[+-]?\d+)?)%?$/.test(s)) return null;
  return s.endsWith("%") ? v / 100 : v;
}

function length(s: string): number | null {
  const m = s.match(/^([+-]?\d*\.?\d+(?:e[+-]?\d+)?)(px)?$/);
  if (!m || (!m[2] && parseFloat(m[1]) !== 0)) return null;
  return parseFloat(m[1]);
}

function angle(s: string | undefined): number | null {
  if (s === undefined) return 0;
  const m = s.match(/^([+-]?\d*\.?\d+(?:e[+-]?\d+)?)(deg|rad|grad|turn)?$/);
  if (!m || (!m[2] && parseFloat(m[1]) !== 0)) return null;
  const v = parseFloat(m[1]);
  return m[2] === "rad" ? (v * 180) / Math.PI : m[2] === "grad" ? v * 0.9 : m[2] === "turn" ? v * 360 : v;
}

/** The list a filter string names: [] for none, null if it doesn't parse (the setter then keeps the last). */
export function parseFilter(input: string): FilterFn[] | null {
  const s = String(input).trim().toLowerCase();
  if (s === "none") return [];
  const out: FilterFn[] = [];
  const re = /\s*([a-z-]+)\(((?:[^()]|\([^()]*\))*)\)\s*/y;
  let at = 0;
  while (at < s.length) {
    re.lastIndex = at;
    const m = re.exec(s);
    if (!m) return null;
    at = re.lastIndex;
    const [name, body] = [m[1], m[2].trim()];
    const args = body.match(/(?:[a-z-]+\([^()]*\)|#[0-9a-f]+|[^\s,]+)/g) ?? [];
    if (AMOUNTS.has(name)) {
      if (args.length > 1) return null;
      const a = amount(args[0]);
      if (a === null) return null;
      out.push([name, ["grayscale", "invert", "opacity", "sepia"].includes(name) ? Math.min(a, 1) : a]);
    } else if (name === "blur") {
      const r = args[0] === undefined ? 0 : length(args[0]);
      if (r === null || r < 0 || args.length > 1) return null;
      out.push([name, r]);
    } else if (name === "hue-rotate") {
      const a = angle(args[0]);
      if (a === null || args.length > 1) return null;
      out.push([name, a]);
    } else if (name === "drop-shadow") {
      // two or three lengths and a colour before or after them (currentcolor: black, a canvas's default)
      let color: RGBA = [0, 0, 0, 1];
      const lens: number[] = [];
      for (const a of args) {
        const l = length(a);
        if (l !== null) lens.push(l);
        else {
          const c = parseColor(a);
          if (!c) return null;
          color = c;
        }
      }
      if (lens.length < 2 || lens.length > 3 || (lens[2] ?? 0) < 0) return null;
      out.push([name, lens[0], lens[1], lens[2] ?? 0, color]);
    } else return null;
  }
  return out.length ? out : null;
}
