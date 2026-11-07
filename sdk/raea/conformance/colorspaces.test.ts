// raea/color's Hsla, Oklaba, Oklcha and Xyza against bevy_color's own table (crates/bevy_color/src/test_colors.rs, Bevy
// v0.20.0): each named colour in sRGB, linear, HSL, Oklab, Oklch and XYZ, converted both ways (docs/control.feature FC1f).
import { describe, expect, test } from "bun:test";
import { Hsla, LinearRgba, Oklaba, Oklcha, Srgba, Xyza } from "raea/color";

type C4 = [number, number, number, number];
// name, rgb, linear_rgb, hsl, oklab, oklch, xyz
const TABLE: [string, C4, C4, C4, C4, C4, C4][] = [
  ["black", [0, 0, 0, 1], [0, 0, 0, 1], [0, 0, 0, 1], [0, 0, 0, 1], [0, 0, 0, 1], [0, 0, 0, 1]],
  ["white", [1, 1, 1, 1], [1, 1, 1, 1], [0, 0, 1, 1], [1, 0, 0.000000059604645, 1], [1, 0.000000059604645, 90, 1], [0.95047, 1, 1.08883, 1]],
  ["red", [1, 0, 0, 1], [1, 0, 0, 1], [0, 1, 0.5, 1], [0.6279554, 0.22486295, 0.1258463, 1], [0.6279554, 0.25768322, 29.233906, 1], [0.4124564, 0.2126729, 0.0193339, 1]],
  ["green", [0, 1, 0, 1], [0, 1, 0, 1], [120, 1, 0.5, 1], [0.8664396, -0.2338874, 0.1794985, 1], [0.8664396, 0.2948271, 142.49532, 1], [0.3575761, 0.7151522, 0.119192, 1]],
  ["blue", [0, 0, 1, 1], [0, 0, 1, 1], [240, 1, 0.5, 1], [0.4520137, -0.032456964, -0.31152815, 1], [0.4520137, 0.31321436, 264.05203, 1], [0.1804375, 0.072175, 0.9503041, 1]],
  ["yellow", [1, 1, 0, 1], [1, 1, 0, 1], [60, 1, 0.5, 1], [0.9679827, -0.07136908, 0.19856972, 1], [0.9679827, 0.21100587, 109.76924, 1], [0.7700325, 0.9278251, 0.1385259, 1]],
  ["magenta", [1, 0, 1, 1], [1, 0, 1, 1], [300, 1, 0.5, 1], [0.7016738, 0.27456632, -0.16915613, 1], [0.7016738, 0.32249102, 328.36343, 1], [0.5928939, 0.28484792, 0.969638, 1]],
  ["cyan", [0, 1, 1, 1], [0, 1, 1, 1], [180, 1, 0.5, 1], [0.90539926, -0.1494439, -0.039398134, 1], [0.90539926, 0.15454996, 194.76895, 1], [0.5380136, 0.78732723, 1.069496, 1]],
  ["olive", [0.5, 0.5, 0, 1], [0.21404114, 0.21404114, 0, 1], [60, 1, 0.25, 1], [0.57902855, -0.042691574, 0.11878061, 1], [0.57902855, 0.12621966, 109.76922, 1], [0.16481864, 0.19859275, 0.029650241, 1]],
  ["purple", [0.5, 0, 0.5, 1], [0.21404114, 0, 0.21404114, 1], [300, 1, 0.25, 1], [0.41972777, 0.1642403, -0.10118592, 1], [0.41972777, 0.19290791, 328.36343, 1], [0.12690368, 0.060969174, 0.20754242, 1]],
  ["teal", [0, 0.5, 0.5, 1], [0, 0.21404114, 0.21404114, 1], [180, 1, 0.25, 1], [0.54159236, -0.08939436, -0.02356726, 1], [0.54159236, 0.09244873, 194.769, 1], [0.11515705, 0.16852042, 0.22891617, 1]],
  ["navy", [0, 0, 0.5, 1], [0, 0, 0.21404114, 1], [240, 1, 0.25, 1], [0.27038592, -0.01941514, -0.18635012, 1], [0.27038592, 0.18735878, 264.05203, 1], [0.03862105, 0.01544842, 0.20340417, 1]],
];

const near = (got: number[], want: number[], tol = 0.001) => expect(got.every((g, i) => Math.abs(g - want[i]!) <= tol)).toBe(true);
const v = (c: { [k: string]: number }, ...k: string[]) => k.map((n) => c[n]!);
const lin = (c: C4) => new LinearRgba(...c);
const srgb = (c: C4) => new Srgba(...c);

describe("bevy_color's spaces, by its table", () => {
  for (const [name, rgb, linear, hsl, oklab, oklch, xyz] of TABLE) {
    test(name, () => {
      near(v(Oklaba.from(lin(linear)), "lightness", "a", "b", "alpha"), oklab);
      near(v(Oklaba.from(lin(linear)).to_linear(), "red", "green", "blue", "alpha"), linear);
      near(v(Xyza.from(lin(linear)), "x", "y", "z", "alpha"), xyz);
      near(v(new Xyza(...xyz).to_linear(), "red", "green", "blue", "alpha"), linear);
      near(v(Hsla.from(srgb(rgb)), "hue", "saturation", "lightness", "alpha"), hsl);
      near(v(new Hsla(...hsl).to_srgba(), "red", "green", "blue", "alpha"), rgb);
      // a hue only means something with chroma (black's and white's are whatever the rounding leaves)
      const ch = Oklcha.from(lin(linear));
      near([ch.lightness, ch.chroma], [oklch[0], oklch[1]]);
      if (oklch[1] > 0.01) near([ch.hue], [oklch[2]], 0.01);
      near(v(new Oklcha(...oklch).to_linear(), "red", "green", "blue"), linear.slice(0, 3));
      near(v(lin(linear).to_srgba(), "red", "green", "blue"), rgb.slice(0, 3));
    });
  }

  test("a hue mixes the short way round; the rest lerp", () => {
    expect(new Hsla(350, 0, 0).mix(new Hsla(10, 1, 1), 0.5).hue).toBe(0);
    expect(new Oklcha(0, 0, 10).mix(new Oklcha(1, 1, 350), 0.25).hue).toBe(5);
    near(v(new Hsla(350, 0, 0, 0).mix(new Hsla(10, 1, 1, 1), 0.25), "saturation", "lightness", "alpha"), [0.25, 0.25, 0.25]);
    near(v(new Srgba(0, 0, 0).mix(new Srgba(1, 0.5, 0), 0.5), "red", "green", "blue"), [0.5, 0.25, 0]);
    expect(new Srgba(1, 0.5, 0).to_hex()).toBe("#FF8000");
    expect(new Srgba(1, 0.5, 0, 0.5).to_hex()).toBe("#FF800080");
  });
});
