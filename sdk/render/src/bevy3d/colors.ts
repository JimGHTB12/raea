// Bevy's colours, Srgba and LinearRgba, and how a material sends them as the old material's colour props.
import { toHex } from "../3d";

// the colour and scale a LinearRgba was made from (LinearRgba.from(Srgba.hex(c)).mul(k)), so StandardMaterial sends
// the old material's emissive and emissiveIntensity exactly rather than an 8-bit colour back from the floats
export const MADE_FROM = Symbol("raea.madeFrom");

type MadeFrom = { hex: string; k: number };

const srgbToLinear = (c: number) => (c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4));

const linearToSrgb = (c: number) => (c <= 0.0031308 ? c * 12.92 : 1.055 * Math.pow(c, 1 / 2.4) - 0.055);

/** Bevy's Srgba: gamma-encoded colour, each channel 0..1. */
export class Srgba {
  [MADE_FROM]?: MadeFrom;
  constructor(public red: number, public green: number, public blue: number, public alpha = 1) {}
  /** Bevy's Srgba::hex: #rgb, #rrggbb, rgb() or a common name */
  static hex(hex: string): Srgba {
    const h = toHex("Srgba.hex", "hex", hex);
    const c = [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16) / 255);
    const s = new Srgba(c[0], c[1], c[2]);
    s[MADE_FROM] = { hex, k: 1 };
    return s;
  }
  /** Bevy's Alpha::with_alpha */
  with_alpha(alpha: number): Srgba {
    const s = new Srgba(this.red, this.green, this.blue, alpha);
    s[MADE_FROM] = this[MADE_FROM];
    return s;
  }
}

// a base colour as the old material takes it: its colour (the one it was made from, exactly) and its alpha
export function baseOf(c: string | Srgba | undefined): { color: string; alpha: number } {
  if (c === undefined) return { color: "#ffffff", alpha: 1 };
  if (typeof c === "string") return { color: c, alpha: 1 };
  const f = c[MADE_FROM];
  if (f) return { color: f.hex, alpha: c.alpha };
  const byte = (v: number) => Math.round(Math.max(0, Math.min(1, v)) * 255).toString(16).padStart(2, "0");
  return { color: "#" + byte(c.red) + byte(c.green) + byte(c.blue), alpha: c.alpha };
}

/** Bevy's LinearRgba: linear light, which a StandardMaterial's emissive is; a channel past 1 is brighter than white. */
export class LinearRgba {
  [MADE_FROM]?: MadeFrom;
  constructor(public red: number, public green: number, public blue: number, public alpha = 1) {}
  static rgb(red: number, green: number, blue: number): LinearRgba {
    return new LinearRgba(red, green, blue);
  }
  /** Bevy's LinearRgba::from(Srgba) */
  static from(c: Srgba): LinearRgba {
    const l = new LinearRgba(srgbToLinear(c.red), srgbToLinear(c.green), srgbToLinear(c.blue), c.alpha);
    l[MADE_FROM] = c[MADE_FROM];
    return l;
  }
  /** Bevy's `* k`: each colour channel scaled (alpha kept) */
  mul(k: number): LinearRgba {
    const l = new LinearRgba(this.red * k, this.green * k, this.blue * k, this.alpha);
    const f = this[MADE_FROM];
    if (f) l[MADE_FROM] = { hex: f.hex, k: f.k * k };
    return l;
  }
}

// a StandardMaterial's emissive as the old material takes it: a colour and its intensity. One made from a colour goes
// as that colour; raw floats go as their hue at full strength (8-bit, as any colour row) times their brightest channel.
export function emissiveOf(e: string | LinearRgba): { emissive?: string; emissiveIntensity?: number } {
  if (typeof e === "string") return { emissive: e };
  const f = e[MADE_FROM];
  if (f) return { emissive: f.hex, ...(f.k !== 1 ? { emissiveIntensity: f.k } : {}) };
  const k = Math.max(e.red, e.green, e.blue);
  if (!(k > 0)) return {};
  const byte = (c: number) => Math.round(Math.max(0, Math.min(1, linearToSrgb(c / k))) * 255).toString(16).padStart(2, "0");
  return { emissive: "#" + byte(e.red) + byte(e.green) + byte(e.blue), ...(k !== 1 ? { emissiveIntensity: k } : {}) };
}
