// bevy_color's Okhsla (docs/control.feature FC0a): Björn Ottosson's okhsl, hue (degrees), saturation and lightness
// each 0..1 over Oklab with the sRGB gamut's cusp and a toe on lightness, converted by Bevy's own translation of the C
// reference (bevy_color okcolor_convert.rs: compute_max_saturation, find_cusp, find_gamut_intersection, get_Cs,
// get_ST_mid, toe), in f32 as Bevy runs it: every operation rounded to f32. Bevy's test colours agree to 1e-4 but one:
// navy's hue sits exactly on the boundary between compute_max_saturation's regions (the test is 1.0 > 1 in Bevy's f32),
// so the last bit of the Oklab coming in picks the region, and raea's sRGB -> Oklab (JavaScript's pow and cbrt) is a
// few ulps from Bevy's (the platform's powf and cbrtf): its saturation reads 1.0247, not 1.
import { LinearRgba, Srgba } from "./colors";
import { Hsla, Oklaba } from "./colorspaces";

const f = Math.fround;
// f32 arithmetic: each result rounded, as Rust's f32 operators do
const add = (a: number, b: number) => f(a + b);
const sub = (a: number, b: number) => f(a - b);
const mul = (a: number, b: number) => f(a * b);
const div = (a: number, b: number) => f(a / b);
const sqrt = (a: number) => f(Math.sqrt(a));
/** a sum of products, left to right: dot(c0, x0, c1, x1, ...) = c0*x0 + c1*x1 + ... */
const dot = (...v: number[]) => {
  let s = mul(v[0], v[1]);
  for (let i = 2; i < v.length; i += 2) s = add(s, mul(v[i], v[i + 1]));
  return s;
};
const F32_MAX = 3.4028234663852886e38;

type LC = { L: number; C: number };
type ST = { S: number; T: number };

const toST = ({ L, C }: LC): ST => ({ S: div(C, L), T: div(C, sub(1, L)) });

// the LMS weights for a hue (a, b) and the R, G and B rows of LMS -> linear sRGB, as f32
const K_L = [f(0.39633778), f(0.21580376)], K_M = [f(-0.105561346), f(-0.06385417)], K_S = [f(-0.08948418), f(-1.2914855)];
const ROW_R = [f(4.0767417), f(-3.3077116), f(0.23096994)];
const ROW_G = [f(-1.268438), f(2.6097574), f(-0.34131938)];
const ROW_B = [f(-0.0041960863), f(-0.7034186), f(1.7076147)];
const kOf = (k: number[], a: number, b: number) => dot(k[0], a, k[1], b);

function computeMaxSaturation(a: number, b: number): number {
  let k: number[], w: number[];
  if (sub(mul(f(-1.8817033), a), mul(f(0.8093649), b)) > 1) {
    k = [1.1908628, 1.7657673, 0.5966264, 0.755152, 0.5677124].map(f);
    w = ROW_R;
  } else if (sub(mul(f(1.8144411), a), mul(f(1.1944528), b)) > 1) {
    k = [0.73956515, -0.45954404, 0.08285427, 0.1254107, 0.14503204].map(f);
    w = ROW_G;
  } else {
    k = [1.3573365, -0.00915799, -1.1513021, -0.50559606, 0.00692167].map(f);
    w = ROW_B;
  }
  // k0 + k1 a + k2 b + k3 a a + k4 a b
  let S = add(add(add(add(k[0], mul(k[1], a)), mul(k[2], b)), mul(mul(k[3], a), a)), mul(mul(k[4], a), b));
  const kl = kOf(K_L, a, b), km = kOf(K_M, a, b), ks = kOf(K_S, a, b);
  for (let i = 0; i < 2; i++) {
    const l_ = add(1, mul(S, kl)), m_ = add(1, mul(S, km)), s_ = add(1, mul(S, ks));
    const l = mul(mul(l_, l_), l_), m = mul(mul(m_, m_), m_), s = mul(mul(s_, s_), s_);
    const ldS = mul(mul(mul(3, kl), l_), l_), mdS = mul(mul(mul(3, km), m_), m_), sdS = mul(mul(mul(3, ks), s_), s_);
    const ldS2 = mul(mul(mul(6, kl), kl), l_), mdS2 = mul(mul(mul(6, km), km), m_), sdS2 = mul(mul(mul(6, ks), ks), s_);
    const fv = dot(w[0], l, w[1], m, w[2], s);
    const f1 = dot(w[0], ldS, w[1], mdS, w[2], sdS);
    const f2 = dot(w[0], ldS2, w[1], mdS2, w[2], sdS2);
    S = sub(S, div(mul(fv, f1), sub(mul(f1, f1), mul(mul(0.5, fv), f2))));
  }
  return S;
}

