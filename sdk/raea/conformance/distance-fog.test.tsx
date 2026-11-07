// <Fog> on the camera (Bevy's DistanceFog) and FogFalloff against Bevy (docs/control.feature FC0a: atmospheric_fog's Atmospheric falloff and the
// sun's colour in the fog). The f32 bits come from bevy_pbr fog.rs's own constructors run on bevy_color (a scratch
// truth crate against the vendored bevy-0.20.0), so a change to the order of the multiplies or the colour space shows.
import { expect, test } from "bun:test";
import type { ReactElement } from "react";
import { FogFalloff } from "raea/pbr";
import { Fog } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Transform } from "raea/transform";
import { LinearRgba, Srgba } from "raea/color";

const bits = (x: number) => { const b = new DataView(new ArrayBuffer(4)); b.setFloat32(0, x); return b.getUint32(0).toString(16).padStart(8, "0"); };
type Atmo = { Atmospheric: { extinction: number[]; inscattering: number[] } };
const both = (fo: unknown) => { const a = (fo as Atmo).Atmospheric; return [...a.extinction, ...a.inscattering].map(bits); };
type El = ReactElement<{ type?: string; value?: Record<string, unknown>; children?: unknown }>;

test("from_visibility_colors is Bevy's, bit for bit (atmospheric_fog's own numbers)", () => {
  expect(both(FogFalloff.from_visibility_colors(15, new Srgba(0.35, 0.5, 0.66, 1), new Srgba(0.8, 0.844, 1, 1))))
    .toEqual(["3e195ae6", "3dd48644", "3d5276c8", "3d4f9a0d", "3d900172", "3e4c8237"]);
});

test("alpha scales each by alpha ^ e; a LinearRgba is taken as linear, an Srgba linearised", () => {
  expect(both(FogFalloff.from_visibility_contrast_colors(30, 0.02, new Srgba(0.1, 0.2, 0.3, 0.5), new LinearRgba(0.6, 0.7, 0.8, 0.9))))
    .toEqual(["3c9df0ca", "3c942144", "3c8401a0", "3cc818df", "3d181f26", "3d5ab0bb"]);
});

test("from_visibility and from_visibility_squared: Koschmieder's 5%", () => {
  expect(bits((FogFalloff.from_visibility(15) as { Exponential: { density: number } }).Exponential.density)).toBe("3e4c8237");
  expect(bits((FogFalloff.from_visibility_squared(15) as { ExponentialSquared: { density: number } }).ExponentialSquared.density)).toBe("3dec507d");
});

/** the DistanceFog a <Camera3d> puts on itself for its <Fog> child */
function onCamera(fog: ReactElement) {
  const cam = Camera3d({ transform: Transform.fromXyz(0, 0, 5), children: fog }) as El;
  const found: El[] = [];
  const walk = (n: unknown) => {
    if (Array.isArray(n)) return n.forEach(walk);
    const el = n as El | null;
    if (!el || typeof el !== "object" || !("props" in el)) return;
    if (el.props.type === "bevy_pbr::fog::DistanceFog") found.push(el);
    walk(el.props.children);
  };
  walk(cam);
  expect(found.length).toBe(1);
  return found[0]!.props.value!;
}

test("a <Fog> in the <Camera3d> is bevy_pbr's DistanceFog on it, Bevy's defaults filled in", () => {
  expect(onCamera(<Fog />)).toEqual({
    color: { Srgba: { red: 1, green: 1, blue: 1, alpha: 1 } },
    directional_light_color: { LinearRgba: { red: 0, green: 0, blue: 0, alpha: 0 } },
    directional_light_exponent: 8,
    falloff: { Linear: { start: 0, end: 100 } },
  });
  const sun = onCamera(<Fog color="#ff0000" directional_light_color={new Srgba(1, 0.95, 0.85, 0.5)} directional_light_exponent={30} />);
  expect(sun.color).toEqual({ Srgba: { red: 1, green: 0, blue: 0, alpha: 1 } });
  expect(sun.directional_light_color).toEqual({ Srgba: { red: 1, green: 0.95, blue: 0.85, alpha: 0.5 } });
  expect(sun.directional_light_exponent).toBe(30);
  expect(onCamera(<Fog near={2} far={9} />).falloff).toEqual({ Linear: { start: 2, end: 9 } });
  expect(onCamera(<Fog density={0.2} />).falloff).toEqual({ Exponential: { density: 0.2 } });
  expect(onCamera(<Fog color={new LinearRgba(0.1, 0.2, 0.3, 1)} />).color).toEqual({ LinearRgba: { red: 0.1, green: 0.2, blue: 0.3, alpha: 1 } });
  expect(() => onCamera(<Fog directional_light_exponent={-1} />)).toThrow(RangeError);
  expect(() => onCamera(<Fog falloff={FogFalloff.from_visibility(10)} near={1} />)).toThrow(TypeError);
});

test("in the Canvas3D, Bevy's camera-only fields are refused and Bevy's colours become the hex the scene's fog reads", () => {
  expect(() => Fog({ falloff: FogFalloff.from_visibility(10) })).toThrow(/inside the <Camera3d>/);
  expect(() => Fog({ directional_light_color: "#ffffff" })).toThrow(/inside the <Camera3d>/);
  const hex = (el: unknown) => JSON.stringify(el).match(/#[0-9a-fA-F]{6,8}/)?.[0]?.toLowerCase();
  expect(hex(Fog({ color: new Srgba(1, 0, 0, 1) }))).toBe("#ff0000");
  expect(hex(Fog({ color: new LinearRgba(0.2, 0, 0, 1) }))).toBe("#7c0000");
});
