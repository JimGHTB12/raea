// bevy_color's other spaces (docs/control.feature FC1f): Hsla, Oklaba, Oklcha and Xyza, each converted to and from
// LinearRgba and Srgba by Bevy's own formulas in f32 (crates/bevy_color: oklaba.rs, xyza.rs, oklcha.rs, and Hsla
// through Hsva and Hwba), and mixed as Bevy's Mix does (a hue the short way round).
import { LinearRgba, Srgba } from "./all";

const f = Math.fround;
const remEuclid = (a: number, b: number) => f(((a % b) + b) % b);
/** bevy_color's lerp_hue: the short way round, in [0, 360) */
const lerpHue = (a: number, b: number, t: number) => remEuclid(f(a + f(f(remEuclid(f(b - a + 180), 360) - 180) * t)), 360);
const lerp = (a: number, b: number, t: number) => f(f(a * f(1 - t)) + f(b * t));
type Rgb = LinearRgba | Srgba;
const linearOf = (c: Rgb): LinearRgba => (c instanceof LinearRgba ? c : LinearRgba.from(c));
const srgbaOf = (c: Rgb): Srgba => (c instanceof Srgba ? c : c.to_srgba());

/** Bevy's Oklaba: perceptual lightness and two opponent axes. */
export class Oklaba {
  constructor(public lightness: number, public a: number, public b: number, public alpha = 1) {}
  static from(c: Rgb | Oklcha): Oklaba {
    if (c instanceof Oklcha) {
      const h = f((c.hue * Math.PI) / 180);
      return new Oklaba(c.lightness, f(c.chroma * f(Math.cos(h))), f(c.chroma * f(Math.sin(h))), c.alpha);
    }
    const { red: r, green: g, blue: b, alpha } = linearOf(c);
    const l = f(Math.cbrt(f(f(f(0.41222146 * r) + f(0.53633255 * g)) + f(0.051445995 * b))));
    const m = f(Math.cbrt(f(f(f(0.2119035 * r) + f(0.6806995 * g)) + f(0.10739696 * b))));
    const s = f(Math.cbrt(f(f(f(0.08830246 * r) + f(0.28171885 * g)) + f(0.6299787 * b))));
    return new Oklaba(f(f(f(0.21045426 * l) + f(0.7936178 * m)) - f(0.004072047 * s)), f(f(f(1.9779985 * l) - f(2.4285922 * m)) + f(0.4505937 * s)), f(f(f(0.025904037 * l) + f(0.78277177 * m)) - f(0.80867577 * s)), alpha);
  }
  to_linear(): LinearRgba {
    const { lightness: L, a, b } = this;
    const l = f(f(L + f(0.39633778 * a)) + f(0.21580376 * b)) ** 3;
    const m = f(f(L - f(0.105561346 * a)) - f(0.06385417 * b)) ** 3;
    const s = f(f(L - f(0.08948418 * a)) - f(1.2914855 * b)) ** 3;
    return new LinearRgba(f(4.0767417 * l - 3.3077116 * m + 0.23096994 * s), f(-1.268438 * l + 2.6097574 * m - 0.34131938 * s), f(-0.0041960863 * l - 0.7034186 * m + 1.7076147 * s), this.alpha);
  }
  to_srgba(): Srgba {
    return this.to_linear().to_srgba();
  }
  mix(o: Oklaba, t: number): Oklaba {
    return new Oklaba(lerp(this.lightness, o.lightness, t), lerp(this.a, o.a, t), lerp(this.b, o.b, t), lerp(this.alpha, o.alpha, t));
  }
}

/** Bevy's Oklcha: Oklab's lightness, chroma and hue (degrees). */
export class Oklcha {
  constructor(public lightness: number, public chroma: number, public hue: number, public alpha = 1) {}
  static from(c: Rgb | Oklaba): Oklcha {
    const o = c instanceof Oklaba ? c : Oklaba.from(c);
    const hue = f((Math.atan2(o.b, o.a) * 180) / Math.PI);
    return new Oklcha(o.lightness, f(Math.hypot(o.a, o.b)), hue < 0 ? f(hue + 360) : hue, o.alpha);
  }
  to_linear(): LinearRgba {
    return Oklaba.from(this).to_linear();
  }
  to_srgba(): Srgba {
    return this.to_linear().to_srgba();
  }
  mix(o: Oklcha, t: number): Oklcha {
    return new Oklcha(lerp(this.lightness, o.lightness, t), lerp(this.chroma, o.chroma, t), lerpHue(this.hue, o.hue, t), lerp(this.alpha, o.alpha, t));
  }
  /** Bevy's Luminance::lighter: the lightness raised by `amount`, at most 1 */
  lighter(amount: number): Oklcha {
    return new Oklcha(Math.min(f(this.lightness + amount), 1), this.chroma, this.hue, this.alpha);
  }
  /** Bevy's Luminance::darker: the lightness lowered by `amount`, at least 0 */
  darker(amount: number): Oklcha {
    return new Oklcha(Math.max(f(this.lightness - amount), 0), this.chroma, this.hue, this.alpha);
  }
  /** Bevy's Alpha::with_alpha */
  with_alpha(alpha: number): Oklcha {
    return new Oklcha(this.lightness, this.chroma, this.hue, alpha);
  }
}