function findCusp(a: number, b: number): LC {
  const S = computeMaxSaturation(a, b);
  const rgb = new Oklaba(1, mul(S, a), mul(S, b)).to_linear();
  const L = f(Math.cbrt(div(1, Math.max(rgb.red, rgb.green, rgb.blue))));
  return { L, C: mul(L, S) };
}

function findGamutIntersection(a: number, b: number, L1: number, C1: number, L0: number, cusp: LC): number {
  if (sub(mul(sub(L1, L0), cusp.C), mul(sub(cusp.L, L0), C1)) <= 0) return div(mul(cusp.C, L0), add(mul(C1, cusp.L), mul(cusp.C, sub(L0, L1))));
  let t = div(mul(cusp.C, sub(L0, 1)), add(mul(C1, sub(cusp.L, 1)), mul(cusp.C, sub(L0, L1))));
  const dL = sub(L1, L0), dC = C1;
  const kl = kOf(K_L, a, b), km = kOf(K_M, a, b), ks = kOf(K_S, a, b);
  const ldt = add(dL, mul(dC, kl)), mdt = add(dL, mul(dC, km)), sdt = add(dL, mul(dC, ks));
  for (let i = 0; i < 2; i++) {
    const L = add(mul(L0, sub(1, t)), mul(t, L1)), C = mul(t, C1);
    const l_ = add(L, mul(C, kl)), m_ = add(L, mul(C, km)), s_ = add(L, mul(C, ks));
    const l = mul(mul(l_, l_), l_), m = mul(mul(m_, m_), m_), s = mul(mul(s_, s_), s_);
    const l1 = mul(mul(mul(3, ldt), l_), l_), m1 = mul(mul(mul(3, mdt), m_), m_), s1 = mul(mul(mul(3, sdt), s_), s_);
    const l2 = mul(mul(mul(6, ldt), ldt), l_), m2 = mul(mul(mul(6, mdt), mdt), m_), s2 = mul(mul(mul(6, sdt), sdt), s_);
    // Halley's method on each of R, G and B reaching 1, the nearest root taken
    const step = (w: number[]) => {
      const v = sub(dot(w[0], l, w[1], m, w[2], s), 1);
      const v1 = dot(w[0], l1, w[1], m1, w[2], s1);
      const v2 = dot(w[0], l2, w[1], m2, w[2], s2);
      const u = div(v1, sub(mul(v1, v1), mul(mul(0.5, v), v2)));
      return u >= 0 ? mul(-v, u) : F32_MAX;
    };
    t = add(t, Math.min(step(ROW_R), step(ROW_G), step(ROW_B)));
  }
  return t;
}

function stMid(a: number, b: number): ST {
  const c = f;
  const S = add(c(0.11516993), div(1, add(add(c(7.4477897), mul(c(4.1590123), b)), mul(a, add(add(c(-2.1955736), mul(c(1.751984), b)), mul(a, add(sub(c(-2.1370494), mul(c(10.02301), b)), mul(a, add(add(c(-4.2489457), mul(c(5.387708), b)), mul(c(4.69891), a))))))))));
  const T = add(c(0.11239642), div(1, add(sub(c(1.6132032), mul(c(0.6812438), b)), mul(a, add(add(c(0.40370612), mul(c(0.9014812), b)), mul(a, add(add(c(-0.27087943), mul(c(0.6122399), b)), mul(a, sub(sub(c(0.00299215), mul(c(0.45399568), b)), mul(c(0.14661872), a))))))))));
  return { S, T };
}

function getCs(L: number, a: number, b: number) {
  const cusp = findCusp(a, b);
  const C_max = findGamutIntersection(a, b, L, 1, L, cusp);
  const stMax = toST(cusp);
  const k = div(C_max, Math.min(mul(L, stMax.S), mul(sub(1, L), stMax.T)));
  const mid = stMid(a, b);
  const Ca = mul(L, mid.S), Cb = mul(sub(1, L), mid.T);
  const p4 = (x: number) => mul(mul(mul(x, x), x), x);
  const C_mid = mul(mul(f(0.9), k), sqrt(sqrt(div(1, add(div(1, p4(Ca)), div(1, p4(Cb)))))));
  const Ca0 = mul(L, f(0.4)), Cb0 = mul(sub(1, L), f(0.8));
  const C_0 = sqrt(div(1, add(div(1, mul(Ca0, Ca0)), div(1, mul(Cb0, Cb0)))));
  return { C_0, C_mid, C_max };
}

