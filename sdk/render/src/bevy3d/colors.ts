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
  /** Bevy's Srgba::hex: rgb, rgba, rrggbb or rrggbbaa (the # optional), and also rgb() or a common name */
  static hex(hex: string): Srgba {
    // Bevy's own forms: 3, 4, 6 or 8 hex digits, with or without the #, the last of 4 or 8 its alpha
    const raw = typeof hex === "string" ? hex.trim().replace(/^#/, "") : "";
    if (/^([0-9a-f]{3,4}|[0-9a-f]{6}|[0-9a-f]{8})$/i.test(raw)) {
      const d = (raw.length <= 4 ? [...raw].map((x) => x + x).join("") : raw).toLowerCase();
      const c = [0, 2, 4, 6].map((i) => (i < d.length ? parseInt(d.slice(i, i + 2), 16) / 255 : 1));
      const s = new Srgba(c[0], c[1], c[2], c[3]);
      s[MADE_FROM] = { hex: "#" + d.slice(0, 6), k: 1 };
      return s;
    }
    const h = toHex("Srgba.hex", "hex", hex);
    const c = [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16) / 255);
    const s = new Srgba(c[0], c[1], c[2]);
    s[MADE_FROM] = { hex, k: 1 };
    return s;
  }
  /** Bevy's Alpha::with_alpha */
  /** Bevy's Luminance::darker on an Srgba: its linear colour's, encoded again */
  darker(amount: number): Srgba {
    return LinearRgba.from(this).darker(amount).to_srgba();
  }
  /** Bevy's Luminance::lighter on an Srgba: its linear colour's, encoded again */
  lighter(amount: number): Srgba {
    return LinearRgba.from(this).lighter(amount).to_srgba();
  }
  with_alpha(alpha: number): Srgba {
    const s = new Srgba(this.red, this.green, this.blue, alpha);
    s[MADE_FROM] = this[MADE_FROM];
    return s;
  }
  /** Bevy's Srgba::to_hex: #RRGGBB (and AA when not opaque), each channel clamped and rounded */
  to_hex(): string {
    const byte = (v: number) => Math.round(Math.max(0, Math.min(1, v)) * 255).toString(16).padStart(2, "0").toUpperCase();
    return "#" + byte(this.red) + byte(this.green) + byte(this.blue) + (this.alpha < 1 ? byte(this.alpha) : "");
  }
  /** Bevy's Mix: each channel lerped, in f32 */
  mix(o: Srgba, t: number): Srgba {
    const m = (a: number, b: number) => Math.fround(Math.fround(a * Math.fround(1 - t)) + Math.fround(b * t));
    return new Srgba(m(this.red, o.red), m(this.green, o.green), m(this.blue, o.blue), m(this.alpha, o.alpha));
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
  /** Bevy's Srgba::from(LinearRgba): gamma-encoded (a channel at or below 0 kept as it is) */
  to_srgba(): Srgba {
    const g = (c: number) => (c <= 0 ? c : Math.fround(linearToSrgb(c)));
    return new Srgba(g(this.red), g(this.green), g(this.blue), this.alpha);
  }
  /** Bevy's Luminance::luminance: Rec. 709's weights over the linear channels */
  luminance(): number {
    return this.red * 0.2126 + this.green * 0.7152 + this.blue * 0.0722;
  }
  /** Bevy's Luminance::darker: mixed toward black until its luminance is `amount` lower (at least 0) */
  darker(amount: number): LinearRgba {
    return this.lightness(-amount);
  }
  /** Bevy's Luminance::lighter: mixed toward white until its luminance is `amount` higher (at most 1) */
  lighter(amount: number): LinearRgba {
    return this.lightness(amount);
  }
  private lightness(amount: number): LinearRgba {
    if (!Number.isFinite(amount)) throw new RangeError(`LinearRgba.darker/lighter: amount is a finite number, got ${amount}`);
    const l = this.luminance();
    const t = Math.max(0, Math.min(1, l + amount));
    if (t < l) return this.mix(new LinearRgba(0, 0, 0, this.alpha), (l - t) / l);
    if (t > l) return this.mix(new LinearRgba(1, 1, 1, this.alpha), (t - l) / (1 - l));
    return new LinearRgba(this.red, this.green, this.blue, this.alpha);
  }
  /** Bevy's Mix: each channel lerped, in f32 */
  mix(o: LinearRgba, t: number): LinearRgba {
    const m = (a: number, b: number) => Math.fround(Math.fround(a * Math.fround(1 - t)) + Math.fround(b * t));
    return new LinearRgba(m(this.red, o.red), m(this.green, o.green), m(this.blue, o.blue), m(this.alpha, o.alpha));
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
// Its strength is Bevy's, as written (`emissive_nits`, core/scene emissive_power), not a 3d.tsx material's relative
// emissiveIntensity: Bevy's bloom_3d glows at LinearRgba 150 and 1000.
export function emissiveOf(e: string | LinearRgba): { emissive?: string; emissive_nits?: number } {
  if (typeof e === "string") return { emissive: e, emissive_nits: 1 };
  const f = e[MADE_FROM];
  if (f) return { emissive: f.hex, emissive_nits: f.k };
  const k = Math.max(e.red, e.green, e.blue);
  if (!(k > 0)) return {};
  const byte = (c: number) => Math.round(Math.max(0, Math.min(1, linearToSrgb(c / k))) * 255).toString(16).padStart(2, "0");
  return { emissive: "#" + byte(e.red) + byte(e.green) + byte(e.blue), emissive_nits: k };
}