/** Bevy's Xyza: CIE 1931 XYZ (D65, as linear sRGB's). */
export class Xyza {
  constructor(public x: number, public y: number, public z: number, public alpha = 1) {}
  static from(c: Rgb): Xyza {
    const { red: r, green: g, blue: b, alpha } = linearOf(c);
    return new Xyza(f(r * 0.4124564 + g * 0.3575761 + b * 0.1804375), f(r * 0.2126729 + g * 0.7151522 + b * 0.072175), f(r * 0.0193339 + g * 0.119192 + b * 0.9503041), alpha);
  }
  to_linear(): LinearRgba {
    const { x, y, z } = this;
    return new LinearRgba(f(x * 3.2404542 + y * -1.5371385 + z * -0.4985314), f(x * -0.969266 + y * 1.8760108 + z * 0.041556), f(x * 0.0556434 + y * -0.2040259 + z * 1.0572252), this.alpha);
  }
  to_srgba(): Srgba {
    return this.to_linear().to_srgba();
  }
  mix(o: Xyza, t: number): Xyza {
    return new Xyza(lerp(this.x, o.x, t), lerp(this.y, o.y, t), lerp(this.z, o.z, t), lerp(this.alpha, o.alpha, t));
  }
}

/** Bevy's Hsla: hue (degrees), saturation and lightness, each 0..1; converted through Hsva and Hwba, as Bevy's is. */
export class Hsla {
  constructor(public hue: number, public saturation: number, public lightness: number, public alpha = 1) {}
  static from(c: Rgb): Hsla {
    const { red, green, blue, alpha } = srgbaOf(c);
    // Srgba -> Hwba
    const max = Math.max(0, red, green, blue);
    const min = Math.min(1, red, green, blue);
    const chroma = f(max - min);
    let hue = chroma === 0 ? 0 : red === max ? f(f(60 * f(green - blue)) / chroma) : green === max ? f(60 * f(2 + f(f(blue - red) / chroma))) : f(60 * f(4 + f(f(red - green) / chroma)));
    if (hue < 0) hue = f(360 + hue);
    const [w, bl] = [min, f(1 - max)];
    // Hwba -> Hsva -> Hsla
    const v = f(1 - bl);
    const s = v !== 0 ? f(1 - f(w / v)) : 0;
    const l = f(v * f(1 - f(s / 2)));
    return new Hsla(hue, l === 0 || l === 1 ? 0 : f(f(v - l) / Math.min(l, f(1 - l))), l, alpha);
  }
  to_srgba(): Srgba {
    const { hue, saturation: s0, lightness: l } = this;
    // Hsla -> Hsva -> Hwba
    const v = f(l + f(s0 * Math.min(l, f(1 - l))));
    const s = v === 0 ? 0 : f(2 * f(1 - f(l / v)));
    const w = f(f(1 - s) * v);
    // Hwba -> Srgba
    const h = f(f(hue % 360) / 60);
    const i = Math.floor(h);
    const fr = i % 2 === 0 ? f(h - i) : f(1 - f(h - i));
    const n = f(w + f(fr * f(v - w)));
    const rgb: [number, number, number] = [[v, n, w], [n, v, w], [w, v, n], [w, n, v], [n, w, v], [v, w, n]][i] as [number, number, number];
    return new Srgba(rgb[0], rgb[1], rgb[2], this.alpha);
  }
  to_linear(): LinearRgba {
    return LinearRgba.from(this.to_srgba());
  }
  mix(o: Hsla, t: number): Hsla {
    return new Hsla(lerpHue(this.hue, o.hue, t), lerp(this.saturation, o.saturation, t), lerp(this.lightness, o.lightness, t), lerp(this.alpha, o.alpha, t));
  }
}