const K1 = f(0.206), K2 = f(0.03), K3 = div(add(1, K1), add(1, K2));
const toe = (x: number) => mul(0.5, add(sub(mul(K3, x), K1), sqrt(add(mul(sub(mul(K3, x), K1), sub(mul(K3, x), K1)), mul(mul(mul(4, K2), K3), x)))));
const toeInv = (x: number) => div(add(mul(x, x), mul(K1, x)), mul(K3, add(x, K2)));
const MID = f(0.8), MID_INV = f(1.25), PI = f(Math.PI);

/** Bevy's Okhsla: Oklab's hue (degrees), a saturation relative to the sRGB gamut, and a lightness with a toe, each
 *  0..1. @bevy bevy_color::Okhsla */
export class Okhsla {
  /** Hue in degrees, saturation, lightness and alpha 0 to 1, in Okhsl. */
  constructor(public hue: number, public saturation: number, public lightness: number, public alpha = 1) {}
  /** From Oklab (bevy_color oklab_to_okhsl). */
  static from_oklaba(lab: Oklaba): Okhsla {
    const L = f(lab.lightness), a = f(lab.a), b = f(lab.b), alpha = lab.alpha;
    if (L >= 1) return new Okhsla(0, 0, 1, alpha);
    if (L <= 0) return new Okhsla(0, 0, 0, alpha);
    const C = sqrt(add(mul(a, a), mul(b, b)));
    if (C === 0) return new Okhsla(0, 0, toe(L), alpha);
    const a_ = div(a, C), b_ = div(b, C);
    const h = add(0.5, div(mul(0.5, f(Math.atan2(-b, -a))), PI));
    const { C_0, C_mid, C_max } = getCs(L, a_, b_);
    let s: number;
    if (C < C_mid) {
      const k1 = mul(MID, C_0), k2 = sub(1, div(k1, C_mid));
      s = mul(div(C, add(k1, mul(k2, C))), MID);
    } else {
      const k0 = C_mid, k1 = div(mul(mul(mul(mul(sub(1, MID), C_mid), C_mid), MID_INV), MID_INV), C_0), k2 = sub(1, div(k1, sub(C_max, C_mid)));
      s = add(MID, mul(sub(1, MID), div(sub(C, k0), add(k1, mul(k2, sub(C, k0))))));
    }
    return new Okhsla(mul(h, 360), s, toe(L), alpha);
  }
  /** From any sRGB colour, through Oklab. */
  static from(c: Srgba | LinearRgba | Hsla): Okhsla {
    return Okhsla.from_oklaba(Oklaba.from(c instanceof Hsla ? c.to_srgba() : c));
  }
  /** To Oklab (bevy_color okhsl_to_oklab). */
  to_oklaba(): Oklaba {
    const s = f(this.saturation), l = f(this.lightness), alpha = this.alpha;
    const h = div(f(this.hue), 360);
    if (l >= 1) return Oklaba.from(new LinearRgba(1, 1, 1, alpha));
    if (l <= 0) return Oklaba.from(new LinearRgba(0, 0, 0, alpha));
    const a_ = f(Math.cos(mul(mul(2, PI), h))), b_ = f(Math.sin(mul(mul(2, PI), h)));
    const L = toeInv(l);
    const { C_0, C_mid, C_max } = getCs(L, a_, b_);
    let C: number;
    if (s < MID) {
      const t = mul(MID_INV, s), k1 = mul(MID, C_0), k2 = sub(1, div(k1, C_mid));
      C = div(mul(t, k1), sub(1, mul(k2, t)));
    } else {
      const t = div(sub(s, MID), sub(1, MID)), k0 = C_mid, k1 = div(mul(mul(mul(mul(sub(1, MID), C_mid), C_mid), MID_INV), MID_INV), C_0), k2 = sub(1, div(k1, sub(C_max, C_mid)));
      C = add(k0, div(mul(t, k1), sub(1, mul(k2, t))));
    }
    return new Oklaba(L, mul(C, a_), mul(C, b_), alpha);
  }
  /** The same colour as a LinearRgba. */
  to_linear(): LinearRgba {
    return this.to_oklaba().to_linear();
  }
  /** The same colour as an Srgba. */
  to_srgba(): Srgba {
    return this.to_oklaba().to_srgba();
  }
  /** The same colour with this alpha. */
  with_alpha(alpha: number): Okhsla {
    return new Okhsla(this.hue, this.saturation, this.lightness, alpha);
  }
}
